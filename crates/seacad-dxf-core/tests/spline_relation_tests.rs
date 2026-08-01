use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineFlagsSemantic, DxfSplineLinearPlanarRelation,
    DxfSplinePlanarNormalRelation, DxfSplineRationalWeightRelation, DxfSplineRelationDirectory,
    DxfSplineRelationEntry, DxfSplineVectorSemanticState, NoopDxfReadObserver,
};

type RelationEvidence = (
    DxfSplineLinearPlanarRelation,
    DxfSplineRationalWeightRelation,
    DxfSplinePlanarNormalRelation,
);

#[test]
fn every_dialect_has_ascii_binary_relation_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_relations = ascii.spline_relation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_relations = binary.spline_relation_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_relations)?;
        assert_expected(&binary_relations)?;
        assert_eq!(evidence(&ascii_relations), evidence(&binary_relations));
    }
    Ok(())
}

#[test]
fn unavailable_flags_and_normal_edge_states_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n10\n0\n41\n.\n210\n1\n\
0\nSPLINE\n70\n8\n70\n0\n10\n0\n\
0\nSPLINE\n70\n.\n10\n0\n\
0\nSPLINE\n70\n8\n10\n0\n\
0\nSPLINE\n70\n8\n10\n0\n210\n0\n\
0\nSPLINE\n70\n8\n10\n0\n210\n.\n\
0\nSPLINE\n70\n0\n10\n0\n210\n1\n210\n2\n\
0\nSPLINE\n70\n4\n10\n0\n10\n1\n41\n.\n41\n-1\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let relations = document.spline_relation_directory(&DxfCancellationToken::default())?;
    assert_eq!(relations.entries().len(), 8);

    let absent = relations.entries()[0];
    assert_eq!(absent.flags(), DxfSplineFlagsSemantic::Absent);
    assert_unavailable_flags(absent);

    let multiple = relations.entries()[1];
    assert_eq!(
        multiple.flags(),
        DxfSplineFlagsSemantic::Multiple {
            occurrence_count: 2
        }
    );
    assert_unavailable_flags(multiple);

    let invalid = relations.entries()[2];
    assert_eq!(
        invalid.flags(),
        DxfSplineFlagsSemantic::Invalid(DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 })
    );
    assert_unavailable_flags(invalid);

    assert_eq!(
        relations.entries()[3].planar_normal(),
        DxfSplinePlanarNormalRelation::PlanarMissing
    );
    assert_eq!(
        relations.entries()[4].planar_normal(),
        DxfSplinePlanarNormalRelation::PlanarExplicit { is_zero: true }
    );
    let DxfSplinePlanarNormalRelation::PlanarUnavailable { issue } =
        relations.entries()[5].planar_normal()
    else {
        return Err(io::Error::other("planar unavailable normal").into());
    };
    assert!(issue.invalid_components().has_x());
    assert!(!issue.is_missing_x());

    let DxfSplinePlanarNormalRelation::NonPlanarUnexpected { state } =
        relations.entries()[6].planar_normal()
    else {
        return Err(io::Error::other("unexpected nonplanar normal").into());
    };
    let DxfSplineVectorSemanticState::Unavailable(issue) = state else {
        return Err(io::Error::other("duplicate normal evidence").into());
    };
    assert!(issue.duplicate_components().has_x());

    assert_eq!(
        relations.entries()[7].rational_weight(),
        DxfSplineRationalWeightRelation::ExplicitMatched {
            rational: true,
            weight_count: 2,
            invalid_count: 1,
            non_positive_count: 1
        }
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.spline_relation_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_relation_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let relations = document.spline_relation_directory(&DxfCancellationToken::default())?;
    assert_eq!(relations.entry(u64::MAX), None);
    assert_eq!(relations.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        relations.source_id(),
        relations.semantic_directory().source_id()
    );
    assert_copy::<DxfSplineRelationEntry>();
    assert_copy::<DxfSplineLinearPlanarRelation>();
    assert_copy::<DxfSplineRationalWeightRelation>();
    assert_copy::<DxfSplinePlanarNormalRelation>();
    assert_send_sync::<DxfSplineRelationDirectory>();
    assert!(size_of::<DxfSplineRelationEntry>() <= 160);
    Ok(())
}

