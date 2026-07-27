use std::{error::Error, io, num::NonZeroU64};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfErrorCode, DxfGroupCode,
    DxfHeaderNumericDirectory, DxfHeaderNumericEntry, DxfHeaderNumericIssue, DxfHeaderNumericValue,
    DxfHeaderNumericView, DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_numeric_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_standard_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.header_numeric_directory(&DxfCancellationToken::default())?;
        assert_standard_directory(&ascii_directory, ascii.source_id())?;
        let ascii_view = ascii.header_numeric_view(&DxfCancellationToken::default())?;
        assert_standard_view(&ascii_view, ascii.source_id())?;

        let binary_bytes = binary_standard_fixture(version, 0.5_f64.to_bits())?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.header_numeric_directory(&DxfCancellationToken::default())?;
        assert_standard_directory(&binary_directory, binary.source_id())?;
        let binary_view = binary.header_numeric_view(&DxfCancellationToken::default())?;
        assert_standard_view(&binary_view, binary.source_id())?;

        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_view.angle_base().raw_provenance(),
            b" +5.000000000000000E-1 ",
        )?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_view.angle_base().raw_provenance(),
            &0.5_f64.to_le_bytes(),
        )?;
    }
    Ok(())
}

#[test]
fn absent_wrong_missing_multiple_and_duplicate_are_distinct() -> Result<(), Box<dyn Error>> {
    let absent_bytes = ascii_document("AC1032", "");
    let absent_source = DxfMemorySource::new(&absent_bytes, DxfResourceProfile::Safe)?;
    let absent = open_ascii(&absent_source)?;
    let absent_view = absent.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        absent_view.angle_direction().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(absent_view.angle_direction().raw_provenance(), None);

    let wrong_bytes = ascii_document("AC1032", "9\n$ANGDIR\n71\n1\n");
    let wrong_source = DxfMemorySource::new(&wrong_bytes, DxfResourceProfile::Safe)?;
    let wrong = open_ascii(&wrong_source)?;
    let wrong_directory = wrong.header_numeric_directory(&DxfCancellationToken::default())?;
    let wrong_entry = wrong_directory
        .entry("angdir")
        .ok_or(io::Error::other("missing numeric directory entry"))?;
    assert!(wrong_entry.value().as_double().is_none());
    assert_eq!(
        wrong_entry
            .value()
            .as_int16()
            .and_then(|value| value.invalid_issue()),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(71)?))
    );
    let wrong_view = wrong.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        wrong_view.angle_direction().invalid_issue(),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(71)?))
    );
    assert_raw_value(
        DxfRawDocumentView::from(&wrong),
        wrong_view.angle_direction().raw_provenance(),
        b"1",
    )?;

    let missing_bytes = ascii_document("AC1032", "9\n$ANGDIR\n");
    let missing_source = DxfMemorySource::new(&missing_bytes, DxfResourceProfile::Safe)?;
    let missing = open_ascii(&missing_source)?;
    let missing_view = missing.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        missing_view.angle_direction().invalid_issue(),
        Some(&DxfHeaderNumericIssue::MissingValue)
    );
    assert_raw_value(
        DxfRawDocumentView::from(&missing),
        missing_view.angle_direction().raw_provenance(),
        b"$ANGDIR",
    )?;

    let multiple_bytes = ascii_document("AC1032", "9\n$ANGDIR\n70\n1\n70\n2\n");
    let multiple_source = DxfMemorySource::new(&multiple_bytes, DxfResourceProfile::Safe)?;
    let multiple = open_ascii(&multiple_source)?;
    let multiple_view = multiple.header_numeric_view(&DxfCancellationToken::default())?;
    assert_count_issue(multiple_view.angle_direction().invalid_issue(), false, 2)?;
    assert_raw_value(
        DxfRawDocumentView::from(&multiple),
        multiple_view.angle_direction().raw_provenance(),
        b"2",
    )?;

    let duplicate_bytes = ascii_document("AC1032", "9\n$ANGDIR\n70\n1\n9\n$ANGDIR\n70\n2\n");
    let duplicate_source = DxfMemorySource::new(&duplicate_bytes, DxfResourceProfile::Safe)?;
    let duplicate = open_ascii(&duplicate_source)?;
    let duplicate_view = duplicate.header_numeric_view(&DxfCancellationToken::default())?;
    assert_count_issue(duplicate_view.angle_direction().invalid_issue(), true, 2)?;
    assert_raw_value(
        DxfRawDocumentView::from(&duplicate),
        duplicate_view.angle_direction().raw_provenance(),
        b"2",
    )?;
    Ok(())
}

