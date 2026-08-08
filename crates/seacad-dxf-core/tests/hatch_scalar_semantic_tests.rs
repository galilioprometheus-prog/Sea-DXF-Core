use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HATCH_SCALAR_ROLES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchScalarRole, DxfHatchScalarSemanticDirectory, DxfHatchScalarSemanticEntry,
    DxfHatchScalarSemanticIssue, DxfHatchScalarSemanticValue, DxfHatchScalarValue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

type SemanticEvidence = (
    DxfHatchScalarRole,
    DxfSemanticValueState,
    Option<DxfHatchScalarValue>,
    Option<DxfHatchScalarSemanticIssue>,
);

#[test]
fn every_dialect_has_ascii_binary_hatch_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_semantics =
            ascii.hatch_scalar_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_semantics =
            binary.hatch_scalar_semantic_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_semantics, version)?;
        assert_expected(&binary_semantics, version)?;
        assert_eq!(evidence(&ascii_semantics), evidence(&binary_semantics));
    }
    Ok(())
}

#[test]
fn defaults_duplicates_invalid_numbers_and_domains_are_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n0\nHATCH\n100\nAcDbHatch\n\
2\nA\n2\nB\n70\n.\n71\n2\n75\n3\n76\n-1\n77\n2\n78\n-1\n91\n-1\n98\n-1\n\
450\n2\n451\n1\n452\n2\n453\n1\n461\n1.5\n462\n-0.1\n463\n2\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let semantics = document.hatch_scalar_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(semantics.entries().len(), DXF_HATCH_SCALAR_ROLES.len() * 2);
    let first = semantics.card_directory().evidence_directory().entries()[0];
    let first_ordinal = first.subclass_ordinal();
    assert_eq!(
        semantic_for(&semantics, first_ordinal, DxfHatchScalarRole::ExtrusionX)?.value(),
        Some(&double(0.0))
    );
    assert_eq!(
        semantic_for(&semantics, first_ordinal, DxfHatchScalarRole::ExtrusionY)?.state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        semantic_for(&semantics, first_ordinal, DxfHatchScalarRole::ExtrusionZ)?.value(),
        Some(&double(1.0))
    );
    assert_eq!(
        semantic_for(&semantics, first_ordinal, DxfHatchScalarRole::GradientTint)?.state(),
        DxfSemanticValueState::Absent
    );

    let second = semantics.card_directory().evidence_directory().entries()[1];
    let second_ordinal = second.subclass_ordinal();
    let pattern = semantic_for(&semantics, second_ordinal, DxfHatchScalarRole::PatternName)?;
    assert_eq!(
        pattern.invalid_issue(),
        Some(&DxfHatchScalarSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(pattern.raw_provenance(), None);
    let solid = semantic_for(
        &semantics,
        second_ordinal,
        DxfHatchScalarRole::SolidFillFlag,
    )?;
    assert!(matches!(
        solid.invalid_issue(),
        Some(DxfHatchScalarSemanticIssue::InvalidAsciiNumber(_))
    ));
    assert!(solid.raw_provenance().is_some());

    for (role, value) in [
        (
            DxfHatchScalarRole::AssociativityFlag,
            DxfHatchScalarValue::Int16(2),
        ),
        (
            DxfHatchScalarRole::HatchStyle,
            DxfHatchScalarValue::Int16(3),
        ),
        (
            DxfHatchScalarRole::PatternType,
            DxfHatchScalarValue::Int16(-1),
        ),
        (
            DxfHatchScalarRole::PatternDoubleFlag,
            DxfHatchScalarValue::Int16(2),
        ),
        (
            DxfHatchScalarRole::PatternLineCount,
            DxfHatchScalarValue::Int16(-1),
        ),
        (
            DxfHatchScalarRole::BoundaryPathCount,
            DxfHatchScalarValue::Int32(-1),
        ),
        (
            DxfHatchScalarRole::SeedPointCount,
            DxfHatchScalarValue::Int32(-1),
        ),
        (
            DxfHatchScalarRole::GradientKind,
            DxfHatchScalarValue::Int32(2),
        ),
        (
            DxfHatchScalarRole::GradientReserved,
            DxfHatchScalarValue::Int32(1),
        ),
        (
            DxfHatchScalarRole::GradientColorMode,
            DxfHatchScalarValue::Int32(2),
        ),
        (
            DxfHatchScalarRole::GradientColorCount,
            DxfHatchScalarValue::Int32(1),
        ),
        (DxfHatchScalarRole::GradientShift, double(1.5)),
        (DxfHatchScalarRole::GradientTint, double(-0.1)),
        (DxfHatchScalarRole::GradientReservedValue, double(2.0)),
    ] {
        let semantic = semantic_for(&semantics, second_ordinal, role)?;
        assert_eq!(
            semantic.invalid_issue(),
            Some(&DxfHatchScalarSemanticIssue::ValueOutOfDomain(value))
        );
        assert!(semantic.raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn binary_nonfinite_and_duplicate_subclass_provenance_remain_explicit() -> Result<(), Box<dyn Error>>
{
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_prefix(version)?;
    push_double_bits(&mut bytes, version, 52, f64::NAN.to_bits())?;
    push_string(&mut bytes, version, 100, b"AcDbHatch")?;
    push_i16(&mut bytes, version, 70, 1)?;
    binary_suffix(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let semantics = document.hatch_scalar_semantic_directory(&DxfCancellationToken::default())?;
    let entries = semantics.card_directory().evidence_directory().entries();
    assert_eq!(entries.len(), 2);
    let raw = entries[0].subclass().entity().record().ordinal();
    assert_eq!(semantics.entries_for_raw_record(raw).len(), 50);
    let angle = semantic_for(
        &semantics,
        entries[0].subclass_ordinal(),
        DxfHatchScalarRole::PatternAngle,
    )?;
    assert!(matches!(
        angle.invalid_issue(),
        Some(DxfHatchScalarSemanticIssue::NonFiniteDouble(value)) if value.to_f64().is_nan()
    ));
    assert!(angle.raw_provenance().is_some());
    let solid = semantic_for(
        &semantics,
        entries[1].subclass_ordinal(),
        DxfHatchScalarRole::SolidFillFlag,
    )?;
    assert_eq!(solid.value(), Some(&DxfHatchScalarValue::Int16(1)));
    assert!(solid.raw_provenance().is_some());
    Ok(())
}

#[test]
fn cancellation_lookup_field_provenance_and_public_bounds_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_scalar_semantic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_scalar_semantic_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let semantics = document.hatch_scalar_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(semantics.entry(u64::MAX), None);
    assert_eq!(semantics.entries_for_subclass(u64::MAX), None);
    assert!(semantics.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(
        semantics.entry_for_role(u64::MAX, DxfHatchScalarRole::PatternName),
        None
    );
    assert_eq!(
        semantics.source_id(),
        semantics.card_directory().source_id()
    );
    let first = semantics.entries()[0];
    assert_eq!(
        first.semantic().field_provenance().schema_namespace(),
        "entity.hatch"
    );
    assert_eq!(
        first.semantic().field_provenance().schema_field_id(),
        "elevation_z"
    );
    assert_eq!(
        first.semantic().field_provenance().document_source_id(),
        semantics.source_id()
    );
    assert_copy::<DxfHatchScalarSemanticEntry>();
    assert_send_sync::<DxfHatchScalarSemanticDirectory>();
    assert!(size_of::<DxfHatchScalarSemanticEntry>() <= 320);
    Ok(())
}

fn assert_expected(
    semantics: &DxfHatchScalarSemanticDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(semantics.entries().len(), DXF_HATCH_SCALAR_ROLES.len());
    let scalar_entry = semantics.card_directory().evidence_directory().entries()[0];
    let entries = semantics
        .entries_for_subclass(scalar_entry.subclass_ordinal())
        .ok_or(io::Error::other("semantic entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.role()).collect::<Vec<_>>(),
        DXF_HATCH_SCALAR_ROLES
    );
    for entry in entries {
        let high_code = matches!(
            entry.role(),
            DxfHatchScalarRole::GradientKind
                | DxfHatchScalarRole::GradientReserved
                | DxfHatchScalarRole::GradientColorMode
                | DxfHatchScalarRole::GradientColorCount
                | DxfHatchScalarRole::GradientRotation
                | DxfHatchScalarRole::GradientShift
                | DxfHatchScalarRole::GradientTint
                | DxfHatchScalarRole::GradientReservedValue
                | DxfHatchScalarRole::GradientName
        );
        assert_eq!(
            entry.semantic().state(),
            if version == DxfAcadVersion::Ac1009 && high_code {
                DxfSemanticValueState::Absent
            } else {
                DxfSemanticValueState::Explicit
            }
        );
    }
    Ok(())
}

fn semantic_for(
    semantics: &DxfHatchScalarSemanticDirectory,
    subclass: u64,
    role: DxfHatchScalarRole,
) -> Result<DxfHatchScalarSemanticValue, io::Error> {
    semantics
        .entry_for_role(subclass, role)
        .map(|entry| *entry.semantic())
        .ok_or(io::Error::other("semantic entry"))
}

fn evidence(semantics: &DxfHatchScalarSemanticDirectory) -> Vec<SemanticEvidence> {
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

fn double(value: f64) -> DxfHatchScalarValue {
    DxfHatchScalarValue::Double(DxfDouble::from_f64(value))
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let modern = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "450\n1\n451\n0\n452\n0\n453\n2\n460\n0.5\n461\n0.25\n462\n0.75\n463\n1\n470\nLINEAR\n"
            .to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nHATCH\n100\nAcDbHatch\n30\n3\n210\n0\n220\n0\n230\n1\n2\nANSI31\n70\n0\n71\n1\n\
91\n1\n75\n0\n76\n1\n52\n0.25\n41\n2\n77\n0\n78\n0\n47\n0.01\n98\n1\n{modern}\
0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    for (code, value) in [(30, 3.0), (210, 0.0), (220, 0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 2, b"ANSI31")?;
    for (code, value) in [(70, 0), (71, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    for (code, value) in [(75, 0), (76, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 52, 0.25)?;
    push_double(&mut bytes, version, 41, 2.0)?;
    push_i16(&mut bytes, version, 77, 0)?;
    push_i16(&mut bytes, version, 78, 0)?;
    push_double(&mut bytes, version, 47, 0.01)?;
    push_i32(&mut bytes, version, 98, 1)?;
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [(450, 1), (451, 0), (452, 0), (453, 2)] {
            push_i32(&mut bytes, version, code, value)?;
        }
        for (code, value) in [(460, 0.5), (461, 0.25), (462, 0.75), (463, 1.0)] {
            push_double(&mut bytes, version, code, value)?;
        }
        push_string(&mut bytes, version, 470, b"LINEAR")?;
    }
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
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
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
