use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineAuxiliaryDirectory, DxfSplineNumber,
    DxfSplineNumericIssue, DxfSplineVectorComponentState, DxfSplineVectorEntry,
    DxfSplineVectorKind, DxfSplineVectorState, DxfSplineWeightCountDisposition,
    DxfSplineWeightEntry, DxfSplineWeightState, NoopDxfReadObserver,
};

type VectorEvidence = (DxfSplineVectorKind, DxfSplineVectorState, [Option<u64>; 3]);

#[test]
fn every_dialect_has_ascii_binary_weight_and_vector_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_auxiliary_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.spline_auxiliary_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            weight_evidence(&ascii_directory)?,
            weight_evidence(&binary_directory)?
        );
        assert_eq!(
            vector_evidence(&ascii_directory)?,
            vector_evidence(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn numeric_failures_and_subclass_decoys_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n100\nAcDbSpline\n10\n0\n41\n1e-9999\n\
210\n.\n210\n1\n100\nAcDbOther\n41\n7\n12\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_auxiliary_directory(&DxfCancellationToken::default())?;
    let weight = directory.weights()[0];
    assert!(matches!(
        weight.state(),
        DxfSplineWeightState::Explicit {
            weight_count: 1,
            control_point_count: 1,
            disposition: DxfSplineWeightCountDisposition::Matched,
        }
    ));
    let [member] = directory
        .members_for_weight(weight.ordinal())
        .ok_or(io::Error::other("weight members"))?
    else {
        return Err(io::Error::other("one weight member").into());
    };
    assert!(matches!(
        directory
            .value_for_member(*member)
            .ok_or(io::Error::other("weight value"))?
            .value(),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    ));

    let raw = weight.record().record().ordinal();
    let normal = directory
        .vector_for_kind(raw, DxfSplineVectorKind::Normal)
        .ok_or(io::Error::other("normal"))?;
    assert!(matches!(
        normal.state(),
        DxfSplineVectorState::Ambiguous {
            duplicate_components
        } if duplicate_components.has_x()
            && !duplicate_components.has_y()
            && !duplicate_components.has_z()
    ));
    assert_eq!(
        normal.x(),
        DxfSplineVectorComponentState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        directory
            .vector_for_kind(raw, DxfSplineVectorKind::StartTangent)
            .ok_or(io::Error::other("start tangent"))?
            .state(),
        DxfSplineVectorState::Absent
    );
    assert_eq!(
        directory
            .members_for_weight(weight.ordinal())
            .ok_or(io::Error::other("retained weight member"))?
            .len(),
        1
    );
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
        document.spline_auxiliary_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_auxiliary_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.spline_auxiliary_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.weight(u64::MAX), None);
    assert_eq!(directory.vector(u64::MAX), None);
    assert_eq!(directory.weight_for_raw_record(u64::MAX), None);
    assert_eq!(directory.vectors_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.vector_for_kind(u64::MAX, DxfSplineVectorKind::Normal),
        None
    );
    assert_eq!(directory.members_for_weight(u64::MAX), None);
    assert_copy::<DxfSplineWeightEntry>();
    assert_copy::<DxfSplineWeightState>();
    assert_copy::<DxfSplineVectorEntry>();
    assert_copy::<DxfSplineVectorState>();
    assert_send_sync::<DxfSplineAuxiliaryDirectory>();
    assert!(size_of::<DxfSplineWeightEntry>() <= 80);
    assert!(size_of::<DxfSplineVectorEntry>() <= 112);
    Ok(())
}

