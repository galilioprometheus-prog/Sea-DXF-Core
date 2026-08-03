use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataOccurrenceKind, DxfEntityXDataPointComponents,
    DxfEntityXDataPointKind, DxfEntityXDataPointMember, DxfEntityXDataPointTuple,
    DxfEntityXDataPointTupleDirectory, DxfEntityXDataPointTupleState, DxfEntityXDataValue,
    DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

type TupleEvidence = (
    DxfEntityXDataPointKind,
    DxfEntityXDataOccurrenceKind,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

#[test]
fn every_supported_version_has_ascii_binary_point_tuple_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = complete_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.entity_xdata_point_tuple_directory(&token())?;

        let binary_bytes = complete_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.entity_xdata_point_tuple_directory(&token())?;

        assert_complete_directory(&ascii_directory)?;
        assert_complete_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn partial_reordered_interrupted_and_orphan_components_remain_exact() -> Result<(), Box<dyn Error>>
{
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = malformed_fixture(format)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_xdata_point_tuple_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_xdata_point_tuple_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };

        assert_eq!(directory.tuples().len(), 11);
        assert_components(directory.tuples()[0], false, true, true);
        assert!(matches!(
            directory.tuples()[0].context(),
            DxfEntityXDataOccurrenceKind::Orphan
        ));
        assert_eq!(
            directory.tuples()[1].state(),
            DxfEntityXDataPointTupleState::Complete
        );
        let invalid_tuple = directory.tuples()[1];
        assert!(matches!(
            directory
                .entry_for_member(
                    invalid_tuple,
                    invalid_tuple.x().ok_or(io::Error::other("x"))?
                )
                .ok_or(io::Error::other("x entry"))?
                .value(),
            DxfEntityXDataValue::Invalid { .. }
        ));
        assert_components(directory.tuples()[2], true, false, true);
        assert_components(directory.tuples()[3], false, true, false);
        assert_components(directory.tuples()[4], false, false, true);
        assert_components(directory.tuples()[5], true, true, false);
        assert_components(directory.tuples()[6], false, true, true);
        assert_components(directory.tuples()[7], true, false, false);
        assert_components(directory.tuples()[8], false, true, true);
        assert_components(directory.tuples()[9], true, false, false);
        assert_components(directory.tuples()[10], false, true, true);

        let applications = directory.typed_directory().xdata_directory().applications();
        assert_eq!(applications.len(), 2);
        assert_eq!(directory.tuples_for_application(applications[0])?.len(), 7);
        assert_eq!(directory.tuples_for_application(applications[1])?.len(), 1);
        assert!(matches!(
            directory.tuples()[8].context(),
            DxfEntityXDataOccurrenceKind::ApplicationValue {
                application_ordinal: 1
            }
        ));
        assert!(matches!(
            directory.tuples()[10].context(),
            DxfEntityXDataOccurrenceKind::Orphan
        ));
        for tuple in directory.tuples().iter().copied() {
            for member in [tuple.x(), tuple.y(), tuple.z()].into_iter().flatten() {
                let entry = directory
                    .entry_for_member(tuple, member)
                    .ok_or(io::Error::other("tuple member"))?;
                assert_eq!(directory.tuple_for_entry(entry), Some(tuple));
            }
        }
    }
    Ok(())
}

