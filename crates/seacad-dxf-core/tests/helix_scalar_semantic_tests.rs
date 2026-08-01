use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HELIX_SCALAR_ROLES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfHelixConstraintType,
    DxfHelixHandedness, DxfHelixScalarDirectory, DxfHelixScalarEntry, DxfHelixScalarIssue,
    DxfHelixScalarValue, DxfHelixValueRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type SemanticEvidence = (
    DxfHelixValueRole,
    DxfSemanticValueState,
    Option<DxfHelixScalarValue>,
    Option<DxfHelixScalarIssue>,
);

#[test]
fn every_dialect_has_ascii_binary_helix_scalar_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_semantics = ascii.helix_scalar_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_semantics = binary.helix_scalar_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_semantics, version)?;
        assert_expected(&binary_semantics, version)?;
        assert_eq!(evidence(&ascii_semantics), evidence(&binary_semantics));
    }
    Ok(())
}

#[test]
fn absent_duplicate_invalid_domain_and_nonfinite_states_are_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n\
0\nHELIX\n100\nAcDbHelix\n90\n1\n90\n2\n91\n-1\n40\n.\n41\n0\n42\n-3\n290\n2\n280\n3\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let semantics = document.helix_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(semantics.entries().len(), DXF_HELIX_SCALAR_ROLES.len() * 2);
    assert!(
        semantics.entries()[..DXF_HELIX_SCALAR_ROLES.len()]
            .iter()
            .all(|entry| entry.semantic().state() == DxfSemanticValueState::Absent)
    );

    let second_raw = semantics.card_directory().evidence_directory().records()[1]
        .entity()
        .record()
        .ordinal();
    let major = semantic_for(&semantics, second_raw, DxfHelixValueRole::MajorVersion)?;
    assert_eq!(
        major.invalid_issue(),
        Some(&DxfHelixScalarIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(major.raw_provenance(), None);

    let maintenance = semantic_for(
        &semantics,
        second_raw,
        DxfHelixValueRole::MaintenanceVersion,
    )?;
    assert_eq!(
        maintenance.value(),
        Some(&DxfHelixScalarValue::MaintenanceVersion(-1))
    );
    let radius = semantic_for(&semantics, second_raw, DxfHelixValueRole::Radius)?;
    assert!(matches!(
        radius.invalid_issue(),
        Some(DxfHelixScalarIssue::InvalidAsciiNumber(_))
    ));
    assert!(radius.raw_provenance().is_some());
    assert_eq!(
        semantic_for(&semantics, second_raw, DxfHelixValueRole::Turns)?.value(),
        Some(&DxfHelixScalarValue::Turns(
            seacad_dxf_core::DxfDouble::from_f64(0.0)
        ))
    );
    assert_eq!(
        semantic_for(&semantics, second_raw, DxfHelixValueRole::Handedness)?.invalid_issue(),
        Some(&DxfHelixScalarIssue::InvalidHandedness(2))
    );
    assert_eq!(
        semantic_for(&semantics, second_raw, DxfHelixValueRole::ConstraintType)?.invalid_issue(),
        Some(&DxfHelixScalarIssue::InvalidConstraintType(3))
    );

    let binary_bytes = binary_nonfinite_fixture()?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    let binary_semantics = binary.helix_scalar_directory(&DxfCancellationToken::default())?;
    let raw = binary_semantics
        .card_directory()
        .evidence_directory()
        .records()[0]
        .entity()
        .record()
        .ordinal();
    assert!(matches!(
        semantic_for(&binary_semantics, raw, DxfHelixValueRole::Turns)?.invalid_issue(),
        Some(DxfHelixScalarIssue::NonFiniteDouble(value)) if value.to_f64().is_nan()
    ));
    Ok(())
}

#[test]
fn cancellation_lookup_provenance_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_scalar_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_scalar_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let semantics = document.helix_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(semantics.entry(u64::MAX), None);
    assert_eq!(semantics.entries_for_raw_record(u64::MAX), None);
    assert_eq!(
        semantics.entry_for_role(u64::MAX, DxfHelixValueRole::Radius),
        None
    );
    assert_eq!(
        semantics.source_id(),
        semantics.card_directory().source_id()
    );
    let first = semantics.entries()[0];
    assert_eq!(
        first.semantic().field_provenance().schema_namespace(),
        "entity.helix"
    );
    assert_eq!(
        first.semantic().field_provenance().schema_field_id(),
        "major_version"
    );
    assert_eq!(
        first.semantic().field_provenance().document_source_id(),
        semantics.source_id()
    );
    assert_copy::<DxfHelixScalarEntry>();
    assert_send_sync::<DxfHelixScalarDirectory>();
    assert!(size_of::<DxfHelixScalarEntry>() <= 288);
    Ok(())
}