#[test]
fn ascii_grammar_is_locale_free_bounded_and_typed() -> Result<(), Box<dyn Error>> {
    for (raw, expected) in [
        ("", DxfAsciiNumericIssue::Empty),
        (".", DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }),
        (
            "1e",
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 },
        ),
        (
            "NaN",
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 },
        ),
        (
            "inf",
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 },
        ),
        ("1e9999", DxfAsciiNumericIssue::OutOfRange),
        ("1e-9999", DxfAsciiNumericIssue::OutOfRange),
    ] {
        let bytes = ascii_document("AC1032", &format!("9\n$ANGBASE\n50\n{raw}\n"));
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let view = document.header_numeric_view(&DxfCancellationToken::default())?;
        assert_eq!(
            view.angle_base().invalid_issue(),
            Some(&DxfHeaderNumericIssue::InvalidAsciiNumber(expected))
        );
    }

    let long = "0".repeat(8_193);
    let bytes = ascii_document("AC1032", &format!("9\n$ANGBASE\n50\n{long}\n"));
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = document.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        view.angle_base().value().copied().map(DxfDouble::to_bits),
        Some(0_f64.to_bits())
    );

    for (raw, expected) in [
        ("32768", DxfAsciiNumericIssue::OutOfRange),
        ("+", DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }),
        (
            "12x",
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 },
        ),
    ] {
        let bytes = ascii_document("AC1032", &format!("9\n$ANGDIR\n70\n{raw}\n"));
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let view = document.header_numeric_view(&DxfCancellationToken::default())?;
        assert_eq!(
            view.angle_direction().invalid_issue(),
            Some(&DxfHeaderNumericIssue::InvalidAsciiNumber(expected))
        );
    }

    let boundary_bytes = ascii_document(
        "AC1032",
        "9\n$ANGBASE\n50\n .5 \n9\n$ANGDIR\n70\n -32768 \n",
    );
    let boundary_source = DxfMemorySource::new(&boundary_bytes, DxfResourceProfile::Safe)?;
    let boundary = open_ascii(&boundary_source)?;
    let boundary_view = boundary.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        boundary_view
            .angle_base()
            .value()
            .copied()
            .map(DxfDouble::to_bits),
        Some(0.5_f64.to_bits())
    );
    assert_eq!(boundary_view.angle_direction().value(), Some(&i16::MIN));

    let negative_zero_bytes = ascii_document("AC1032", "9\n$ANGBASE\n50\n-0.0\n");
    let negative_zero_source =
        DxfMemorySource::new(&negative_zero_bytes, DxfResourceProfile::Safe)?;
    let negative_zero = open_ascii(&negative_zero_source)?;
    let negative_zero_view = negative_zero.header_numeric_view(&DxfCancellationToken::default())?;
    assert_eq!(
        negative_zero_view
            .angle_base()
            .value()
            .copied()
            .map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    Ok(())
}

#[test]
fn binary_preserves_ieee_bits_and_signed_boundaries() -> Result<(), Box<dyn Error>> {
    let nan_bits = 0x7ff8_0000_0000_0042_u64;
    let bytes = binary_boundary_fixture(DxfAcadVersion::Ac1032, nan_bits)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let view = document.header_numeric_view(&DxfCancellationToken::default())?;
    let angle = view
        .angle_base()
        .value()
        .copied()
        .ok_or(io::Error::other("missing angle"))?;
    assert_eq!(angle.to_bits(), nan_bits);
    assert!(angle.to_f64().is_nan());
    assert!(!angle.is_finite());
    assert_eq!(view.acad_maintenance_version().value(), Some(&i16::MIN));
    assert_eq!(view.angle_direction().value(), Some(&i16::MAX));
    Ok(())
}

