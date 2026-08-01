use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfHelixAxisDerivationStage, DxfHelixAxisRelation,
    DxfHelixHeightRelation, DxfHelixRadiusDomain, DxfHelixRelationDirectory, DxfHelixRelationEntry,
    DxfHelixTurnsDomain, DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type RelationEvidence = (
    DxfHelixAxisRelation,
    DxfHelixRadiusDomain,
    DxfHelixTurnsDomain,
    DxfHelixHeightRelation,
);

#[test]
fn every_dialect_has_ascii_binary_helix_relation_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_relations = ascii.helix_relation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_relations = binary.helix_relation_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_relations)?;
        assert_expected(&binary_relations)?;
        assert_eq!(evidence(&ascii_relations), evidence(&binary_relations));
    }
    Ok(())
}

#[test]
fn unavailable_domain_flat_residual_and_overflow_states_are_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nHELIX\n100\nAcDbHelix\n\
0\nHELIX\n100\nAcDbHelix\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n\
12\n0\n22\n0\n32\n0\n40\n-1\n41\n0\n42\n0\n\
0\nHELIX\n100\nAcDbHelix\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n1\n\
12\n0\n22\n0\n32\n1\n40\n0\n41\n501\n42\n0\n\
0\nHELIX\n100\nAcDbHelix\n10\n1e308\n20\n0\n30\n0\n11\n-1e308\n21\n0\n31\n0\n\
12\n0\n22\n0\n32\n1\n40\n2\n41\n2\n42\n1e308\n\
0\nHELIX\n100\nAcDbHelix\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n\
12\n0\n22\n.\n32\n1\n40\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let relations = document.helix_relation_directory(&DxfCancellationToken::default())?;
    assert_eq!(relations.entries().len(), 5);

    let absent = relations.entries()[0];
    assert_eq!(absent.axis(), DxfHelixAxisRelation::Unavailable);
    assert_eq!(absent.radius(), DxfHelixRadiusDomain::Unavailable);
    assert_eq!(absent.turns(), DxfHelixTurnsDomain::Unavailable);
    assert_eq!(absent.height(), DxfHelixHeightRelation::Unavailable);

    let zero = relations.entries()[1];
    assert_eq!(
        zero.axis(),
        DxfHelixAxisRelation::ZeroAxis {
            radial_vector: vector([1.0, 0.0, 0.0])
        }
    );
    assert_eq!(
        zero.radius(),
        DxfHelixRadiusDomain::Negative {
            radius: double(-1.0)
        }
    );
    assert_eq!(
        zero.turns(),
        DxfHelixTurnsDomain::NonPositive { turns: double(0.0) }
    );
    assert_eq!(
        zero.height(),
        DxfHelixHeightRelation::Compared {
            turns: double(0.0),
            turn_height: double(0.0),
            axial_height: double(0.0),
            flat: true,
        }
    );

    let non_perpendicular = relations.entries()[2];
    assert!(matches!(
        non_perpendicular.axis(),
        DxfHelixAxisRelation::Compared {
            orthogonality_residual,
            exactly_perpendicular: false,
            ..
        } if orthogonality_residual == double(1.0)
    ));
    assert_eq!(
        non_perpendicular.radius(),
        DxfHelixRadiusDomain::NonNegative {
            radius: double(0.0)
        }
    );
    assert_eq!(
        non_perpendicular.turns(),
        DxfHelixTurnsDomain::AboveCommandLimit {
            turns: double(501.0)
        }
    );
    assert!(matches!(
        non_perpendicular.height(),
        DxfHelixHeightRelation::Compared { flat: true, .. }
    ));

    let overflow = relations.entries()[3];
    assert_eq!(
        overflow.axis(),
        DxfHelixAxisRelation::DerivedNonFinite {
            stage: DxfHelixAxisDerivationStage::RadialVector
        }
    );
    assert_eq!(
        overflow.height(),
        DxfHelixHeightRelation::DerivedNonFinite {
            turns: double(2.0),
            turn_height: double(1.0e308),
        }
    );

    let invalid = relations.entries()[4];
    assert_eq!(invalid.axis(), DxfHelixAxisRelation::Unavailable);
    assert_eq!(invalid.radius(), DxfHelixRadiusDomain::Unavailable);
    assert_eq!(invalid.turns(), DxfHelixTurnsDomain::Unavailable);
    assert_eq!(invalid.height(), DxfHelixHeightRelation::Unavailable);
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_relation_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_relation_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let relations = document.helix_relation_directory(&DxfCancellationToken::default())?;
    let entry = relations.entries()[0];
    let raw = entry.record().entity().record().ordinal();
    assert_eq!(relations.entry(0), Some(entry));
    assert_eq!(relations.entry(u64::MAX), None);
    assert_eq!(relations.entry_for_raw_record(raw), Some(entry));
    assert_eq!(relations.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        relations.source_id(),
        relations.scalar_directory().source_id()
    );
    assert_eq!(
        relations.source_id(),
        relations.vector_directory().source_id()
    );
    assert_copy::<DxfHelixRelationEntry>();
    assert_send_sync::<DxfHelixRelationDirectory>();
    assert!(size_of::<DxfHelixRelationEntry>() <= 320);
    Ok(())
}

fn assert_expected(relations: &DxfHelixRelationDirectory) -> Result<(), Box<dyn Error>> {
    let [entry] = relations.entries() else {
        return Err(io::Error::other("one HELIX relation").into());
    };
    assert_eq!(
        entry.axis(),
        DxfHelixAxisRelation::Compared {
            radial_vector: vector([3.0, 0.0, 0.0]),
            normalized_axis: vector([0.0, 0.0, 1.0]),
            axis_length: double(2.0),
            derived_base_radius: double(3.0),
            orthogonality_residual: double(0.0),
            exactly_perpendicular: true,
        }
    );
    assert_eq!(
        entry.radius(),
        DxfHelixRadiusDomain::NonNegative {
            radius: double(5.0)
        }
    );
    assert_eq!(
        entry.turns(),
        DxfHelixTurnsDomain::WithinCommandLimit { turns: double(4.0) }
    );
    assert_eq!(
        entry.height(),
        DxfHelixHeightRelation::Compared {
            turns: double(4.0),
            turn_height: double(0.5),
            axial_height: double(2.0),
            flat: false,
        }
    );
    Ok(())
}

fn evidence(relations: &DxfHelixRelationDirectory) -> Vec<RelationEvidence> {
    relations
        .entries()
        .iter()
        .copied()
        .map(|entry| (entry.axis(), entry.radius(), entry.turns(), entry.height()))
        .collect()
}

fn double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(value)
}

fn vector(value: [f64; 3]) -> [DxfDouble; 3] {
    value.map(double)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n42\n0.5\n31\n3\n\
10\n1\n22\n0\n41\n4\n11\n4\n30\n3\n12\n0\n21\n2\n40\n5\n\
20\n2\n32\n2\n0\nENDSEC\n0\nEOF\n"
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
        (0, b"HELIX"),
        (100, b"AcDbHelix"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (42, 0.5),
        (31, 3.0),
        (10, 1.0),
        (22, 0.0),
        (41, 4.0),
        (11, 4.0),
        (30, 3.0),
        (12, 0.0),
        (21, 2.0),
        (40, 5.0),
        (20, 2.0),
        (32, 2.0),
    ] {
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
