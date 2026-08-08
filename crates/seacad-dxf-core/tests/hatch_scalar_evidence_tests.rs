use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HATCH_SCALAR_ROLES, DxfAcadVersion, DxfAsciiNumericIssue,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfError, DxfHatchScalarDirectory, DxfHatchScalarEntry, DxfHatchScalarIssue,
    DxfHatchScalarOccurrence, DxfHatchScalarRole, DxfHatchScalarValue, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type Evidence = (DxfHatchScalarRole, DxfHatchScalarValue);

#[test]
fn every_dialect_has_ascii_binary_hatch_scalar_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.hatch_scalar_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.hatch_scalar_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
        assert_eq!(evidence(&ascii_directory)?, expected_evidence(version));
        assert_text(&ascii, &ascii_directory)?;
        assert_text(&binary, &binary_directory)?;
    }
    Ok(())
}

#[test]
fn duplicates_invalid_numbers_and_nested_decoys_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n70\n.\n70\n1\n\
91\n2147483648\n91\n2\n41\n1e-9999\n41\n2\n92\n2\n72\n1\n93\n1\n\
10\n1\n20\n2\n11\n3\n21\n4\n40\n5\n42\n6\n53\n7\n43\n8\n44\n9\n\
45\n10\n46\n11\n79\n1\n49\n12\n73\n1\n102\n{APP\n70\n9\n102\n}\n\
1001\nAPP\n1070\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 1);
    assert_eq!(directory.occurrences().len(), 6);
    let values = directory.occurrences();
    assert_eq!(values[0].role(), DxfHatchScalarRole::SolidFillFlag);
    assert_eq!(
        values[0].value(),
        Err(DxfHatchScalarIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert_eq!(values[1].value(), Ok(DxfHatchScalarValue::Int16(1)));
    assert_eq!(
        values[2].value(),
        Err(DxfHatchScalarIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(values[3].value(), Ok(DxfHatchScalarValue::Int32(2)));
    assert_eq!(
        values[4].value(),
        Err(DxfHatchScalarIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[5].value(),
        Ok(DxfHatchScalarValue::Double(DxfDouble::from_f64(2.0)))
    );
    assert!(values.iter().all(|value| !matches!(
        value.group().group_code().value(),
        10 | 11 | 20 | 21 | 40 | 42 | 43 | 44 | 45 | 46 | 49 | 53 | 72 | 73 | 79 | 92 | 93
    )));
    Ok(())
}

#[test]
fn exact_hatch_subclasses_are_independent_and_mesh_is_excluded() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nHATCH\n100\nAcDbHatch\n70\n1\n100\nAcDbHatch\n\
91\n0\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n70\n0\n100\nacdbhatch\n\
71\n1\n0\nMESH\n100\nAcDbSubDMesh\n71\n2\n91\n3\n0\nHATCH\n100\nAcDbHatch\n\
2\nSOLID\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    assert_eq!(directory.occurrences().len(), 3);
    let raw = directory.entries()[0]
        .subclass()
        .entity()
        .record()
        .ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    assert_eq!(
        directory
            .occurrences_for_subclass(directory.entries()[0].subclass_ordinal())
            .ok_or(io::Error::other("first subclass"))?[0]
            .role(),
        DxfHatchScalarRole::SolidFillFlag
    );
    assert_eq!(
        directory
            .occurrences_for_subclass(directory.entries()[1].subclass_ordinal())
            .ok_or(io::Error::other("second subclass"))?[0]
            .role(),
        DxfHatchScalarRole::BoundaryPathCount
    );
    assert_eq!(
        directory.occurrences()[2].role(),
        DxfHatchScalarRole::PatternName
    );
    assert!(
        directory
            .occurrences()
            .iter()
            .all(|value| value.value() != Ok(DxfHatchScalarValue::Int16(2)))
    );
    Ok(())
}

#[test]
fn cancellation_lookups_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_scalar_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_scalar_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.hatch_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry_for_subclass(u64::MAX), None);
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(directory.occurrences_for_subclass(u64::MAX), None);
    assert_eq!(directory.occurrence_for_group(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(DXF_HATCH_SCALAR_ROLES.len(), 25);
    assert_copy::<DxfHatchScalarOccurrence>();
    assert_copy::<DxfHatchScalarEntry>();
    assert_send_sync::<DxfHatchScalarDirectory>();
    assert!(size_of::<DxfHatchScalarOccurrence>() <= 96);
    assert!(size_of::<DxfHatchScalarEntry>() <= 256);
    Ok(())
}

fn assert_directory(directory: &DxfHatchScalarDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 1);
    let entry = directory.entries()[0];
    let values = directory
        .occurrences_for_subclass(entry.subclass_ordinal())
        .ok_or(io::Error::other("hatch occurrences"))?;
    assert_eq!(values.len() as u64, entry.occurrence_range().len());
    assert_eq!(
        directory
            .entries_for_raw_record(entry.subclass().entity().record().ordinal())
            .len(),
        1
    );
    for value in values.iter().copied() {
        assert_eq!(
            directory.occurrence_for_group(value.group().occurrence()),
            Some(value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfHatchScalarDirectory) -> Result<Vec<Evidence>, io::Error> {
    directory
        .occurrences()
        .iter()
        .map(|value| {
            Ok((
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("valid scalar"))?,
            ))
        })
        .collect()
}

fn expected_evidence(version: DxfAcadVersion) -> Vec<Evidence> {
    use DxfHatchScalarRole::*;
    let mut values = vec![
        (ElevationZ, double(3.0)),
        (ExtrusionX, double(0.0)),
        (ExtrusionY, double(0.0)),
        (ExtrusionZ, double(1.0)),
        (PatternName, DxfHatchScalarValue::Text),
        (SolidFillFlag, DxfHatchScalarValue::Int16(0)),
        (AssociativityFlag, DxfHatchScalarValue::Int16(1)),
        (BoundaryPathCount, DxfHatchScalarValue::Int32(1)),
        (HatchStyle, DxfHatchScalarValue::Int16(0)),
        (PatternType, DxfHatchScalarValue::Int16(1)),
        (PatternAngle, double(0.25)),
        (PatternScale, double(2.0)),
        (PatternDoubleFlag, DxfHatchScalarValue::Int16(0)),
        (PatternLineCount, DxfHatchScalarValue::Int16(0)),
        (PixelSize, double(0.01)),
        (SeedPointCount, DxfHatchScalarValue::Int32(1)),
    ];
    if version != DxfAcadVersion::Ac1009 {
        values.extend([
            (GradientKind, DxfHatchScalarValue::Int32(1)),
            (GradientReserved, DxfHatchScalarValue::Int32(0)),
            (GradientColorMode, DxfHatchScalarValue::Int32(0)),
            (GradientColorCount, DxfHatchScalarValue::Int32(2)),
            (GradientRotation, double(0.5)),
            (GradientShift, double(0.25)),
            (GradientTint, double(0.75)),
            (GradientReservedValue, double(1.0)),
            (GradientName, DxfHatchScalarValue::Text),
        ]);
    }
    values
}

fn double(value: f64) -> DxfHatchScalarValue {
    DxfHatchScalarValue::Double(DxfDouble::from_f64(value))
}

fn assert_text(
    document: &impl AsRefRawDocument,
    directory: &DxfHatchScalarDirectory,
) -> Result<(), Box<dyn Error>> {
    let view = document.raw_view();
    for occurrence in directory.occurrences().iter().copied().filter(|value| {
        matches!(
            value.role(),
            DxfHatchScalarRole::PatternName | DxfHatchScalarRole::GradientName
        )
    }) {
        let expected = match occurrence.role() {
            DxfHatchScalarRole::PatternName => b"ANSI31".as_slice(),
            DxfHatchScalarRole::GradientName => b"LINEAR".as_slice(),
            _ => return Err(io::Error::other("text role").into()),
        };
        let len = usize::try_from(occurrence.group().value_payload_span().len())?;
        let mut bytes = vec![0; len];
        view.read_span(occurrence.group().value_payload_span(), &mut bytes)?;
        assert_eq!(bytes, expected);
    }
    Ok(())
}

trait AsRefRawDocument {
    fn raw_view(&self) -> DxfRawDocumentView<'_>;
}

impl AsRefRawDocument for DxfAsciiRawDocument<'_> {
    fn raw_view(&self) -> DxfRawDocumentView<'_> {
        DxfRawDocumentView::from(self)
    }
}

impl AsRefRawDocument for DxfBinaryRawDocument<'_> {
    fn raw_view(&self) -> DxfRawDocumentView<'_> {
        DxfRawDocumentView::from(self)
    }
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