#[test]
fn cancellation_and_public_metadata_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_standard_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    let error = document
        .header_numeric_view(&cancellation)
        .err()
        .ok_or(io::Error::other("cancelled semantic read passed"))?;
    assert_eq!(error.code(), DxfErrorCode::CANCELLED);
    let directory_error = document
        .header_numeric_directory(&cancellation)
        .err()
        .ok_or(io::Error::other("cancelled directory read passed"))?;
    assert_eq!(directory_error.code(), DxfErrorCode::CANCELLED);

    assert_copy::<DxfDouble>();
    assert_copy::<DxfAsciiNumericIssue>();
    assert_copy::<DxfHeaderNumericIssue>();
    assert_copy::<DxfHeaderNumericValue>();
    assert_copy::<DxfHeaderNumericEntry>();
    assert_copy::<DxfHeaderNumericView>();
    assert_send_sync::<DxfHeaderNumericDirectory>();
    assert_send_sync::<DxfHeaderNumericView>();
    Ok(())
}

fn assert_standard_directory(
    directory: &DxfHeaderNumericDirectory,
    source_id: seacad_dxf_core::DxfSourceId,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.schema_version(), "dxf.v1");
    assert_eq!(directory.source_id(), source_id);
    let expected = [
        (0_u64, "acadmaintver", "$ACADMAINTVER"),
        (2, "angbase", "$ANGBASE"),
        (3, "angdir", "$ANGDIR"),
        (4, "attmode", "$ATTMODE"),
        (5, "aunits", "$AUNITS"),
        (6, "auprec", "$AUPREC"),
    ];
    assert_eq!(directory.entries().len(), expected.len());
    for (entry, (ordinal, id, name)) in directory.entries().iter().zip(expected) {
        assert_eq!(entry.schema_ordinal(), ordinal);
        assert_eq!(entry.schema_field_id(), id);
        assert_eq!(entry.dxf_name(), name);
        assert_eq!(entry.value().state(), DxfSemanticValueState::Explicit);
        assert_eq!(
            entry.value().field_provenance().document_source_id(),
            source_id
        );
        assert_eq!(directory.entry_at_schema_ordinal(ordinal), Some(entry));
    }
    assert!(directory.entry("acadver").is_none());
    assert!(directory.entry_at_schema_ordinal(1).is_none());
    assert_eq!(
        directory
            .entry("angbase")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(0.5_f64.to_bits())
    );
    assert_eq!(
        directory
            .entry("angdir")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&1)
    );
    let debug = format!("{directory:?}");
    assert!(debug.contains("numeric_field_count"));
    assert!(!debug.contains("$ANGBASE"));
    Ok(())
}

fn assert_standard_view(
    view: &DxfHeaderNumericView,
    source_id: seacad_dxf_core::DxfSourceId,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(view.source_id(), source_id);
    assert_eq!(view.acad_maintenance_version().value(), Some(&i16::MIN));
    assert_eq!(
        view.angle_base().value().copied().map(DxfDouble::to_bits),
        Some(0.5_f64.to_bits())
    );
    assert_eq!(view.angle_direction().value(), Some(&1));
    assert_eq!(view.attribute_mode().value(), Some(&2));
    assert_eq!(view.angular_units().value(), Some(&0));
    assert_eq!(view.angular_precision().value(), Some(&4));
    for (field, expected_id) in [
        (
            view.acad_maintenance_version().field_provenance(),
            "acadmaintver",
        ),
        (view.angle_base().field_provenance(), "angbase"),
        (view.angle_direction().field_provenance(), "angdir"),
        (view.attribute_mode().field_provenance(), "attmode"),
        (view.angular_units().field_provenance(), "aunits"),
        (view.angular_precision().field_provenance(), "auprec"),
    ] {
        assert_eq!(field.document_source_id(), source_id);
        assert_eq!(field.schema_namespace(), "header");
        assert_eq!(field.schema_field_id(), expected_id);
    }
    if [
        view.acad_maintenance_version().state(),
        view.angle_direction().state(),
        view.attribute_mode().state(),
        view.angular_units().state(),
        view.angular_precision().state(),
    ]
    .iter()
    .any(|state| *state != DxfSemanticValueState::Explicit)
        || view.angle_base().state() != DxfSemanticValueState::Explicit
    {
        return Err(io::Error::other("numeric value was not explicit").into());
    }
    Ok(())
}

