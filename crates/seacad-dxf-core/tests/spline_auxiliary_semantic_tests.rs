use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineAuxiliarySemanticDirectory, DxfSplineVectorKind,
    DxfSplineVectorSemanticEntry, DxfSplineVectorSemanticState, DxfSplineWeightSemanticEntry,
    DxfSplineWeightSemanticState, DxfSplineWeightValueState, NoopDxfReadObserver,
};

type VectorEvidence = (DxfSplineVectorKind, u8, Option<[u64; 3]>, u8, u8, bool);

#[test]
fn every_dialect_has_ascii_binary_effective_value_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.spline_auxiliary_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.spline_auxiliary_semantic_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            weight_evidence(&ascii_directory)?,
            weight_evidence(&binary_directory)?
        );
        assert_eq!(
            vector_evidence(&ascii_directory),
            vector_evidence(&binary_directory)
        );
    }
    Ok(())
}

#[test]
fn invalid_nonpositive_duplicate_and_missing_x_states_are_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n10\n0\n10\n1\n10\n2\n\
41\n0\n41\n-1\n41\n.\n12\n1\n12\n2\n22\n.\n23\n5\n210\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.spline_auxiliary_semantic_directory(&DxfCancellationToken::default())?;

    let weight = directory.weights()[0];
    assert_eq!(
        weight.state(),
        DxfSplineWeightSemanticState::ExplicitMatched { weight_count: 3 }
    );
    let values = directory
        .values_for_weight(weight.ordinal())
        .ok_or(io::Error::other("weight values"))?;
    assert!(matches!(
        values[0],
        DxfSplineWeightValueState::NonPositive { weight, .. } if weight.to_f64() == 0.0
    ));
    assert!(matches!(
        values[1],
        DxfSplineWeightValueState::NonPositive { weight, .. } if weight.to_f64() == -1.0
    ));
    assert!(matches!(
        values[2],
        DxfSplineWeightValueState::Invalid {
            issue: DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 },
            ..
        }
    ));

    let raw = weight.record().record().ordinal();
    let start = state(&directory, raw, DxfSplineVectorKind::StartTangent)?;
    let DxfSplineVectorSemanticState::Unavailable(start_issue) = start else {
        return Err(io::Error::other("unavailable start tangent").into());
    };
    assert!(start_issue.duplicate_components().has_x());
    assert!(start_issue.invalid_components().has_y());
    assert!(!start_issue.is_missing_x());

    let end = state(&directory, raw, DxfSplineVectorKind::EndTangent)?;
    let DxfSplineVectorSemanticState::Unavailable(end_issue) = end else {
        return Err(io::Error::other("unavailable end tangent").into());
    };
    assert!(end_issue.is_missing_x());
    assert!(end_issue.duplicate_components().is_empty());
    assert!(end_issue.invalid_components().is_empty());

    let normal = state(&directory, raw, DxfSplineVectorKind::Normal)?;
    let DxfSplineVectorSemanticState::Explicit(normal) = normal else {
        return Err(io::Error::other("explicit normal").into());
    };
    assert!(normal.is_zero());
    assert!(normal.explicit_components().has_x());
    assert!(!normal.explicit_components().has_y());
    assert_eq!(normal.y().to_f64(), 0.0);
    assert_eq!(normal.z().to_f64(), 0.0);
    Ok(())
}

#[test]
fn cancellation_bounds_and_public_traits_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.spline_auxiliary_semantic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_auxiliary_semantic_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.spline_auxiliary_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.values_for_weight(u64::MAX), None);
    assert_eq!(
        directory.vector_for_kind(u64::MAX, DxfSplineVectorKind::Normal),
        None
    );
    assert_copy::<DxfSplineWeightSemanticEntry>();
    assert_copy::<DxfSplineWeightValueState>();
    assert_copy::<DxfSplineVectorSemanticEntry>();
    assert_copy::<DxfSplineVectorSemanticState>();
    assert_send_sync::<DxfSplineAuxiliarySemanticDirectory>();
    assert!(size_of::<DxfSplineWeightSemanticEntry>() <= 80);
    assert!(size_of::<DxfSplineVectorSemanticEntry>() <= 96);
    Ok(())
}

