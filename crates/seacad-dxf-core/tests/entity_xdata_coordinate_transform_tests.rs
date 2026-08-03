use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataCoordinateTransformDirectory, DxfEntityXDataCoordinateTransformInput,
    DxfEntityXDataCoordinateTransformIssue, DxfEntityXDataPointComponent, DxfEntityXDataPointKind,
    DxfEntityXDataTransformedPointEntry, DxfEntityXDataTransformedPointIssue,
    DxfEntityXDataTransformedPointState, DxfError, DxfMemorySource, DxfRawDocumentFormat,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_matches_autocad_transform_channels() -> Result<(), Box<dyn Error>> {
    let transform = oracle_transform().map_err(transform_error)?;
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = complete_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?
            .entity_xdata_coordinate_transform_directory(transform, &token())?;

        let binary_bytes = complete_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?
            .entity_xdata_coordinate_transform_directory(transform, &token())?;

        assert_oracle_results(&ascii)?;
        assert_oracle_results(&binary)?;
        assert_eq!(evidence(&ascii)?, evidence(&binary)?);
    }
    Ok(())
}

#[test]
fn partial_invalid_and_overflow_tuples_fail_closed() -> Result<(), Box<dyn Error>> {
    let transform = DxfEntityXDataCoordinateTransform::uniform_scale(
        d3([0.0; 3]),
        DxfDouble::from_f64(f64::MAX),
    )
    .map_err(transform_error)?;
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = failure_fixture(format)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => open_ascii(&source)?
                .entity_xdata_coordinate_transform_directory(transform, &token())?,
            DxfRawDocumentFormat::Binary => open_binary(&source)?
                .entity_xdata_coordinate_transform_directory(transform, &token())?,
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(directory.entries().len(), 3);
        assert!(matches!(
            directory.entries()[0].state(),
            DxfEntityXDataTransformedPointState::Unavailable(
                DxfEntityXDataTransformedPointIssue::InvalidComponent {
                    component: DxfEntityXDataPointComponent::X,
                    ..
                }
            )
        ));
        assert!(matches!(
            directory.entries()[1].state(),
            DxfEntityXDataTransformedPointState::Unavailable(
                DxfEntityXDataTransformedPointIssue::PartialTuple(components)
            ) if components.has_x() && components.has_y() && !components.has_z()
        ));
        assert!(matches!(
            directory.entries()[2].state(),
            DxfEntityXDataTransformedPointState::Unavailable(
                DxfEntityXDataTransformedPointIssue::NonFiniteDerivedPoint
            )
        ));
    }
    Ok(())
}