#[test]
fn source_cancellation_lookup_and_metadata_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = complete_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_xdata_point_tuple_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = token();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.entity_xdata_point_tuple_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.entity_xdata_point_tuple_directory(&token())?;
    assert_eq!(directory.tuple(u64::MAX), None);
    let non_point = directory.typed_directory().entries()[0];
    assert_eq!(directory.tuple_for_entry(non_point), None);
    assert!(directory.tuples().len() <= directory.typed_directory().entries().len());

    let mut other_bytes = bytes.clone();
    let secret_offset = other_bytes
        .windows(b"SECRET_TUPLE_PAYLOAD".len())
        .position(|window| window == b"SECRET_TUPLE_PAYLOAD")
        .ok_or(io::Error::other("secret offset"))?;
    other_bytes[secret_offset] = b'X';
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.entity_xdata_point_tuple_directory(&token())?;
    assert!(matches!(
        directory
            .tuples_for_application(other.typed_directory().xdata_directory().applications()[0]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory.tuples_for_entity(other.tuples()[0].entity()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    let other_tuple = other.tuples()[0];
    let other_member = other_tuple.x().ok_or(io::Error::other("other x"))?;
    let other_entry = other
        .entry_for_member(other_tuple, other_member)
        .ok_or(io::Error::other("other entry"))?;
    assert_eq!(directory.tuple_for_entry(other_entry), None);
    assert_eq!(directory.entry_for_member(other_tuple, other_member), None);
    assert_copy::<DxfEntityXDataPointComponents>();
    assert_copy::<DxfEntityXDataPointMember>();
    assert_copy::<DxfEntityXDataPointTupleState>();
    assert_copy::<DxfEntityXDataPointTuple>();
    assert_send_sync::<DxfEntityXDataPointTupleDirectory>();
    assert!(size_of::<DxfEntityXDataPointMember>() <= 8);
    assert!(size_of::<DxfEntityXDataPointTuple>() <= 160);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_TUPLE_PAYLOAD"));
    Ok(())
}

fn assert_complete_directory(
    directory: &DxfEntityXDataPointTupleDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory.source_id(),
        directory.typed_directory().source_id()
    );
    assert_eq!(directory.tuples().len(), 6);
    let expected = [
        DxfEntityXDataPointKind::Point,
        DxfEntityXDataPointKind::WorldPosition,
        DxfEntityXDataPointKind::WorldDisplacement,
        DxfEntityXDataPointKind::WorldDirection,
        DxfEntityXDataPointKind::Point,
        DxfEntityXDataPointKind::WorldDirection,
    ];
    for (ordinal, (tuple, kind)) in directory.tuples().iter().copied().zip(expected).enumerate() {
        assert_eq!(tuple.ordinal(), ordinal as u64);
        assert_eq!(tuple.kind(), kind);
        assert_eq!(tuple.state(), DxfEntityXDataPointTupleState::Complete);
        assert_eq!(directory.tuple(tuple.ordinal()), Some(tuple));
    }
    let applications = directory.typed_directory().xdata_directory().applications();
    assert_eq!(applications.len(), 3);
    assert_eq!(directory.tuples_for_application(applications[0])?.len(), 4);
    assert_eq!(directory.tuples_for_application(applications[1])?.len(), 1);
    assert_eq!(directory.tuples_for_application(applications[2])?.len(), 1);
    let entities = directory
        .typed_directory()
        .xdata_directory()
        .entity_directory()
        .entities();
    assert_eq!(directory.tuples_for_entity(entities[0])?.len(), 5);
    assert_eq!(directory.tuples_for_entity(entities[1])?.len(), 1);
    Ok(())
}

fn evidence(
    directory: &DxfEntityXDataPointTupleDirectory,
) -> Result<Vec<TupleEvidence>, io::Error> {
    directory
        .tuples()
        .iter()
        .copied()
        .map(|tuple| {
            Ok((
                tuple.kind(),
                tuple.context(),
                number(directory, tuple, tuple.x())?,
                number(directory, tuple, tuple.y())?,
                number(directory, tuple, tuple.z())?,
            ))
        })
        .collect()
}

fn number(
    directory: &DxfEntityXDataPointTupleDirectory,
    tuple: DxfEntityXDataPointTuple,
    member: Option<DxfEntityXDataPointMember>,
) -> Result<Option<u64>, io::Error> {
    member
        .map(|member| {
            match directory
                .entry_for_member(tuple, member)
                .ok_or(io::Error::other("member entry"))?
                .value()
            {
                DxfEntityXDataValue::Double { value, .. } => Ok(value.to_f64().to_bits()),
                _ => Err(io::Error::other("expected double")),
            }
        })
        .transpose()
}

fn assert_components(tuple: DxfEntityXDataPointTuple, x: bool, y: bool, z: bool) {
    let components = tuple.components();
    assert_eq!(
        (components.has_x(), components.has_y(), components.has_z()),
        (x, y, z)
    );
    assert_eq!(components.is_complete(), x && y && z);
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Double(f64),
}

fn complete_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut groups = header(version);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP_A")),
        (1000, Value::Text(b"SECRET_TUPLE_PAYLOAD")),
    ]);
    append_tuple(&mut groups, 0, [1.0, 2.0, 3.0]);
    append_tuple(&mut groups, 1, [4.0, 5.0, 6.0]);
    append_tuple(&mut groups, 2, [7.0, 8.0, 9.0]);
    append_tuple(&mut groups, 3, [10.0, 11.0, 12.0]);
    groups.extend([(1001, Value::Text(b"APP_B"))]);
    append_tuple(&mut groups, 0, [13.0, 14.0, 15.0]);
    groups.extend([(0, Value::Text(b"LINE")), (1001, Value::Text(b"APP_C"))]);
    append_tuple(&mut groups, 3, [16.0, 17.0, 18.0]);
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn malformed_fixture(format: DxfRawDocumentFormat) -> io::Result<Vec<u8>> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1020, Value::Double(1.0)),
        (1030, Value::Double(2.0)),
        (1001, Value::Text(b"APP")),
        (1010, Value::Double(f64::NAN)),
        (1020, Value::Double(2.0)),
        (1030, Value::Double(3.0)),
        (1011, Value::Double(4.0)),
        (1031, Value::Double(6.0)),
        (1022, Value::Double(7.0)),
        (1040, Value::Double(8.0)),
        (1032, Value::Double(9.0)),
        (1013, Value::Double(10.0)),
        (1023, Value::Double(11.0)),
        (1023, Value::Double(12.0)),
        (1033, Value::Double(13.0)),
        (1010, Value::Double(20.0)),
        (1001, Value::Text(b"APP2")),
        (1020, Value::Double(21.0)),
        (1030, Value::Double(22.0)),
        (8, Value::Text(b"Layer0")),
        (1010, Value::Double(30.0)),
        (8, Value::Text(b"Layer1")),
        (1020, Value::Double(31.0)),
        (1030, Value::Double(32.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, DxfAcadVersion::Ac1032, &groups)
}

fn append_tuple(groups: &mut Vec<(i16, Value<'static>)>, suffix: i16, values: [f64; 3]) {
    groups.extend([
        (1010 + suffix, Value::Double(values[0])),
        (1020 + suffix, Value::Double(values[1])),
        (1030 + suffix, Value::Double(values[2])),
    ]);
}

fn header(version: DxfAcadVersion) -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
    ]
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> io::Result<Vec<u8>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[(i16, Value<'_>)]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        push_code(&mut bytes, version, *code)?;
        match value {
            Value::Text(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
    }
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else if version == DxfAcadVersion::Ac1009 {
        return Err(io::Error::other("group code"));
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
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