fn assert_count_issue(
    issue: Option<&DxfHeaderNumericIssue>,
    variables: bool,
    expected: u64,
) -> Result<(), Box<dyn Error>> {
    let count = match issue {
        Some(DxfHeaderNumericIssue::MultipleVariables { occurrence_count }) if variables => {
            *occurrence_count
        }
        Some(DxfHeaderNumericIssue::MultipleValueGroups { group_count }) if !variables => {
            *group_count
        }
        _ => return Err(io::Error::other("unexpected count issue").into()),
    };
    assert_eq!(
        count,
        NonZeroU64::new(expected).ok_or(io::Error::other("zero count"))?
    );
    Ok(())
}

fn assert_raw_value(
    document: DxfRawDocumentView<'_>,
    raw: Option<seacad_dxf_core::DxfRawValueProvenance>,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    let raw = raw.ok_or(io::Error::other("missing raw provenance"))?;
    let mut observed = vec![0_u8; expected.len()];
    document.read_span(raw.value_span(), &mut observed)?;
    assert_eq!(observed, expected);
    Ok(())
}

fn ascii_standard_fixture(version: &str) -> Vec<u8> {
    ascii_document(
        version,
        "9\n$ACADMAINTVER\n70\n-32768\n9\n$ANGBASE\n50\n +5.000000000000000E-1 \n9\n$ANGDIR\n70\n+1\n9\n$ATTMODE\n70\n2\n9\n$AUNITS\n70\n0\n9\n$AUPREC\n70\n4\n",
    )
}

fn ascii_document(version: &str, body: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n{body}0\nENDSEC\n0\nEOF\n")
        .into_bytes()
}

fn binary_standard_fixture(version: DxfAcadVersion, angle_bits: u64) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_prefix(version)?;
    push_binary_string(&mut bytes, version, 9, b"$ACADMAINTVER")?;
    push_binary_i16(&mut bytes, version, 70, i16::MIN)?;
    push_binary_string(&mut bytes, version, 9, b"$ANGBASE")?;
    push_binary_double_bits(&mut bytes, version, 50, angle_bits)?;
    for (name, value) in [
        (b"$ANGDIR".as_slice(), 1_i16),
        (b"$ATTMODE", 2),
        (b"$AUNITS", 0),
        (b"$AUPREC", 4),
    ] {
        push_binary_string(&mut bytes, version, 9, name)?;
        push_binary_i16(&mut bytes, version, 70, value)?;
    }
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_boundary_fixture(version: DxfAcadVersion, angle_bits: u64) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_prefix(version)?;
    push_binary_string(&mut bytes, version, 9, b"$ACADMAINTVER")?;
    push_binary_i16(&mut bytes, version, 70, i16::MIN)?;
    push_binary_string(&mut bytes, version, 9, b"$ANGBASE")?;
    push_binary_double_bits(&mut bytes, version, 50, angle_bits)?;
    push_binary_string(&mut bytes, version, 9, b"$ANGDIR")?;
    push_binary_i16(&mut bytes, version, 70, i16::MAX)?;
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (group_code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
    ] {
        push_binary_string(&mut bytes, version, group_code, value)?;
    }
    Ok(bytes)
}

fn binary_suffix(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> Result<(), io::Error> {
    for (group_code, value) in [(0_i16, b"ENDSEC".as_slice()), (0, b"EOF")] {
        push_binary_string(bytes, version, group_code, value)?;
    }
    Ok(())
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    push_binary_group_code(bytes, version, group_code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_binary_i16(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: i16,
) -> Result<(), io::Error> {
    push_binary_group_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_binary_double_bits(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    bits: u64,
) -> Result<(), io::Error> {
    push_binary_group_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
    Ok(())
}

fn push_binary_group_code(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code width"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    Ok(())
}

fn open_ascii<'a>(
    source: &'a DxfMemorySource<'_>,
) -> Result<DxfAsciiRawDocument<'a>, seacad_dxf_core::DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(
    source: &'a DxfMemorySource<'_>,
) -> Result<DxfBinaryRawDocument<'a>, seacad_dxf_core::DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn code(value: i16) -> Result<DxfGroupCode, io::Error> {
    DxfGroupCode::new(value).ok_or(io::Error::other("group code"))
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