fn assert_expected(
    semantics: &DxfHelixScalarDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(semantics.entries().len(), DXF_HELIX_SCALAR_ROLES.len());
    let record = semantics.card_directory().evidence_directory().records()[0];
    let raw = record.entity().record().ordinal();
    let entries = semantics
        .entries_for_raw_record(raw)
        .ok_or(io::Error::other("scalar entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.role()).collect::<Vec<_>>(),
        DXF_HELIX_SCALAR_ROLES
    );
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::MajorVersion)?.value(),
        Some(&DxfHelixScalarValue::MajorVersion(1))
    );
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::MaintenanceVersion)?.value(),
        Some(&DxfHelixScalarValue::MaintenanceVersion(0))
    );
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::Radius)?.value(),
        Some(&DxfHelixScalarValue::Radius(
            seacad_dxf_core::DxfDouble::from_f64(4.0)
        ))
    );
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::Turns)?.value(),
        Some(&DxfHelixScalarValue::Turns(
            seacad_dxf_core::DxfDouble::from_f64(2.5)
        ))
    );
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::TurnHeight)?.value(),
        Some(&DxfHelixScalarValue::TurnHeight(
            seacad_dxf_core::DxfDouble::from_f64(-3.0)
        ))
    );
    let expected_handedness = if version == DxfAcadVersion::Ac1009 {
        None
    } else {
        Some(&DxfHelixScalarValue::Handedness(DxfHelixHandedness::Right))
    };
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::Handedness)?.value(),
        expected_handedness
    );
    let expected_constraint = if version == DxfAcadVersion::Ac1009 {
        None
    } else {
        Some(&DxfHelixScalarValue::ConstraintType(
            DxfHelixConstraintType::Height,
        ))
    };
    assert_eq!(
        semantic_for(semantics, raw, DxfHelixValueRole::ConstraintType)?.value(),
        expected_constraint
    );
    Ok(())
}

fn semantic_for(
    semantics: &DxfHelixScalarDirectory,
    raw: u64,
    role: DxfHelixValueRole,
) -> Result<seacad_dxf_core::DxfHelixScalarSemanticValue, io::Error> {
    let entry = semantics
        .entry_for_role(raw, role)
        .ok_or(io::Error::other("scalar entry"))?;
    Ok(*entry.semantic())
}

fn evidence(semantics: &DxfHelixScalarDirectory) -> Vec<SemanticEvidence> {
    semantics
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.role(),
                entry.semantic().state(),
                entry.semantic().value().copied(),
                entry.semantic().invalid_issue().copied(),
            )
        })
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let modern_tail = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "290\n1\n280\n2\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n42\n-3\n90\n1\n\
41\n2.5\n91\n0\n40\n4\n{modern_tail}0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    push_double(&mut bytes, version, 42, -3.0)?;
    push_i32(&mut bytes, version, 90, 1)?;
    push_double(&mut bytes, version, 41, 2.5)?;
    push_i32(&mut bytes, version, 91, 0)?;
    push_double(&mut bytes, version, 40, 4.0)?;
    if version != DxfAcadVersion::Ac1009 {
        push_code(&mut bytes, version, 290)?;
        bytes.push(1);
        push_i16(&mut bytes, version, 280, 2)?;
    }
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_nonfinite_fixture() -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_prefix(version)?;
    push_double_bits(&mut bytes, version, 41, f64::NAN.to_bits())?;
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    Ok(bytes)
}

fn binary_suffix(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
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

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
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
    push_double_bits(bytes, version, code, value.to_bits())
}

fn push_double_bits(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    bits: u64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
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