fn assert_directory(directory: &DxfSplineAuxiliaryDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.weights().len(), 3);
    assert_eq!(directory.vectors().len(), 9);
    assert_eq!(
        directory.source_id(),
        directory.point_directory().source_id()
    );
    let first = directory.weights()[0];
    assert_eq!(directory.weight(first.ordinal()), Some(first));
    assert!(matches!(
        first.state(),
        DxfSplineWeightState::Explicit {
            weight_count: 2,
            control_point_count: 2,
            disposition: DxfSplineWeightCountDisposition::Matched,
        }
    ));
    let first_members = directory
        .members_for_weight(first.ordinal())
        .ok_or(io::Error::other("first weight members"))?;
    assert_eq!(first_members.len(), 2);
    assert_eq!(
        member_double(directory, first_members[0])?,
        1.0_f64.to_bits()
    );
    assert_eq!(
        member_double(directory, first_members[1])?,
        0.5_f64.to_bits()
    );

    let second = directory.weights()[1];
    assert!(matches!(
        second.state(),
        DxfSplineWeightState::Explicit {
            weight_count: 1,
            control_point_count: 2,
            disposition: DxfSplineWeightCountDisposition::Mismatched,
        }
    ));
    let third = directory.weights()[2];
    assert_eq!(
        third.state(),
        DxfSplineWeightState::ImplicitUnit {
            control_point_count: 2
        }
    );
    assert!(
        directory
            .members_for_weight(third.ordinal())
            .ok_or(io::Error::other("implicit members"))?
            .is_empty()
    );

    let raw = first.record().record().ordinal();
    let vectors = directory
        .vectors_for_raw_record(raw)
        .ok_or(io::Error::other("vectors"))?;
    assert_eq!(
        vectors.iter().map(|entry| entry.kind()).collect::<Vec<_>>(),
        [
            DxfSplineVectorKind::StartTangent,
            DxfSplineVectorKind::EndTangent,
            DxfSplineVectorKind::Normal,
        ]
    );
    assert!(matches!(
        vectors[0].state(),
        DxfSplineVectorState::Present { components }
            if components.has_x() && !components.has_y() && components.has_z()
    ));
    assert!(matches!(
        vectors[1].state(),
        DxfSplineVectorState::Ambiguous { duplicate_components }
            if duplicate_components.has_x()
                && !duplicate_components.has_y()
                && !duplicate_components.has_z()
    ));
    assert!(matches!(
        vectors[2].state(),
        DxfSplineVectorState::Present { components } if components.is_complete()
    ));
    for vector in vectors.iter().copied() {
        assert_eq!(directory.vector(vector.ordinal()), Some(vector));
    }
    Ok(())
}

fn weight_evidence(
    directory: &DxfSplineAuxiliaryDirectory,
) -> Result<Vec<(DxfSplineWeightState, Vec<u64>)>, io::Error> {
    directory
        .weights()
        .iter()
        .copied()
        .map(|entry| {
            let values = directory
                .members_for_weight(entry.ordinal())
                .ok_or(io::Error::other("weight evidence"))?
                .iter()
                .copied()
                .map(|member| member_double(directory, member))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((entry.state(), values))
        })
        .collect()
}

fn vector_evidence(
    directory: &DxfSplineAuxiliaryDirectory,
) -> Result<Vec<VectorEvidence>, io::Error> {
    directory
        .vectors()
        .iter()
        .copied()
        .map(|entry| {
            Ok((
                entry.kind(),
                entry.state(),
                [
                    component_double(directory, entry.x())?,
                    component_double(directory, entry.y())?,
                    component_double(directory, entry.z())?,
                ],
            ))
        })
        .collect()
}

fn component_double(
    directory: &DxfSplineAuxiliaryDirectory,
    state: DxfSplineVectorComponentState,
) -> Result<Option<u64>, io::Error> {
    match state {
        DxfSplineVectorComponentState::Absent | DxfSplineVectorComponentState::Multiple { .. } => {
            Ok(None)
        }
        DxfSplineVectorComponentState::Unique(member) => member_double(directory, member).map(Some),
        _ => Err(io::Error::other("unknown vector component state")),
    }
}

fn member_double(
    directory: &DxfSplineAuxiliaryDirectory,
    member: seacad_dxf_core::DxfSplineCardMember,
) -> Result<u64, io::Error> {
    match directory
        .value_for_member(member)
        .ok_or(io::Error::other("member value"))?
        .value()
    {
        Ok(DxfSplineNumber::Double(value)) => Ok(value.to_bits()),
        _ => Err(io::Error::other("member double")),
    }
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n100\nAcDbEntity\n41\n99\n100\nAcDbSpline\n\
10\n0\n10\n1\n41\n1\n41\n0.5\n12\n1\n32\n3\n13\n4\n13\n5\n23\n6\n\
210\n0\n220\n0\n230\n1\n100\nAcDbOther\n41\n7\n12\n8\n\
0\nSPLINE\n10\n0\n10\n1\n41\n2\n0\nSPLINE\n10\n0\n10\n1\n0\nENDSEC\n0\nEOF\n"
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
        (100, b"AcDbEntity"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 41, 99.0)?;
    push_string(&mut bytes, version, 100, b"AcDbSpline")?;
    for (code, value) in [
        (10, 0.0),
        (10, 1.0),
        (41, 1.0),
        (41, 0.5),
        (12, 1.0),
        (32, 3.0),
        (13, 4.0),
        (13, 5.0),
        (23, 6.0),
        (210, 0.0),
        (220, 0.0),
        (230, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 100, b"AcDbOther")?;
    push_double(&mut bytes, version, 41, 7.0)?;
    push_double(&mut bytes, version, 12, 8.0)?;
    push_string(&mut bytes, version, 0, b"SPLINE")?;
    for (code, value) in [(10, 0.0), (10, 1.0), (41, 2.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"SPLINE")?;
    push_double(&mut bytes, version, 10, 0.0)?;
    push_double(&mut bytes, version, 10, 1.0)?;
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