fn assert_directory(directory: &DxfSplineAuxiliarySemanticDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.weights().len(), 3);
    assert_eq!(directory.weight_values().len(), 3);
    assert_eq!(directory.vectors().len(), 9);
    assert_eq!(
        directory.source_id(),
        directory.auxiliary_directory().source_id()
    );

    let first = directory.weights()[0];
    assert_eq!(
        first.state(),
        DxfSplineWeightSemanticState::ExplicitMatched { weight_count: 2 }
    );
    assert_eq!(first.value_range().len(), 2);
    let first_values = directory
        .values_for_weight(first.ordinal())
        .ok_or(io::Error::other("first weight values"))?;
    assert_eq!(weight_bits(first_values[0])?, 1.0_f64.to_bits());
    assert_eq!(weight_bits(first_values[1])?, 0.5_f64.to_bits());

    let second = directory.weights()[1];
    assert_eq!(
        second.state(),
        DxfSplineWeightSemanticState::ImplicitUnit {
            control_point_count: 2
        }
    );
    assert!(second.value_range().is_empty());
    assert!(
        directory
            .values_for_weight(second.ordinal())
            .ok_or(io::Error::other("implicit values"))?
            .is_empty()
    );

    let third = directory.weights()[2];
    assert_eq!(
        third.state(),
        DxfSplineWeightSemanticState::CountMismatch {
            weight_count: 1,
            control_point_count: 2
        }
    );

    let raw = first.record().record().ordinal();
    let start = state(directory, raw, DxfSplineVectorKind::StartTangent)?;
    let DxfSplineVectorSemanticState::Explicit(start) = start else {
        return Err(io::Error::other("explicit start tangent").into());
    };
    assert_eq!(start.x().to_f64(), 1.0);
    assert_eq!(start.y().to_f64(), 0.0);
    assert_eq!(start.z().to_f64(), 3.0);
    assert!(start.explicit_components().has_x());
    assert!(!start.explicit_components().has_y());
    assert!(start.explicit_components().has_z());
    assert_eq!(
        state(directory, raw, DxfSplineVectorKind::EndTangent)?,
        DxfSplineVectorSemanticState::Absent
    );
    let DxfSplineVectorSemanticState::Explicit(normal) =
        state(directory, raw, DxfSplineVectorKind::Normal)?
    else {
        return Err(io::Error::other("explicit normal").into());
    };
    assert!(normal.explicit_components().is_complete());

    let second_raw = second.record().record().ordinal();
    let DxfSplineVectorSemanticState::Unavailable(issue) =
        state(directory, second_raw, DxfSplineVectorKind::EndTangent)?
    else {
        return Err(io::Error::other("missing-x end tangent").into());
    };
    assert!(issue.is_missing_x());
    Ok(())
}

fn state(
    directory: &DxfSplineAuxiliarySemanticDirectory,
    raw: u64,
    kind: DxfSplineVectorKind,
) -> Result<DxfSplineVectorSemanticState, io::Error> {
    directory
        .vector_for_kind(raw, kind)
        .map(DxfSplineVectorSemanticEntry::state)
        .ok_or(io::Error::other("vector semantic"))
}

fn weight_evidence(
    directory: &DxfSplineAuxiliarySemanticDirectory,
) -> Result<Vec<(DxfSplineWeightSemanticState, Vec<u64>)>, io::Error> {
    directory
        .weights()
        .iter()
        .copied()
        .map(|entry| {
            let values = directory
                .values_for_weight(entry.ordinal())
                .ok_or(io::Error::other("weight evidence"))?
                .iter()
                .copied()
                .map(weight_bits)
                .collect::<Result<Vec<_>, _>>()?;
            Ok((entry.state(), values))
        })
        .collect()
}

fn weight_bits(state: DxfSplineWeightValueState) -> Result<u64, io::Error> {
    match state {
        DxfSplineWeightValueState::Explicit { weight, .. } => Ok(weight.to_bits()),
        _ => Err(io::Error::other("usable weight")),
    }
}

fn vector_evidence(directory: &DxfSplineAuxiliarySemanticDirectory) -> Vec<VectorEvidence> {
    directory
        .vectors()
        .iter()
        .map(|entry| match entry.state() {
            DxfSplineVectorSemanticState::Absent => (entry.kind(), 0, None, 0, 0, false),
            DxfSplineVectorSemanticState::Explicit(value) => (
                entry.kind(),
                1,
                Some([
                    value.x().to_bits(),
                    value.y().to_bits(),
                    value.z().to_bits(),
                ]),
                component_bits(value.explicit_components()),
                0,
                false,
            ),
            DxfSplineVectorSemanticState::Unavailable(issue) => (
                entry.kind(),
                2,
                None,
                component_bits(issue.duplicate_components()),
                component_bits(issue.invalid_components()),
                issue.is_missing_x(),
            ),
            _ => (entry.kind(), u8::MAX, None, 0, 0, false),
        })
        .collect()
}

fn component_bits(components: seacad_dxf_core::DxfSplineVectorComponents) -> u8 {
    u8::from(components.has_x())
        | (u8::from(components.has_y()) << 1)
        | (u8::from(components.has_z()) << 2)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n10\n0\n10\n1\n41\n1\n41\n0.5\n\
12\n1\n32\n3\n210\n0\n220\n0\n230\n1\n\
0\nSPLINE\n10\n0\n10\n1\n23\n2\n\
0\nSPLINE\n10\n0\n10\n1\n41\n2\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"SPLINE"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (10, 0.0),
        (10, 1.0),
        (41, 1.0),
        (41, 0.5),
        (12, 1.0),
        (32, 3.0),
        (210, 0.0),
        (220, 0.0),
        (230, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"SPLINE")?;
    for (code, value) in [(10, 0.0), (10, 1.0), (23, 2.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"SPLINE")?;
    for (code, value) in [(10, 0.0), (10, 1.0), (41, 2.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

struct CancellingSource<'a> {
    bytes: &'a [u8],
    token: DxfCancellationToken,
    armed: AtomicBool,
}

impl<'a> CancellingSource<'a> {
    fn new(bytes: &'a [u8], token: DxfCancellationToken) -> Self {
        Self {
            bytes,
            token,
            armed: AtomicBool::new(false),
        }
    }

    fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
}

impl DxfByteSource for CancellingSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        if self.armed.load(Ordering::Acquire) {
            self.token.cancel();
        }
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
