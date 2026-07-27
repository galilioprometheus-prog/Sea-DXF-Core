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
        let ascii_ucs = ascii_directory
            .entry("ucsorg")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing ASCII UCSORG"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_ucs[2].raw_provenance(),
            b"-3",
        )?;
        let binary_ucs = binary_directory
            .entry("ucsorg")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing Binary UCSORG"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_ucs[2].raw_provenance(),
            &(-3_f64).to_le_bytes(),
        )?;
        let ascii_ortho = ascii_directory
            .entry("ucsorgtop")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing ASCII UCSORGTOP"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_ortho[2].raw_provenance(),
            b"-153",
        )?;
        let binary_ortho = binary_directory
            .entry("ucsorgtop")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing Binary UCSORGTOP"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_ortho[2].raw_provenance(),
            &(-153_f64).to_le_bytes(),
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
    let absent_directory = absent.header_numeric_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        absent_directory
            .entry("extmax")
            .map(|entry| entry.value().state()),
        Some(DxfSemanticValueState::Absent)
    );

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
fn coordinate_components_keep_independent_failure_evidence() -> Result<(), Box<dyn Error>> {
    let partial_bytes = ascii_document("AC1032", "9\n$EXTMAX\n10\n1\n21\n2\n");
    let partial_source = DxfMemorySource::new(&partial_bytes, DxfResourceProfile::Safe)?;
    let partial = open_ascii(&partial_source)?;
    let partial_directory = partial.header_numeric_directory(&DxfCancellationToken::default())?;
    let partial_value = partial_directory
        .entry("extmax")
        .and_then(|entry| entry.value().as_double3())
        .ok_or(io::Error::other("missing partial EXTMAX"))?;
    assert_eq!(
        partial_directory
            .entry("extmax")
            .map(|entry| entry.value().state()),
        Some(DxfSemanticValueState::Invalid)
    );
    assert_eq!(
        partial_value[0].value().copied().map(DxfDouble::to_bits),
        Some(1_f64.to_bits())
    );
    assert_eq!(
        partial_value[1].invalid_issue(),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(21)?))
    );
    assert_eq!(
        partial_value[2].invalid_issue(),
        Some(&DxfHeaderNumericIssue::MissingValue)
    );
    assert_raw_value(
        DxfRawDocumentView::from(&partial),
        partial_value[1].raw_provenance(),
        b"2",
    )?;
    assert_raw_value(
        DxfRawDocumentView::from(&partial),
        partial_value[2].raw_provenance(),
        b"$EXTMAX",
    )?;

    let extra_bytes = ascii_document("AC1032", "9\n$LIMMAX\n10\n1\n20\n2\n30\n3\n");
    let extra_source = DxfMemorySource::new(&extra_bytes, DxfResourceProfile::Safe)?;
    let extra = open_ascii(&extra_source)?;
    let extra_directory = extra.header_numeric_directory(&DxfCancellationToken::default())?;
    let extra_value = extra_directory
        .entry("limmax")
        .and_then(|entry| entry.value().as_double2())
        .ok_or(io::Error::other("missing extra LIMMAX"))?;
    let expected_issue = DxfHeaderNumericIssue::UnexpectedComponentCount {
        expected_count: NonZeroU64::new(2).ok_or(io::Error::other("zero expected count"))?,
        observed_count: 3,
    };
    assert_eq!(extra_value[0].invalid_issue(), Some(&expected_issue));
    assert_eq!(extra_value[1].invalid_issue(), Some(&expected_issue));
    assert_raw_value(
        DxfRawDocumentView::from(&extra),
        extra_value[0].raw_provenance(),
        b"3",
    )?;

    let duplicate_bytes = ascii_document(
        "AC1032",
        "9\n$EXTMIN\n10\n1\n20\n2\n30\n3\n9\n$EXTMIN\n10\n4\n20\n5\n30\n6\n",
    );
    let duplicate_source = DxfMemorySource::new(&duplicate_bytes, DxfResourceProfile::Safe)?;
    let duplicate = open_ascii(&duplicate_source)?;
    let duplicate_directory =
        duplicate.header_numeric_directory(&DxfCancellationToken::default())?;
    let duplicate_value = duplicate_directory
        .entry("extmin")
        .and_then(|entry| entry.value().as_double3())
        .ok_or(io::Error::other("missing duplicate EXTMIN"))?;
    let duplicate_issue = DxfHeaderNumericIssue::MultipleVariables {
        occurrence_count: NonZeroU64::new(2).ok_or(io::Error::other("zero duplicate count"))?,
    };
    for component in duplicate_value {
        assert_eq!(component.invalid_issue(), Some(&duplicate_issue));
    }
    assert_raw_value(
        DxfRawDocumentView::from(&duplicate),
        duplicate_value[0].raw_provenance(),
        b"4",
    )?;

    let empty_bytes = ascii_document("AC1032", "9\n$PINSBASE\n");
    let empty_source = DxfMemorySource::new(&empty_bytes, DxfResourceProfile::Safe)?;
    let empty = open_ascii(&empty_source)?;
    let empty_directory = empty.header_numeric_directory(&DxfCancellationToken::default())?;
    let empty_value = empty_directory
        .entry("pinsbase")
        .and_then(|entry| entry.value().as_double3())
        .ok_or(io::Error::other("missing empty PINSBASE"))?;
    for component in empty_value {
        assert_eq!(
            component.invalid_issue(),
            Some(&DxfHeaderNumericIssue::MissingValue)
        );
    }

    let ucs_bytes = ascii_document("AC1032", "9\n$UCSXDIR\n10\n1\n30\n0\n30\n0\n");
    let ucs_source = DxfMemorySource::new(&ucs_bytes, DxfResourceProfile::Safe)?;
    let ucs = open_ascii(&ucs_source)?;
    let ucs_directory = ucs.header_numeric_directory(&DxfCancellationToken::default())?;
    let ucs_value = ucs_directory
        .entry("ucsxdir")
        .and_then(|entry| entry.value().as_double3())
        .ok_or(io::Error::other("missing malformed UCSXDIR"))?;
    assert_eq!(
        ucs_value[0].value().copied().map(DxfDouble::to_f64),
        Some(1.0)
    );
    assert_eq!(
        ucs_value[1].invalid_issue(),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(30)?))
    );
    assert_eq!(
        ucs_value[2].value().copied().map(DxfDouble::to_f64),
        Some(0.0)
    );

    let ortho_bytes = ascii_document(
        "AC1032",
        "9\n$PUCSORGLEFT\n10\n1\n20\nnot-a-number\n30\n3\n",
    );
    let ortho_source = DxfMemorySource::new(&ortho_bytes, DxfResourceProfile::Safe)?;
    let ortho = open_ascii(&ortho_source)?;
    let ortho_directory = ortho.header_numeric_directory(&DxfCancellationToken::default())?;
    let ortho_value = ortho_directory
        .entry("pucsorgleft")
        .and_then(|entry| entry.value().as_double3())
        .ok_or(io::Error::other("missing malformed PUCSORGLEFT"))?;
    assert_eq!(
        ortho_value[0].value().copied().map(DxfDouble::to_f64),
        Some(1.0)
    );
    assert!(matches!(
        ortho_value[1].invalid_issue(),
        Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_eq!(
        ortho_value[2].value().copied().map(DxfDouble::to_f64),
        Some(3.0)
    );
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
    let expected: &[(u64, &str, &str, &[i16])] = &[
        (0, "acadmaintver", "$ACADMAINTVER", &[70]),
        (2, "angbase", "$ANGBASE", &[50]),
        (3, "angdir", "$ANGDIR", &[70]),
        (4, "attmode", "$ATTMODE", &[70]),
        (5, "aunits", "$AUNITS", &[70]),
        (6, "auprec", "$AUPREC", &[70]),
        (8, "extmax", "$EXTMAX", &[10, 20, 30]),
        (9, "extmin", "$EXTMIN", &[10, 20, 30]),
        (11, "insbase", "$INSBASE", &[10, 20, 30]),
        (12, "limmax", "$LIMMAX", &[10, 20]),
        (13, "limmin", "$LIMMIN", &[10, 20]),
        (14, "pextmax", "$PEXTMAX", &[10, 20, 30]),
        (15, "pextmin", "$PEXTMIN", &[10, 20, 30]),
        (16, "pinsbase", "$PINSBASE", &[10, 20, 30]),
        (17, "plimmax", "$PLIMMAX", &[10, 20]),
        (18, "plimmin", "$PLIMMIN", &[10, 20]),
        (19, "pucsorg", "$PUCSORG", &[10, 20, 30]),
        (20, "pucsxdir", "$PUCSXDIR", &[10, 20, 30]),
        (21, "pucsydir", "$PUCSYDIR", &[10, 20, 30]),
        (22, "ucsorg", "$UCSORG", &[10, 20, 30]),
        (23, "ucsxdir", "$UCSXDIR", &[10, 20, 30]),
        (24, "ucsydir", "$UCSYDIR", &[10, 20, 30]),
        (25, "pucsorgback", "$PUCSORGBACK", &[10, 20, 30]),
        (26, "pucsorgbottom", "$PUCSORGBOTTOM", &[10, 20, 30]),
        (27, "pucsorgfront", "$PUCSORGFRONT", &[10, 20, 30]),
        (28, "pucsorgleft", "$PUCSORGLEFT", &[10, 20, 30]),
        (29, "pucsorgright", "$PUCSORGRIGHT", &[10, 20, 30]),
        (30, "pucsorgtop", "$PUCSORGTOP", &[10, 20, 30]),
        (31, "ucsorgback", "$UCSORGBACK", &[10, 20, 30]),
        (32, "ucsorgbottom", "$UCSORGBOTTOM", &[10, 20, 30]),
        (33, "ucsorgfront", "$UCSORGFRONT", &[10, 20, 30]),
        (34, "ucsorgleft", "$UCSORGLEFT", &[10, 20, 30]),
        (35, "ucsorgright", "$UCSORGRIGHT", &[10, 20, 30]),
        (36, "ucsorgtop", "$UCSORGTOP", &[10, 20, 30]),
    ];
    assert_eq!(directory.entries().len(), expected.len());
    for (entry, &(ordinal, id, name, group_codes)) in directory.entries().iter().zip(expected) {
        assert_eq!(entry.schema_ordinal(), ordinal);
        assert_eq!(entry.schema_field_id(), id);
        assert_eq!(entry.dxf_name(), name);
        assert_eq!(entry.group_codes(), group_codes);
        assert_eq!(entry.value().state(), DxfSemanticValueState::Explicit);
        assert_eq!(
            entry.value().field_provenance().document_source_id(),
            source_id
        );
        assert_eq!(directory.entry_at_schema_ordinal(ordinal), Some(entry));
    }
    assert!(directory.entry("acadver").is_none());
    assert!(directory.entry_at_schema_ordinal(1).is_none());
    assert!(directory.entry_at_schema_ordinal(10).is_none());
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
    assert_double_components(
        directory
            .entry("extmax")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing EXTMAX tuple"))?,
        &[1.25, -2.5, 3.75],
    );
    assert_double_components(
        directory
            .entry("limmin")
            .and_then(|entry| entry.value().as_double2())
            .ok_or(io::Error::other("missing LIMMIN tuple"))?,
        &[-10.0, -20.0],
    );
    assert_double_components(
        directory
            .entry("plimmax")
            .and_then(|entry| entry.value().as_double2())
            .ok_or(io::Error::other("missing PLIMMAX tuple"))?,
        &[100.0, 200.0],
    );
    assert_double_components(
        directory
            .entry("pucsorg")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing PUCSORG tuple"))?,
        &[1.0, 2.0, 3.0],
    );
    assert_double_components(
        directory
            .entry("ucsorg")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing UCSORG tuple"))?,
        &[-1.0, -2.0, -3.0],
    );
    assert_double_components(
        directory
            .entry("pucsorgback")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing PUCSORGBACK tuple"))?,
        &[101.0, 102.0, 103.0],
    );
    assert_double_components(
        directory
            .entry("ucsorgtop")
            .and_then(|entry| entry.value().as_double3())
            .ok_or(io::Error::other("missing UCSORGTOP tuple"))?,
        &[-151.0, -152.0, -153.0],
    );
    let debug = format!("{directory:?}");
    assert!(debug.contains("numeric_field_count"));
    assert!(!debug.contains("$ANGBASE"));
    Ok(())
}

fn assert_double_components<const N: usize>(
    components: &[seacad_dxf_core::DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; N],
    expected: &[f64; N],
) {
    for (component, expected) in components.iter().zip(expected) {
        assert_eq!(
            component.value().copied().map(DxfDouble::to_bits),
            Some(expected.to_bits())
        );
    }
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
        "9\n$ACADMAINTVER\n70\n-32768\n9\n$ANGBASE\n50\n +5.000000000000000E-1 \n9\n$ANGDIR\n70\n+1\n9\n$ATTMODE\n70\n2\n9\n$AUNITS\n70\n0\n9\n$AUPREC\n70\n4\n9\n$EXTMAX\n10\n1.25\n20\n-2.5\n30\n3.75\n9\n$EXTMIN\n10\n-4.5\n20\n5.25\n30\n-6.75\n9\n$INSBASE\n10\n7\n20\n8\n30\n9\n9\n$LIMMAX\n10\n10\n20\n20\n9\n$LIMMIN\n10\n-10\n20\n-20\n9\n$PEXTMAX\n10\n11\n20\n22\n30\n33\n9\n$PEXTMIN\n10\n-11\n20\n-22\n30\n-33\n9\n$PINSBASE\n10\n0.125\n20\n0.25\n30\n0.5\n9\n$PLIMMAX\n10\n100\n20\n200\n9\n$PLIMMIN\n10\n-100\n20\n-200\n9\n$PUCSORG\n10\n1\n20\n2\n30\n3\n9\n$PUCSXDIR\n10\n1\n20\n0\n30\n0\n9\n$PUCSYDIR\n10\n0\n20\n1\n30\n0\n9\n$UCSORG\n10\n-1\n20\n-2\n30\n-3\n9\n$UCSXDIR\n10\n1\n20\n0\n30\n0\n9\n$UCSYDIR\n10\n0\n20\n1\n30\n0\n9\n$PUCSORGBACK\n10\n101\n20\n102\n30\n103\n9\n$PUCSORGBOTTOM\n10\n111\n20\n112\n30\n113\n9\n$PUCSORGFRONT\n10\n121\n20\n122\n30\n123\n9\n$PUCSORGLEFT\n10\n131\n20\n132\n30\n133\n9\n$PUCSORGRIGHT\n10\n141\n20\n142\n30\n143\n9\n$PUCSORGTOP\n10\n151\n20\n152\n30\n153\n9\n$UCSORGBACK\n10\n-101\n20\n-102\n30\n-103\n9\n$UCSORGBOTTOM\n10\n-111\n20\n-112\n30\n-113\n9\n$UCSORGFRONT\n10\n-121\n20\n-122\n30\n-123\n9\n$UCSORGLEFT\n10\n-131\n20\n-132\n30\n-133\n9\n$UCSORGRIGHT\n10\n-141\n20\n-142\n30\n-143\n9\n$UCSORGTOP\n10\n-151\n20\n-152\n30\n-153\n",
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
    for (name, values) in [
        (b"$EXTMAX".as_slice(), &[1.25, -2.5, 3.75][..]),
        (b"$EXTMIN".as_slice(), &[-4.5, 5.25, -6.75][..]),
        (b"$INSBASE".as_slice(), &[7.0, 8.0, 9.0][..]),
        (b"$LIMMAX".as_slice(), &[10.0, 20.0][..]),
        (b"$LIMMIN".as_slice(), &[-10.0, -20.0][..]),
        (b"$PEXTMAX".as_slice(), &[11.0, 22.0, 33.0][..]),
        (b"$PEXTMIN".as_slice(), &[-11.0, -22.0, -33.0][..]),
        (b"$PINSBASE".as_slice(), &[0.125, 0.25, 0.5][..]),
        (b"$PLIMMAX".as_slice(), &[100.0, 200.0][..]),
        (b"$PLIMMIN".as_slice(), &[-100.0, -200.0][..]),
        (b"$PUCSORG".as_slice(), &[1.0, 2.0, 3.0][..]),
        (b"$PUCSXDIR".as_slice(), &[1.0, 0.0, 0.0][..]),
        (b"$PUCSYDIR".as_slice(), &[0.0, 1.0, 0.0][..]),
        (b"$UCSORG".as_slice(), &[-1.0, -2.0, -3.0][..]),
        (b"$UCSXDIR".as_slice(), &[1.0, 0.0, 0.0][..]),
        (b"$UCSYDIR".as_slice(), &[0.0, 1.0, 0.0][..]),
        (b"$PUCSORGBACK".as_slice(), &[101.0, 102.0, 103.0][..]),
        (b"$PUCSORGBOTTOM".as_slice(), &[111.0, 112.0, 113.0][..]),
        (b"$PUCSORGFRONT".as_slice(), &[121.0, 122.0, 123.0][..]),
        (b"$PUCSORGLEFT".as_slice(), &[131.0, 132.0, 133.0][..]),
        (b"$PUCSORGRIGHT".as_slice(), &[141.0, 142.0, 143.0][..]),
        (b"$PUCSORGTOP".as_slice(), &[151.0, 152.0, 153.0][..]),
        (b"$UCSORGBACK".as_slice(), &[-101.0, -102.0, -103.0][..]),
        (b"$UCSORGBOTTOM".as_slice(), &[-111.0, -112.0, -113.0][..]),
        (b"$UCSORGFRONT".as_slice(), &[-121.0, -122.0, -123.0][..]),
        (b"$UCSORGLEFT".as_slice(), &[-131.0, -132.0, -133.0][..]),
        (b"$UCSORGRIGHT".as_slice(), &[-141.0, -142.0, -143.0][..]),
        (b"$UCSORGTOP".as_slice(), &[-151.0, -152.0, -153.0][..]),
    ] {
        push_binary_double_tuple(&mut bytes, version, name, values)?;
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

fn push_binary_double_tuple(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    values: &[f64],
) -> Result<(), io::Error> {
    push_binary_string(bytes, version, 9, name)?;
    for (group_code, value) in [10_i16, 20, 30].into_iter().zip(values.iter().copied()) {
        push_binary_double_bits(bytes, version, group_code, value.to_bits())?;
    }
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