#[test]
fn transforms_and_directories_are_validated_bounded_and_source_bound() -> Result<(), Box<dyn Error>>
{
    assert!(matches!(
        DxfEntityXDataCoordinateTransform::translation(d3([f64::NAN, 0.0, 0.0])),
        Err(DxfEntityXDataCoordinateTransformIssue::NonFiniteInput(
            DxfEntityXDataCoordinateTransformInput::Translation
        ))
    ));
    assert!(matches!(
        DxfEntityXDataCoordinateTransform::uniform_scale(d3([0.0; 3]), DxfDouble::from_f64(0.0)),
        Err(DxfEntityXDataCoordinateTransformIssue::ZeroScaleFactor)
    ));
    assert!(matches!(
        DxfEntityXDataCoordinateTransform::rotation(
            d3([0.0; 3]),
            d3([0.0; 3]),
            DxfDouble::from_f64(90.0)
        ),
        Err(DxfEntityXDataCoordinateTransformIssue::ZeroLengthRotationAxis)
    ));
    assert!(matches!(
        DxfEntityXDataCoordinateTransform::mirror(d3([0.0; 3]), d3([0.0; 3])),
        Err(DxfEntityXDataCoordinateTransformIssue::ZeroLengthMirrorNormal)
    ));
    let huge = DxfEntityXDataCoordinateTransform::translation(d3([f64::MAX, 0.0, 0.0]))
        .map_err(transform_error)?;
    assert!(matches!(
        huge.then(huge),
        Err(DxfEntityXDataCoordinateTransformIssue::NonFiniteDerivedTransform)
    ));

    let bytes = complete_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_xdata_coordinate_transform_directory(
            DxfEntityXDataCoordinateTransform::identity(),
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = document.entity_xdata_coordinate_transform_directory(
        DxfEntityXDataCoordinateTransform::identity(),
        &token(),
    )?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(
        directory.entries().len(),
        directory.point_tuple_directory().tuples().len()
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(directory.entry(entry.tuple().ordinal()), Some(entry));
        assert_eq!(directory.entry_for_tuple(entry.tuple()), Some(entry));
    }

    let other_bytes = complete_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?.entity_xdata_coordinate_transform_directory(
        DxfEntityXDataCoordinateTransform::identity(),
        &token(),
    )?;
    assert_eq!(directory.entry_for_tuple(other.entries()[0].tuple()), None);
    assert!(!format!("{directory:?}").contains("SECRET_TRANSFORM_PAYLOAD"));
    assert_copy::<DxfEntityXDataCoordinateTransform>();
    assert_copy::<DxfEntityXDataTransformedPointEntry>();
    assert_send_sync::<DxfEntityXDataCoordinateTransformDirectory>();
    assert!(size_of::<DxfEntityXDataCoordinateTransform>() <= 256);
    assert!(size_of::<DxfEntityXDataTransformedPointEntry>() <= 224);
    Ok(())
}

fn oracle_transform()
-> Result<DxfEntityXDataCoordinateTransform, DxfEntityXDataCoordinateTransformIssue> {
    let scale = DxfEntityXDataCoordinateTransform::uniform_scale(
        d3([10.0, 0.0, 0.0]),
        DxfDouble::from_f64(2.0),
    )?;
    let rotate = DxfEntityXDataCoordinateTransform::rotation(
        d3([0.0; 3]),
        d3([0.0, 0.0, 1.0]),
        DxfDouble::from_f64(90.0),
    )?;
    let translate = DxfEntityXDataCoordinateTransform::translation(d3([10.0, 20.0, 30.0]))?;
    let mirror = DxfEntityXDataCoordinateTransform::mirror(d3([0.0; 3]), d3([1.0, 0.0, 0.0]))?;
    scale.then(rotate)?.then(translate)?.then(mirror)
}

fn assert_oracle_results(
    directory: &DxfEntityXDataCoordinateTransformDirectory,
) -> Result<(), io::Error> {
    assert_eq!(directory.entries().len(), 4);
    let expected = [
        (DxfEntityXDataPointKind::Point, [1.0, 2.0, 3.0]),
        (DxfEntityXDataPointKind::WorldPosition, [-6.0, 12.0, 36.0]),
        (DxfEntityXDataPointKind::WorldDisplacement, [4.0, 2.0, 6.0]),
        (DxfEntityXDataPointKind::WorldDirection, [2.0, 1.0, 3.0]),
    ];
    for (entry, (kind, transformed)) in directory.entries().iter().copied().zip(expected) {
        assert_eq!(entry.tuple().kind(), kind);
        let DxfEntityXDataTransformedPointState::Available {
            original,
            transformed: actual,
        } = entry.state()
        else {
            return Err(io::Error::other("available transform"));
        };
        assert_vector(original, [1.0, 2.0, 3.0]);
        assert_vector(actual, transformed);
    }
    Ok(())
}

fn evidence(
    directory: &DxfEntityXDataCoordinateTransformDirectory,
) -> Result<Vec<(DxfEntityXDataPointKind, [u64; 3])>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| match entry.state() {
            DxfEntityXDataTransformedPointState::Available { transformed, .. } => Ok((
                entry.tuple().kind(),
                transformed.map(|value| value.to_f64().to_bits()),
            )),
            _ => Err(io::Error::other("available transform")),
        })
        .collect()
}

fn assert_vector(actual: [DxfDouble; 3], expected: [f64; 3]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!((actual.to_f64() - expected).abs() <= 1.0e-12);
    }
}

fn d3(values: [f64; 3]) -> [DxfDouble; 3] {
    values.map(DxfDouble::from_f64)
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
        (1001, Value::Text(b"APP")),
        (1000, Value::Text(b"SECRET_TRANSFORM_PAYLOAD")),
    ]);
    for suffix in 0..=3 {
        append_tuple(&mut groups, suffix, [1.0, 2.0, 3.0]);
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn failure_fixture(format: DxfRawDocumentFormat) -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut groups = header(version);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP")),
    ]);
    append_tuple(&mut groups, 1, [f64::NAN, 2.0, 3.0]);
    groups.extend([
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP")),
        (1012, Value::Double(1.0)),
        (1022, Value::Double(2.0)),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP")),
    ]);
    append_tuple(&mut groups, 1, [f64::MAX, 1.0, 1.0]);
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
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

fn transform_error(issue: DxfEntityXDataCoordinateTransformIssue) -> io::Error {
    io::Error::other(format!("{issue:?}"))
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