fn assert_expected(directory: &DxfSplineRelationDirectory) -> Result<(), Box<dyn Error>> {
    let entries = directory.entries();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].ordinal(), 0);
    assert_eq!(directory.entry(0), Some(entries[0]));
    let first_raw = entries[0].record().record().ordinal();
    assert_eq!(directory.entry_for_raw_record(first_raw), Some(entries[0]));
    assert_eq!(
        entries[0].linear_planar(),
        DxfSplineLinearPlanarRelation::LinearWithoutPlanar
    );
    assert_eq!(
        entries[0].rational_weight(),
        DxfSplineRationalWeightRelation::ImplicitUnit {
            rational: false,
            control_point_count: 2
        }
    );
    assert_eq!(
        entries[0].planar_normal(),
        DxfSplinePlanarNormalRelation::NonPlanarOmitted
    );

    assert_eq!(
        entries[1].linear_planar(),
        DxfSplineLinearPlanarRelation::LinearPlanar
    );
    assert_eq!(
        entries[1].rational_weight(),
        DxfSplineRationalWeightRelation::ExplicitMatched {
            rational: true,
            weight_count: 2,
            invalid_count: 0,
            non_positive_count: 0
        }
    );
    assert_eq!(
        entries[1].planar_normal(),
        DxfSplinePlanarNormalRelation::PlanarExplicit { is_zero: false }
    );

    assert_eq!(
        entries[2].rational_weight(),
        DxfSplineRationalWeightRelation::ImplicitUnit {
            rational: true,
            control_point_count: 2
        }
    );
    assert_eq!(
        entries[3].rational_weight(),
        DxfSplineRationalWeightRelation::ExplicitCountMismatch {
            rational: false,
            weight_count: 1,
            control_point_count: 2,
            invalid_count: 0,
            non_positive_count: 0
        }
    );
    let DxfSplinePlanarNormalRelation::NonPlanarUnexpected {
        state: DxfSplineVectorSemanticState::Explicit(normal),
    } = entries[4].planar_normal()
    else {
        return Err(io::Error::other("explicit unexpected normal").into());
    };
    assert!(!normal.is_zero());
    Ok(())
}

fn assert_unavailable_flags(entry: DxfSplineRelationEntry) {
    assert_eq!(
        entry.linear_planar(),
        DxfSplineLinearPlanarRelation::UnavailableFlags
    );
    assert_eq!(
        entry.rational_weight(),
        DxfSplineRationalWeightRelation::UnavailableFlags
    );
    assert_eq!(
        entry.planar_normal(),
        DxfSplinePlanarNormalRelation::UnavailableFlags
    );
}

fn evidence(directory: &DxfSplineRelationDirectory) -> Vec<RelationEvidence> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.linear_planar(),
                entry.rational_weight(),
                entry.planar_normal(),
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n70\n16\n10\n0\n10\n1\n\
0\nSPLINE\n70\n28\n10\n0\n10\n1\n41\n1\n41\n0.5\n210\n1\n\
0\nSPLINE\n70\n4\n10\n0\n10\n1\n\
0\nSPLINE\n70\n0\n10\n0\n10\n1\n41\n2\n\
0\nSPLINE\n70\n0\n10\n0\n210\n1\n\
0\nENDSEC\n0\nEOF\n"
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
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_record(&mut bytes, version, 16, &[0.0, 1.0], &[], None)?;
    push_record(&mut bytes, version, 28, &[0.0, 1.0], &[1.0, 0.5], Some(1.0))?;
    push_record(&mut bytes, version, 4, &[0.0, 1.0], &[], None)?;
    push_record(&mut bytes, version, 0, &[0.0, 1.0], &[2.0], None)?;
    push_record(&mut bytes, version, 0, &[0.0], &[], Some(1.0))?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_record(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    flags: i16,
    controls: &[f64],
    weights: &[f64],
    normal_x: Option<f64>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"SPLINE")?;
    push_i16(bytes, version, 70, flags)?;
    for value in controls {
        push_double(bytes, version, 10, *value)?;
    }
    for value in weights {
        push_double(bytes, version, 41, *value)?;
    }
    if let Some(value) = normal_x {
        push_double(bytes, version, 210, value)?;
    }
    Ok(())
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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
