use std::{error::Error, io, num::NonZeroU64};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfDayParts, DxfDouble, DxfElapsedDays,
    DxfErrorCode, DxfGroupCode, DxfHeaderNumericDirectory, DxfHeaderNumericEntry,
    DxfHeaderNumericIssue, DxfHeaderNumericValue, DxfHeaderNumericView, DxfJulianDate,
    DxfMemorySource, DxfRawDocumentView, DxfRawValueProvenance, DxfReadOptions, DxfResourceProfile,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSemanticValueState, DxfSourceId,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_numeric_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_standard_fixture(version.code())?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.header_numeric_directory(&DxfCancellationToken::default())?;
        assert_standard_directory(&ascii_directory, ascii.source_id(), version)?;
        let ascii_view = ascii.header_numeric_view(&DxfCancellationToken::default())?;
        assert_standard_view(&ascii_view, ascii.source_id())?;

        let binary_bytes = binary_standard_fixture(version, 0.5_f64.to_bits())?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.header_numeric_directory(&DxfCancellationToken::default())?;
        assert_standard_directory(&binary_directory, binary.source_id(), version)?;
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
        let ascii_ltscale = ascii_directory
            .entry("ltscale")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing ASCII LTSCALE"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_ltscale.raw_provenance(),
            b"2.5",
        )?;
        let binary_color = binary_directory
            .entry("cecolor")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary CECOLOR"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_color.raw_provenance(),
            &256_i16.to_le_bytes(),
        )?;
        let ascii_pelevation = ascii_directory
            .entry("pelevation")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing ASCII PELEVATION"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_pelevation.raw_provenance(),
            b"-7.25",
        )?;
        let binary_pdmode = binary_directory
            .entry("pdmode")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary PDMODE"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_pdmode.raw_provenance(),
            &34_i16.to_le_bytes(),
        )?;
        let ascii_shadow = ascii_directory
            .entry("shadowplanelocation")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing ASCII SHADOWPLANELOCATION"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_shadow.raw_provenance(),
            b"-100.25",
        )?;
        let binary_ortho_view = binary_directory
            .entry("pucsorthoview")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary PUCSORTHOVIEW"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_ortho_view.raw_provenance(),
            &6_i16.to_le_bytes(),
        )?;
        let ascii_thickness = ascii_directory
            .entry("thickness")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing ASCII THICKNESS"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_thickness.raw_provenance(),
            b"-1.25",
        )?;
        let binary_tree_depth = binary_directory
            .entry("treedepth")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary TREEDEPTH"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_tree_depth.raw_provenance(),
            &10_i16.to_le_bytes(),
        )?;
        let ascii_created = ascii_directory
            .entry("tdcreate")
            .and_then(|entry| entry.value().as_julian_date())
            .ok_or(io::Error::other("missing ASCII TDCREATE"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_created.raw_provenance(),
            b"2451544.91568287",
        )?;
        let binary_timer = binary_directory
            .entry("tdusrtimer")
            .and_then(|entry| entry.value().as_elapsed_days())
            .ok_or(io::Error::other("missing Binary TDUSRTIMER"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_timer.raw_provenance(),
            &0.5_f64.to_le_bytes(),
        )?;
        let binary_insunits = binary_directory
            .entry("insunits")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary INSUNITS"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_insunits.raw_provenance(),
            &6_i16.to_le_bytes(),
        )?;
        let ascii_dimaltf = ascii_directory
            .entry("dimaltf")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing ASCII DIMALTF"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_dimaltf.raw_provenance(),
            b"25.4",
        )?;
        let binary_dimtxt = binary_directory
            .entry("dimtxt")
            .and_then(|entry| entry.value().as_double())
            .ok_or(io::Error::other("missing Binary DIMTXT"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_dimtxt.raw_provenance(),
            &2.25_f64.to_le_bytes(),
        )?;
        let ascii_dimadec = ascii_directory
            .entry("dimadec")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing ASCII DIMADEC"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&ascii),
            ascii_dimadec.raw_provenance(),
            b"3",
        )?;
        let binary_dimdsep = binary_directory
            .entry("dimdsep")
            .and_then(|entry| entry.value().as_int16())
            .ok_or(io::Error::other("missing Binary DIMDSEP"))?;
        assert_raw_value(
            DxfRawDocumentView::from(&binary),
            binary_dimdsep.raw_provenance(),
            &44_i16.to_le_bytes(),
        )?;
        if version != DxfAcadVersion::Ac1009 {
            let ascii_endcaps = ascii_directory
                .entry("endcaps")
                .and_then(|entry| entry.value().as_int16())
                .ok_or(io::Error::other("missing ASCII ENDCAPS"))?;
            assert_raw_value(
                DxfRawDocumentView::from(&ascii),
                ascii_endcaps.raw_provenance(),
                b"2",
            )?;
            let binary_extnames = binary_directory
                .entry("extnames")
                .and_then(|entry| entry.value().as_boolean())
                .ok_or(io::Error::other("missing Binary EXTNAMES"))?;
            assert_raw_value(
                DxfRawDocumentView::from(&binary),
                binary_extnames.raw_provenance(),
                &[1],
            )?;
            let ascii_celweight = ascii_directory
                .entry("celweight")
                .and_then(|entry| entry.value().as_int16())
                .ok_or(io::Error::other("missing ASCII CELWEIGHT"))?;
            assert_raw_value(
                DxfRawDocumentView::from(&ascii),
                ascii_celweight.raw_provenance(),
                b"-1",
            )?;
            let binary_xclipframe = binary_directory
                .entry("xclipframe")
                .and_then(|entry| entry.value().as_boolean())
                .ok_or(io::Error::other("missing Binary XCLIPFRAME"))?;
            assert_raw_value(
                DxfRawDocumentView::from(&binary),
                binary_xclipframe.raw_provenance(),
                &[1],
            )?;
        }
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

    let malformed_scalar_bytes = ascii_document(
        "AC1032",
        "9\n$CHAMFERC\n40\nnot-a-number\n9\n$FILLMODE\n70\n32768\n",
    );
    let malformed_scalar_source =
        DxfMemorySource::new(&malformed_scalar_bytes, DxfResourceProfile::Safe)?;
    let malformed_scalars = open_ascii(&malformed_scalar_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    for field_id in ["chamferc", "fillmode"] {
        let issue = malformed_scalars
            .entry(field_id)
            .and_then(|entry| match entry.value() {
                DxfHeaderNumericValue::Double(value) => value.invalid_issue(),
                DxfHeaderNumericValue::Int16(value) => value.invalid_issue(),
                _ => None,
            });
        assert!(matches!(
            issue,
            Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
        ));
    }

    let malformed_mode_bytes = ascii_document(
        "AC1032",
        "9\n$PDSIZE\n40\nnot-a-number\n9\n$PLIMCHECK\n70\n32768\n",
    );
    let malformed_mode_source =
        DxfMemorySource::new(&malformed_mode_bytes, DxfResourceProfile::Safe)?;
    let malformed_modes = open_ascii(&malformed_mode_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    for field_id in ["pdsize", "plimcheck"] {
        let issue = malformed_modes
            .entry(field_id)
            .and_then(|entry| match entry.value() {
                DxfHeaderNumericValue::Double(value) => value.invalid_issue(),
                DxfHeaderNumericValue::Int16(value) => value.invalid_issue(),
                _ => None,
            });
        assert!(matches!(
            issue,
            Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
        ));
    }

    let malformed_display_bytes = ascii_document(
        "AC1032",
        "9\n$PSVPSCALE\n40\nnot-a-number\n9\n$SHADEDIF\n70\n32768\n",
    );
    let malformed_display_source =
        DxfMemorySource::new(&malformed_display_bytes, DxfResourceProfile::Safe)?;
    let malformed_display = open_ascii(&malformed_display_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    for field_id in ["psvpscale", "shadedif"] {
        let issue = malformed_display
            .entry(field_id)
            .and_then(|entry| match entry.value() {
                DxfHeaderNumericValue::Double(value) => value.invalid_issue(),
                DxfHeaderNumericValue::Int16(value) => value.invalid_issue(),
                _ => None,
            });
        assert!(matches!(
            issue,
            Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
        ));
    }

    let malformed_surface_bytes = ascii_document(
        "AC1032",
        "9\n$TEXTSIZE\n40\nnot-a-number\n9\n$SURFTAB2\n70\n32768\n",
    );
    let malformed_surface_source =
        DxfMemorySource::new(&malformed_surface_bytes, DxfResourceProfile::Safe)?;
    let malformed_surface = open_ascii(&malformed_surface_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    for field_id in ["textsize", "surftab2"] {
        let issue = malformed_surface
            .entry(field_id)
            .and_then(|entry| match entry.value() {
                DxfHeaderNumericValue::Double(value) => value.invalid_issue(),
                DxfHeaderNumericValue::Int16(value) => value.invalid_issue(),
                _ => None,
            });
        assert!(matches!(
            issue,
            Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
        ));
    }

    let malformed_time_bytes = ascii_document(
        "AC1032",
        "9\n$TDCREATE\n40\nnot-a-number\n9\n$TDINDWG\n40\n1e9999\n",
    );
    let malformed_time_source =
        DxfMemorySource::new(&malformed_time_bytes, DxfResourceProfile::Safe)?;
    let malformed_time = open_ascii(&malformed_time_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    for field_id in ["tdcreate", "tdindwg"] {
        let issue = malformed_time
            .entry(field_id)
            .and_then(|entry| match entry.value() {
                DxfHeaderNumericValue::JulianDate(value) => value.invalid_issue(),
                DxfHeaderNumericValue::ElapsedDays(value) => value.invalid_issue(),
                _ => None,
            });
        assert!(matches!(
            issue,
            Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
        ));
    }

    let malformed_flag_bytes =
        ascii_document("AC1032", "9\n$ENDCAPS\n280\n32768\n9\n$EXTNAMES\n290\n2\n");
    let malformed_flag_source =
        DxfMemorySource::new(&malformed_flag_bytes, DxfResourceProfile::Safe)?;
    let malformed_flags = open_ascii(&malformed_flag_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        malformed_flags
            .entry("endcaps")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.invalid_issue()),
        Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_eq!(
        malformed_flags
            .entry("extnames")
            .and_then(|entry| entry.value().as_boolean())
            .and_then(|value| value.invalid_issue()),
        Some(&DxfHeaderNumericIssue::BooleanOutOfDomain { value: 2 })
    );

    let malformed_remaining_bytes =
        ascii_document("AC1032", "9\n$CELWEIGHT\n370\n32768\n9\n$XEDIT\n290\n-1\n");
    let malformed_remaining_source =
        DxfMemorySource::new(&malformed_remaining_bytes, DxfResourceProfile::Safe)?;
    let malformed_remaining = open_ascii(&malformed_remaining_source)?
        .header_numeric_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        malformed_remaining
            .entry("celweight")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.invalid_issue()),
        Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_eq!(
        malformed_remaining
            .entry("xedit")
            .and_then(|entry| entry.value().as_boolean())
            .and_then(|value| value.invalid_issue()),
        Some(&DxfHeaderNumericIssue::BooleanOutOfDomain { value: -1 })
    );
    Ok(())
}

#[test]
fn dimension_double_fields_preserve_failure_evidence() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_document(
        "AC1032",
        "9\n$DIMALTF\n41\n25.4\n9\n$DIMASZ\n40\nnot-a-number\n9\n$DIMTXT\n40\n1\n9\n$DIMTXT\n40\n2\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.header_numeric_directory(&DxfCancellationToken::default())?;

    let wrong_group = directory
        .entry("dimaltf")
        .and_then(|entry| entry.value().as_double())
        .ok_or(io::Error::other("missing DIMALTF"))?;
    assert_eq!(
        wrong_group.invalid_issue(),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(41)?))
    );
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        wrong_group.raw_provenance(),
        b"25.4",
    )?;

    let malformed = directory
        .entry("dimasz")
        .and_then(|entry| entry.value().as_double())
        .ok_or(io::Error::other("missing DIMASZ"))?;
    assert!(matches!(
        malformed.invalid_issue(),
        Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        malformed.raw_provenance(),
        b"not-a-number",
    )?;

    let duplicate = directory
        .entry("dimtxt")
        .and_then(|entry| entry.value().as_double())
        .ok_or(io::Error::other("missing DIMTXT"))?;
    assert_count_issue(duplicate.invalid_issue(), true, 2)?;
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        duplicate.raw_provenance(),
        b"2",
    )?;
    Ok(())
}

#[test]
fn dimension_formatting_fields_preserve_i16_failure_evidence() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_document(
        "AC1032",
        "9\n$DIMADEC\n71\n3\n9\n$DIMALTD\n70\n32768\n9\n$DIMZIN\n70\n8\n9\n$DIMZIN\n70\n12\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.header_numeric_directory(&DxfCancellationToken::default())?;

    let wrong_group = directory
        .entry("dimadec")
        .and_then(|entry| entry.value().as_int16())
        .ok_or(io::Error::other("missing DIMADEC"))?;
    assert_eq!(
        wrong_group.invalid_issue(),
        Some(&DxfHeaderNumericIssue::InvalidGroupCode(code(71)?))
    );
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        wrong_group.raw_provenance(),
        b"3",
    )?;

    let malformed = directory
        .entry("dimaltd")
        .and_then(|entry| entry.value().as_int16())
        .ok_or(io::Error::other("missing DIMALTD"))?;
    assert!(matches!(
        malformed.invalid_issue(),
        Some(DxfHeaderNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        malformed.raw_provenance(),
        b"32768",
    )?;

    let duplicate = directory
        .entry("dimzin")
        .and_then(|entry| entry.value().as_int16())
        .ok_or(io::Error::other("missing DIMZIN"))?;
    assert_count_issue(duplicate.invalid_issue(), true, 2)?;
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        duplicate.raw_provenance(),
        b"12",
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
    let directory = document.header_numeric_directory(&DxfCancellationToken::default())?;
    let created = directory
        .entry("tdcreate")
        .and_then(|entry| entry.value().as_julian_date())
        .and_then(|value| value.value())
        .copied()
        .ok_or(io::Error::other("missing binary date"))?;
    assert_eq!(created.raw().to_bits(), nan_bits);
    assert!(created.day_parts().is_none());
    let dimension_position = directory
        .entry("dimtvp")
        .and_then(|entry| entry.value().as_double())
        .ok_or(io::Error::other("missing binary DIMTVP"))?;
    assert_eq!(
        dimension_position.value().copied().map(DxfDouble::to_bits),
        Some(nan_bits)
    );
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        dimension_position.raw_provenance(),
        &nan_bits.to_le_bytes(),
    )?;
    let dimension_separator = directory
        .entry("dimdsep")
        .and_then(|entry| entry.value().as_int16())
        .ok_or(io::Error::other("missing binary DIMDSEP"))?;
    assert_eq!(dimension_separator.value(), Some(&i16::MIN));
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        dimension_separator.raw_provenance(),
        &i16::MIN.to_le_bytes(),
    )?;
    let invalid_boolean = directory
        .entry("extnames")
        .and_then(|entry| entry.value().as_boolean())
        .ok_or(io::Error::other("missing binary EXTNAMES"))?;
    assert_eq!(
        invalid_boolean.invalid_issue(),
        Some(&DxfHeaderNumericIssue::BooleanOutOfDomain { value: 255 })
    );
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        invalid_boolean.raw_provenance(),
        &[255],
    )?;
    let lineweight = directory
        .entry("celweight")
        .and_then(|entry| entry.value().as_int16())
        .ok_or(io::Error::other("missing binary CELWEIGHT"))?;
    assert_eq!(lineweight.value(), Some(&i16::MIN));
    assert_raw_value(
        DxfRawDocumentView::from(&document),
        lineweight.raw_provenance(),
        &i16::MIN.to_le_bytes(),
    )?;
    Ok(())
}

#[test]
fn date_and_elapsed_day_parts_are_exact_and_timezone_free() -> Result<(), Box<dyn Error>> {
    assert!(DxfDouble::from_f64(1.0).is_finite());

    let date_raw = DxfDouble::from_f64(2_451_544.915_682_87);
    let date = DxfJulianDate::from_raw(date_raw);
    assert_eq!(date.raw(), date_raw);
    let date_parts = date.day_parts().ok_or(io::Error::other("date parts"))?;
    assert_eq!(date_parts.whole_days(), 2_451_544);
    assert_eq!(
        date_parts.fractional_day().to_bits(),
        (date_raw.to_f64() - 2_451_544.0).to_bits()
    );

    let elapsed_raw = DxfDouble::from_f64(3.25);
    let elapsed = DxfElapsedDays::from_raw(elapsed_raw);
    assert_eq!(elapsed.raw(), elapsed_raw);
    let elapsed_parts = elapsed
        .day_parts()
        .ok_or(io::Error::other("elapsed parts"))?;
    assert_eq!(elapsed_parts.whole_days(), 3);
    assert_eq!(elapsed_parts.fractional_day().to_bits(), 0.25_f64.to_bits());

    assert!(
        DxfJulianDate::from_raw(DxfDouble::from_f64(f64::INFINITY))
            .day_parts()
            .is_none()
    );
    assert!(
        DxfElapsedDays::from_raw(DxfDouble::from_f64(9_223_372_036_854_775_808.0))
            .day_parts()
            .is_none()
    );
    let negative = DxfElapsedDays::from_raw(DxfDouble::from_f64(-3.25))
        .day_parts()
        .ok_or(io::Error::other("negative elapsed parts"))?;
    assert_eq!(negative.whole_days(), -3);
    assert_eq!(negative.fractional_day().to_bits(), (-0.25_f64).to_bits());

    let minimum = DxfElapsedDays::from_raw(DxfDouble::from_f64(i64::MIN as f64))
        .day_parts()
        .ok_or(io::Error::other("minimum elapsed parts"))?;
    assert_eq!(minimum.whole_days(), i64::MIN);
    assert_eq!(minimum.fractional_day().to_bits(), 0.0_f64.to_bits());
    Ok(())
}

#[test]
fn numeric_value_contract_preserves_tuple_states_and_late_provenance() -> Result<(), Box<dyn Error>>
{
    let field = DxfSemanticFieldProvenance::new(
        DxfSourceId::from([0x5a; DxfSourceId::BYTE_LEN]),
        "header",
        "tuple",
    );
    let span = ByteSpan::new(40, 48).ok_or(io::Error::other("tuple span"))?;
    let raw = DxfRawValueProvenance::new(7, span).ok_or(io::Error::other("tuple raw"))?;

    let defaulted2 = DxfHeaderNumericValue::Double2([
        DxfSemanticValue::defaulted(DxfDouble::from_f64(1.0), field),
        DxfSemanticValue::defaulted(DxfDouble::from_f64(2.0), field),
    ]);
    assert_eq!(defaulted2.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(defaulted2.raw_provenance(), None);

    let absent2 = DxfHeaderNumericValue::Double2([
        DxfSemanticValue::absent(field),
        DxfSemanticValue::absent(field),
    ]);
    assert_eq!(absent2.state(), DxfSemanticValueState::Absent);

    let defaulted3 = DxfHeaderNumericValue::Double3([
        DxfSemanticValue::defaulted(DxfDouble::from_f64(1.0), field),
        DxfSemanticValue::defaulted(DxfDouble::from_f64(2.0), field),
        DxfSemanticValue::defaulted(DxfDouble::from_f64(3.0), field),
    ]);
    assert_eq!(defaulted3.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(defaulted3.raw_provenance(), None);

    let late_raw = DxfHeaderNumericValue::Double3([
        DxfSemanticValue::absent(field),
        DxfSemanticValue::absent(field),
        DxfSemanticValue::explicit(DxfDouble::from_f64(3.0), field, raw),
    ]);
    assert_eq!(late_raw.state(), DxfSemanticValueState::Invalid);
    assert_eq!(late_raw.raw_provenance(), Some(raw));
    Ok(())
}

#[test]
fn cancellation_and_public_metadata_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_standard_fixture("AC1032")?;
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
    assert_copy::<DxfDayParts>();
    assert_copy::<DxfElapsedDays>();
    assert_copy::<DxfJulianDate>();
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
    version: DxfAcadVersion,
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
        (37, "cecolor", "$CECOLOR", &[62]),
        (38, "celtscale", "$CELTSCALE", &[40]),
        (39, "chamfera", "$CHAMFERA", &[40]),
        (40, "chamferb", "$CHAMFERB", &[40]),
        (41, "chamferc", "$CHAMFERC", &[40]),
        (42, "chamferd", "$CHAMFERD", &[40]),
        (43, "cmljust", "$CMLJUST", &[70]),
        (44, "cmlscale", "$CMLSCALE", &[40]),
        (45, "elevation", "$ELEVATION", &[40]),
        (46, "filletrad", "$FILLETRAD", &[40]),
        (47, "fillmode", "$FILLMODE", &[70]),
        (48, "ltscale", "$LTSCALE", &[40]),
        (49, "limcheck", "$LIMCHECK", &[70]),
        (50, "lunits", "$LUNITS", &[70]),
        (51, "luprec", "$LUPREC", &[70]),
        (52, "maxactvp", "$MAXACTVP", &[70]),
        (53, "measurement", "$MEASUREMENT", &[70]),
        (54, "mirrtext", "$MIRRTEXT", &[70]),
        (55, "orthomode", "$ORTHOMODE", &[70]),
        (56, "pdmode", "$PDMODE", &[70]),
        (57, "pdsize", "$PDSIZE", &[40]),
        (58, "pelevation", "$PELEVATION", &[40]),
        (59, "plimcheck", "$PLIMCHECK", &[70]),
        (60, "plinewid", "$PLINEWID", &[40]),
        (61, "plinegen", "$PLINEGEN", &[70]),
        (62, "proxygraphics", "$PROXYGRAPHICS", &[70]),
        (63, "psltscale", "$PSLTSCALE", &[70]),
        (64, "psvpscale", "$PSVPSCALE", &[40]),
        (65, "pucsorthoview", "$PUCSORTHOVIEW", &[70]),
        (66, "qtextmode", "$QTEXTMODE", &[70]),
        (67, "regenmode", "$REGENMODE", &[70]),
        (68, "shadedge", "$SHADEDGE", &[70]),
        (69, "shadedif", "$SHADEDIF", &[70]),
        (70, "shadowplanelocation", "$SHADOWPLANELOCATION", &[40]),
        (71, "sketchinc", "$SKETCHINC", &[40]),
        (72, "skpoly", "$SKPOLY", &[70]),
        (73, "splinesegs", "$SPLINESEGS", &[70]),
        (74, "splinetype", "$SPLINETYPE", &[70]),
        (75, "surftab1", "$SURFTAB1", &[70]),
        (76, "surftab2", "$SURFTAB2", &[70]),
        (77, "surftype", "$SURFTYPE", &[70]),
        (78, "surfu", "$SURFU", &[70]),
        (79, "surfv", "$SURFV", &[70]),
        (80, "textsize", "$TEXTSIZE", &[40]),
        (81, "thickness", "$THICKNESS", &[40]),
        (82, "tilemode", "$TILEMODE", &[70]),
        (83, "tracewid", "$TRACEWID", &[40]),
        (84, "treedepth", "$TREEDEPTH", &[70]),
        (85, "tdcreate", "$TDCREATE", &[40]),
        (86, "tducreate", "$TDUCREATE", &[40]),
        (87, "tdupdate", "$TDUPDATE", &[40]),
        (88, "tduupdate", "$TDUUPDATE", &[40]),
        (89, "tdindwg", "$TDINDWG", &[40]),
        (90, "tdusrtimer", "$TDUSRTIMER", &[40]),
        (91, "endcaps", "$ENDCAPS", &[280]),
        (92, "extnames", "$EXTNAMES", &[290]),
        (93, "halogap", "$HALOGAP", &[280]),
        (94, "hidetext", "$HIDETEXT", &[290]),
        (95, "indexctl", "$INDEXCTL", &[280]),
        (96, "intersectiondisplay", "$INTERSECTIONDISPLAY", &[290]),
        (97, "joinstyle", "$JOINSTYLE", &[280]),
        (98, "lwdisplay", "$LWDISPLAY", &[290]),
        (99, "obsltype", "$OBSLTYPE", &[280]),
        (100, "pstylemode", "$PSTYLEMODE", &[290]),
        (101, "celweight", "$CELWEIGHT", &[370]),
        (102, "cepsntype", "$CEPSNTYPE", &[380]),
        (103, "cshadow", "$CSHADOW", &[280]),
        (104, "dispsilh", "$DISPSILH", &[70]),
        (105, "insunits", "$INSUNITS", &[70]),
        (106, "interferecolor", "$INTERFERECOLOR", &[62]),
        (107, "intersectioncolor", "$INTERSECTIONCOLOR", &[70]),
        (108, "obscolor", "$OBSCOLOR", &[70]),
        (109, "sortents", "$SORTENTS", &[280]),
        (110, "ucsorthoview", "$UCSORTHOVIEW", &[70]),
        (111, "unitmode", "$UNITMODE", &[70]),
        (112, "usrtimer", "$USRTIMER", &[70]),
        (113, "visretain", "$VISRETAIN", &[70]),
        (114, "worldview", "$WORLDVIEW", &[70]),
        (115, "xclipframe", "$XCLIPFRAME", &[290]),
        (116, "xedit", "$XEDIT", &[290]),
        (117, "dimaltf", "$DIMALTF", &[40]),
        (118, "dimaltrnd", "$DIMALTRND", &[40]),
        (119, "dimasz", "$DIMASZ", &[40]),
        (120, "dimcen", "$DIMCEN", &[40]),
        (121, "dimdle", "$DIMDLE", &[40]),
        (122, "dimdli", "$DIMDLI", &[40]),
        (123, "dimexe", "$DIMEXE", &[40]),
        (124, "dimexo", "$DIMEXO", &[40]),
        (125, "dimfac", "$DIMFAC", &[40]),
        (126, "dimgap", "$DIMGAP", &[40]),
        (127, "dimlfac", "$DIMLFAC", &[40]),
        (128, "dimrnd", "$DIMRND", &[40]),
        (129, "dimscale", "$DIMSCALE", &[40]),
        (130, "dimtfac", "$DIMTFAC", &[40]),
        (131, "dimtm", "$DIMTM", &[40]),
        (132, "dimtp", "$DIMTP", &[40]),
        (133, "dimtsz", "$DIMTSZ", &[40]),
        (134, "dimtvp", "$DIMTVP", &[40]),
        (135, "dimtxt", "$DIMTXT", &[40]),
        (136, "dimadec", "$DIMADEC", &[70]),
        (137, "dimaltd", "$DIMALTD", &[70]),
        (138, "dimalttd", "$DIMALTTD", &[70]),
        (139, "dimalttz", "$DIMALTTZ", &[70]),
        (140, "dimaltu", "$DIMALTU", &[70]),
        (141, "dimaltz", "$DIMALTZ", &[70]),
        (142, "dimaunit", "$DIMAUNIT", &[70]),
        (143, "dimazin", "$DIMAZIN", &[70]),
        (144, "dimdec", "$DIMDEC", &[70]),
        (145, "dimdsep", "$DIMDSEP", &[70]),
        (146, "dimlunit", "$DIMLUNIT", &[70]),
        (147, "dimtdec", "$DIMTDEC", &[70]),
        (148, "dimtzin", "$DIMTZIN", &[70]),
        (149, "dimzin", "$DIMZIN", &[70]),
    ];
    assert_eq!(directory.entries().len(), expected.len());
    for (entry, &(ordinal, id, name, group_codes)) in directory.entries().iter().zip(expected) {
        assert_eq!(entry.schema_ordinal(), ordinal);
        assert_eq!(entry.schema_field_id(), id);
        assert_eq!(entry.dxf_name(), name);
        assert_eq!(entry.group_codes(), group_codes);
        let unavailable_in_r12_binary = matches!(ordinal, 91..=103 | 109 | 115..=116);
        let expected_state = if version == DxfAcadVersion::Ac1009 && unavailable_in_r12_binary {
            DxfSemanticValueState::Absent
        } else {
            DxfSemanticValueState::Explicit
        };
        assert_eq!(entry.value().state(), expected_state);
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
    assert_eq!(
        directory
            .entry("cecolor")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&256)
    );
    assert_eq!(
        directory
            .entry("chamferd")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(std::f64::consts::FRAC_PI_4.to_bits())
    );
    assert_eq!(
        directory
            .entry("fillmode")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&1)
    );
    assert_eq!(
        directory
            .entry("ltscale")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(2.5_f64.to_bits())
    );
    assert_eq!(
        directory
            .entry("measurement")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&1)
    );
    assert_eq!(
        directory
            .entry("pdsize")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some((-3.5_f64).to_bits())
    );
    assert_eq!(
        directory
            .entry("plimcheck")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&0)
    );
    assert_eq!(
        directory
            .entry("plinewid")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(0.75_f64.to_bits())
    );
    assert_eq!(
        directory
            .entry("shadedif")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&70)
    );
    assert_eq!(
        directory
            .entry("psvpscale")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(1.5_f64.to_bits())
    );
    assert_eq!(
        directory
            .entry("shadowplanelocation")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some((-100.25_f64).to_bits())
    );
    assert_eq!(
        directory
            .entry("skpoly")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&2)
    );
    assert_eq!(
        directory
            .entry("splinesegs")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&8)
    );
    assert_eq!(
        directory
            .entry("textsize")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some(2.5_f64.to_bits())
    );
    assert_eq!(
        directory
            .entry("thickness")
            .and_then(|entry| entry.value().as_double())
            .and_then(|value| value.value())
            .copied()
            .map(DxfDouble::to_bits),
        Some((-1.25_f64).to_bits())
    );
    assert_eq!(
        directory
            .entry("treedepth")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&10)
    );
    let created = directory
        .entry("tdcreate")
        .and_then(|entry| entry.value().as_julian_date())
        .and_then(|value| value.value())
        .copied()
        .ok_or(io::Error::other("missing TDCREATE"))?;
    assert_eq!(created.raw().to_bits(), 2_451_544.915_682_87_f64.to_bits());
    assert!(
        directory
            .entry("tdcreate")
            .and_then(|entry| entry.value().as_double())
            .is_none()
    );
    let timer = directory
        .entry("tdusrtimer")
        .and_then(|entry| entry.value().as_elapsed_days())
        .and_then(|value| value.value())
        .copied()
        .ok_or(io::Error::other("missing TDUSRTIMER"))?;
    assert_eq!(timer.raw().to_bits(), 0.5_f64.to_bits());
    assert!(
        directory
            .entry("tdusrtimer")
            .and_then(|entry| entry.value().as_double())
            .is_none()
    );
    if version != DxfAcadVersion::Ac1009 {
        assert_eq!(
            directory
                .entry("endcaps")
                .and_then(|entry| entry.value().as_int16())
                .and_then(|value| value.value()),
            Some(&2)
        );
        assert_eq!(
            directory
                .entry("extnames")
                .and_then(|entry| entry.value().as_boolean())
                .and_then(|value| value.value()),
            Some(&true)
        );
        assert_eq!(
            directory
                .entry("pstylemode")
                .and_then(|entry| entry.value().as_boolean())
                .and_then(|value| value.value()),
            Some(&false)
        );
    }
    assert!(
        directory
            .entry("extnames")
            .and_then(|entry| entry.value().as_int16())
            .is_none()
    );
    assert_eq!(
        directory
            .entry("insunits")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&6)
    );
    assert_eq!(
        directory
            .entry("intersectioncolor")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&257)
    );
    assert_eq!(
        directory
            .entry("worldview")
            .and_then(|entry| entry.value().as_int16())
            .and_then(|value| value.value()),
        Some(&0)
    );
    for (field_id, expected) in [
        ("dimaltf", 25.4_f64),
        ("dimaltrnd", 0.125),
        ("dimasz", 2.5),
        ("dimcen", -0.5),
        ("dimdle", 1.25),
        ("dimdli", 3.75),
        ("dimexe", 1.5),
        ("dimexo", 0.625),
        ("dimfac", 0.75),
        ("dimgap", -0.25),
        ("dimlfac", 10.0),
        ("dimrnd", 0.05),
        ("dimscale", 100.0),
        ("dimtfac", 0.625),
        ("dimtm", 0.01),
        ("dimtp", 0.02),
        ("dimtsz", 0.0),
        ("dimtvp", -0.75),
        ("dimtxt", 2.25),
    ] {
        assert_eq!(
            directory
                .entry(field_id)
                .and_then(|entry| entry.value().as_double())
                .and_then(|value| value.value())
                .copied()
                .map(DxfDouble::to_bits),
            Some(expected.to_bits())
        );
    }
    for (field_id, expected) in [
        ("dimadec", 3_i16),
        ("dimaltd", 4),
        ("dimalttd", 2),
        ("dimalttz", 12),
        ("dimaltu", 2),
        ("dimaltz", 8),
        ("dimaunit", 0),
        ("dimazin", 3),
        ("dimdec", 5),
        ("dimdsep", 44),
        ("dimlunit", 2),
        ("dimtdec", 4),
        ("dimtzin", 12),
        ("dimzin", 8),
    ] {
        assert_eq!(
            directory
                .entry(field_id)
                .and_then(|entry| entry.value().as_int16())
                .and_then(|value| value.value()),
            Some(&expected)
        );
    }
    if version != DxfAcadVersion::Ac1009 {
        assert_eq!(
            directory
                .entry("celweight")
                .and_then(|entry| entry.value().as_int16())
                .and_then(|value| value.value()),
            Some(&-1)
        );
        assert_eq!(
            directory
                .entry("xclipframe")
                .and_then(|entry| entry.value().as_boolean())
                .and_then(|value| value.value()),
            Some(&true)
        );
        assert_eq!(
            directory
                .entry("xedit")
                .and_then(|entry| entry.value().as_boolean())
                .and_then(|value| value.value()),
            Some(&false)
        );
    }
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

fn ascii_standard_fixture(version: &str) -> Result<Vec<u8>, io::Error> {
    let mut bytes = ascii_document(
        version,
        "9\n$ACADMAINTVER\n70\n-32768\n9\n$ANGBASE\n50\n +5.000000000000000E-1 \n9\n$ANGDIR\n70\n+1\n9\n$ATTMODE\n70\n2\n9\n$AUNITS\n70\n0\n9\n$AUPREC\n70\n4\n9\n$EXTMAX\n10\n1.25\n20\n-2.5\n30\n3.75\n9\n$EXTMIN\n10\n-4.5\n20\n5.25\n30\n-6.75\n9\n$INSBASE\n10\n7\n20\n8\n30\n9\n9\n$LIMMAX\n10\n10\n20\n20\n9\n$LIMMIN\n10\n-10\n20\n-20\n9\n$PEXTMAX\n10\n11\n20\n22\n30\n33\n9\n$PEXTMIN\n10\n-11\n20\n-22\n30\n-33\n9\n$PINSBASE\n10\n0.125\n20\n0.25\n30\n0.5\n9\n$PLIMMAX\n10\n100\n20\n200\n9\n$PLIMMIN\n10\n-100\n20\n-200\n9\n$PUCSORG\n10\n1\n20\n2\n30\n3\n9\n$PUCSXDIR\n10\n1\n20\n0\n30\n0\n9\n$PUCSYDIR\n10\n0\n20\n1\n30\n0\n9\n$UCSORG\n10\n-1\n20\n-2\n30\n-3\n9\n$UCSXDIR\n10\n1\n20\n0\n30\n0\n9\n$UCSYDIR\n10\n0\n20\n1\n30\n0\n9\n$PUCSORGBACK\n10\n101\n20\n102\n30\n103\n9\n$PUCSORGBOTTOM\n10\n111\n20\n112\n30\n113\n9\n$PUCSORGFRONT\n10\n121\n20\n122\n30\n123\n9\n$PUCSORGLEFT\n10\n131\n20\n132\n30\n133\n9\n$PUCSORGRIGHT\n10\n141\n20\n142\n30\n143\n9\n$PUCSORGTOP\n10\n151\n20\n152\n30\n153\n9\n$UCSORGBACK\n10\n-101\n20\n-102\n30\n-103\n9\n$UCSORGBOTTOM\n10\n-111\n20\n-112\n30\n-113\n9\n$UCSORGFRONT\n10\n-121\n20\n-122\n30\n-123\n9\n$UCSORGLEFT\n10\n-131\n20\n-132\n30\n-133\n9\n$UCSORGRIGHT\n10\n-141\n20\n-142\n30\n-143\n9\n$UCSORGTOP\n10\n-151\n20\n-152\n30\n-153\n9\n$CECOLOR\n62\n256\n9\n$CELTSCALE\n40\n0.25\n9\n$CHAMFERA\n40\n1.25\n9\n$CHAMFERB\n40\n2.5\n9\n$CHAMFERC\n40\n3.75\n9\n$CHAMFERD\n40\n0.7853981633974483\n9\n$CMLJUST\n70\n2\n9\n$CMLSCALE\n40\n20\n9\n$ELEVATION\n40\n-12.5\n9\n$FILLETRAD\n40\n4.25\n9\n$FILLMODE\n70\n1\n9\n$LTSCALE\n40\n2.5\n9\n$LIMCHECK\n70\n1\n9\n$LUNITS\n70\n2\n9\n$LUPREC\n70\n4\n9\n$MAXACTVP\n70\n64\n9\n$MEASUREMENT\n70\n1\n9\n$MIRRTEXT\n70\n0\n9\n$ORTHOMODE\n70\n1\n9\n$PDMODE\n70\n34\n9\n$PDSIZE\n40\n-3.5\n9\n$PELEVATION\n40\n-7.25\n9\n$PLIMCHECK\n70\n0\n9\n$PLINEWID\n40\n0.75\n9\n$PLINEGEN\n70\n1\n9\n$PROXYGRAPHICS\n70\n1\n9\n$PSLTSCALE\n70\n0\n9\n$PSVPSCALE\n40\n1.5\n9\n$PUCSORTHOVIEW\n70\n6\n9\n$QTEXTMODE\n70\n0\n9\n$REGENMODE\n70\n1\n9\n$SHADEDGE\n70\n3\n9\n$SHADEDIF\n70\n70\n9\n$SHADOWPLANELOCATION\n40\n-100.25\n9\n$SKETCHINC\n40\n0.5\n9\n$SKPOLY\n70\n2\n9\n$SPLINESEGS\n70\n8\n9\n$SPLINETYPE\n70\n6\n9\n$SURFTAB1\n70\n6\n9\n$SURFTAB2\n70\n8\n9\n$SURFTYPE\n70\n6\n9\n$SURFU\n70\n12\n9\n$SURFV\n70\n14\n9\n$TEXTSIZE\n40\n2.5\n9\n$THICKNESS\n40\n-1.25\n9\n$TILEMODE\n70\n1\n9\n$TRACEWID\n40\n0.375\n9\n$TREEDEPTH\n70\n10\n9\n$TDCREATE\n40\n2451544.91568287\n9\n$TDUCREATE\n40\n2451544.5\n9\n$TDUPDATE\n40\n2451545.25\n9\n$TDUUPDATE\n40\n2451545.75\n9\n$TDINDWG\n40\n3.25\n9\n$TDUSRTIMER\n40\n0.5\n9\n$ENDCAPS\n280\n2\n9\n$EXTNAMES\n290\n1\n9\n$HALOGAP\n280\n25\n9\n$HIDETEXT\n290\n0\n9\n$INDEXCTL\n280\n3\n9\n$INTERSECTIONDISPLAY\n290\n1\n9\n$JOINSTYLE\n280\n2\n9\n$LWDISPLAY\n290\n1\n9\n$OBSLTYPE\n280\n4\n9\n$PSTYLEMODE\n290\n0\n",
    );
    let remaining_fields = b"9\n$CELWEIGHT\n370\n-1\n9\n$CEPSNTYPE\n380\n3\n9\n$CSHADOW\n280\n2\n9\n$DISPSILH\n70\n1\n9\n$INSUNITS\n70\n6\n9\n$INTERFERECOLOR\n62\n1\n9\n$INTERSECTIONCOLOR\n70\n257\n9\n$OBSCOLOR\n70\n256\n9\n$SORTENTS\n280\n127\n9\n$UCSORTHOVIEW\n70\n4\n9\n$UNITMODE\n70\n1\n9\n$USRTIMER\n70\n1\n9\n$VISRETAIN\n70\n1\n9\n$WORLDVIEW\n70\n0\n9\n$XCLIPFRAME\n290\n1\n9\n$XEDIT\n290\n0\n";
    let dimension_fields = b"9\n$DIMALTF\n40\n25.4\n9\n$DIMALTRND\n40\n0.125\n9\n$DIMASZ\n40\n2.5\n9\n$DIMCEN\n40\n-0.5\n9\n$DIMDLE\n40\n1.25\n9\n$DIMDLI\n40\n3.75\n9\n$DIMEXE\n40\n1.5\n9\n$DIMEXO\n40\n0.625\n9\n$DIMFAC\n40\n0.75\n9\n$DIMGAP\n40\n-0.25\n9\n$DIMLFAC\n40\n10\n9\n$DIMRND\n40\n0.05\n9\n$DIMSCALE\n40\n100\n9\n$DIMTFAC\n40\n0.625\n9\n$DIMTM\n40\n0.01\n9\n$DIMTP\n40\n0.02\n9\n$DIMTSZ\n40\n0\n9\n$DIMTVP\n40\n-0.75\n9\n$DIMTXT\n40\n2.25\n";
    let dimension_formatting_fields = b"9\n$DIMADEC\n70\n3\n9\n$DIMALTD\n70\n4\n9\n$DIMALTTD\n70\n2\n9\n$DIMALTTZ\n70\n12\n9\n$DIMALTU\n70\n2\n9\n$DIMALTZ\n70\n8\n9\n$DIMAUNIT\n70\n0\n9\n$DIMAZIN\n70\n3\n9\n$DIMDEC\n70\n5\n9\n$DIMDSEP\n70\n44\n9\n$DIMLUNIT\n70\n2\n9\n$DIMTDEC\n70\n4\n9\n$DIMTZIN\n70\n12\n9\n$DIMZIN\n70\n8\n";
    let document_suffix = b"0\nENDSEC\n0\nEOF\n";
    bytes.truncate(bytes.len().saturating_sub(document_suffix.len()));
    bytes.extend_from_slice(remaining_fields);
    bytes.extend_from_slice(dimension_fields);
    bytes.extend_from_slice(dimension_formatting_fields);
    bytes.extend_from_slice(document_suffix);
    if version == "AC1009" {
        let extension = b"9\n$ENDCAPS\n";
        let extension_start = bytes
            .windows(extension.len())
            .position(|window| window == extension)
            .ok_or(io::Error::other("missing extended fixture fields"))?;
        bytes.truncate(extension_start);
        bytes.extend_from_slice(b"9\n$DISPSILH\n70\n1\n9\n$INSUNITS\n70\n6\n9\n$INTERFERECOLOR\n62\n1\n9\n$INTERSECTIONCOLOR\n70\n257\n9\n$OBSCOLOR\n70\n256\n9\n$UCSORTHOVIEW\n70\n4\n9\n$UNITMODE\n70\n1\n9\n$USRTIMER\n70\n1\n9\n$VISRETAIN\n70\n1\n9\n$WORLDVIEW\n70\n0\n");
        bytes.extend_from_slice(dimension_fields);
        bytes.extend_from_slice(dimension_formatting_fields);
        bytes.extend_from_slice(document_suffix);
    }
    Ok(bytes)
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
    for (name, group_code, value) in [
        (b"$CECOLOR".as_slice(), 62_i16, 256_i16),
        (b"$CMLJUST".as_slice(), 70, 2),
        (b"$FILLMODE".as_slice(), 70, 1),
        (b"$LIMCHECK".as_slice(), 70, 1),
        (b"$LUNITS".as_slice(), 70, 2),
        (b"$LUPREC".as_slice(), 70, 4),
        (b"$MAXACTVP".as_slice(), 70, 64),
        (b"$MEASUREMENT".as_slice(), 70, 1),
        (b"$MIRRTEXT".as_slice(), 70, 0),
        (b"$ORTHOMODE".as_slice(), 70, 1),
        (b"$PDMODE".as_slice(), 70, 34),
        (b"$PLIMCHECK".as_slice(), 70, 0),
        (b"$PLINEGEN".as_slice(), 70, 1),
        (b"$PROXYGRAPHICS".as_slice(), 70, 1),
        (b"$PSLTSCALE".as_slice(), 70, 0),
        (b"$PUCSORTHOVIEW".as_slice(), 70, 6),
        (b"$QTEXTMODE".as_slice(), 70, 0),
        (b"$REGENMODE".as_slice(), 70, 1),
        (b"$SHADEDGE".as_slice(), 70, 3),
        (b"$SHADEDIF".as_slice(), 70, 70),
        (b"$SKPOLY".as_slice(), 70, 2),
        (b"$SPLINESEGS".as_slice(), 70, 8),
        (b"$SPLINETYPE".as_slice(), 70, 6),
        (b"$SURFTAB1".as_slice(), 70, 6),
        (b"$SURFTAB2".as_slice(), 70, 8),
        (b"$SURFTYPE".as_slice(), 70, 6),
        (b"$SURFU".as_slice(), 70, 12),
        (b"$SURFV".as_slice(), 70, 14),
        (b"$TILEMODE".as_slice(), 70, 1),
        (b"$TREEDEPTH".as_slice(), 70, 10),
        (b"$DISPSILH".as_slice(), 70, 1),
        (b"$INSUNITS".as_slice(), 70, 6),
        (b"$INTERFERECOLOR".as_slice(), 62, 1),
        (b"$INTERSECTIONCOLOR".as_slice(), 70, 257),
        (b"$OBSCOLOR".as_slice(), 70, 256),
        (b"$UCSORTHOVIEW".as_slice(), 70, 4),
        (b"$UNITMODE".as_slice(), 70, 1),
        (b"$USRTIMER".as_slice(), 70, 1),
        (b"$VISRETAIN".as_slice(), 70, 1),
        (b"$WORLDVIEW".as_slice(), 70, 0),
        (b"$DIMADEC".as_slice(), 70, 3),
        (b"$DIMALTD".as_slice(), 70, 4),
        (b"$DIMALTTD".as_slice(), 70, 2),
        (b"$DIMALTTZ".as_slice(), 70, 12),
        (b"$DIMALTU".as_slice(), 70, 2),
        (b"$DIMALTZ".as_slice(), 70, 8),
        (b"$DIMAUNIT".as_slice(), 70, 0),
        (b"$DIMAZIN".as_slice(), 70, 3),
        (b"$DIMDEC".as_slice(), 70, 5),
        (b"$DIMDSEP".as_slice(), 70, 44),
        (b"$DIMLUNIT".as_slice(), 70, 2),
        (b"$DIMTDEC".as_slice(), 70, 4),
        (b"$DIMTZIN".as_slice(), 70, 12),
        (b"$DIMZIN".as_slice(), 70, 8),
    ] {
        push_binary_string(&mut bytes, version, 9, name)?;
        push_binary_i16(&mut bytes, version, group_code, value)?;
    }
    for (name, value) in [
        (b"$CELTSCALE".as_slice(), 0.25_f64),
        (b"$CHAMFERA".as_slice(), 1.25),
        (b"$CHAMFERB".as_slice(), 2.5),
        (b"$CHAMFERC".as_slice(), 3.75),
        (b"$CHAMFERD".as_slice(), std::f64::consts::FRAC_PI_4),
        (b"$CMLSCALE".as_slice(), 20.0),
        (b"$ELEVATION".as_slice(), -12.5),
        (b"$FILLETRAD".as_slice(), 4.25),
        (b"$LTSCALE".as_slice(), 2.5),
        (b"$PDSIZE".as_slice(), -3.5),
        (b"$PELEVATION".as_slice(), -7.25),
        (b"$PLINEWID".as_slice(), 0.75),
        (b"$PSVPSCALE".as_slice(), 1.5),
        (b"$SHADOWPLANELOCATION".as_slice(), -100.25),
        (b"$SKETCHINC".as_slice(), 0.5),
        (b"$TEXTSIZE".as_slice(), 2.5),
        (b"$THICKNESS".as_slice(), -1.25),
        (b"$TRACEWID".as_slice(), 0.375),
        (b"$DIMALTF".as_slice(), 25.4),
        (b"$DIMALTRND".as_slice(), 0.125),
        (b"$DIMASZ".as_slice(), 2.5),
        (b"$DIMCEN".as_slice(), -0.5),
        (b"$DIMDLE".as_slice(), 1.25),
        (b"$DIMDLI".as_slice(), 3.75),
        (b"$DIMEXE".as_slice(), 1.5),
        (b"$DIMEXO".as_slice(), 0.625),
        (b"$DIMFAC".as_slice(), 0.75),
        (b"$DIMGAP".as_slice(), -0.25),
        (b"$DIMLFAC".as_slice(), 10.0),
        (b"$DIMRND".as_slice(), 0.05),
        (b"$DIMSCALE".as_slice(), 100.0),
        (b"$DIMTFAC".as_slice(), 0.625),
        (b"$DIMTM".as_slice(), 0.01),
        (b"$DIMTP".as_slice(), 0.02),
        (b"$DIMTSZ".as_slice(), 0.0),
        (b"$DIMTVP".as_slice(), -0.75),
        (b"$DIMTXT".as_slice(), 2.25),
    ] {
        push_binary_string(&mut bytes, version, 9, name)?;
        push_binary_double_bits(&mut bytes, version, 40, value.to_bits())?;
    }
    for (name, value) in [
        (b"$TDCREATE".as_slice(), 2_451_544.915_682_87_f64),
        (b"$TDUCREATE".as_slice(), 2_451_544.5),
        (b"$TDUPDATE".as_slice(), 2_451_545.25),
        (b"$TDUUPDATE".as_slice(), 2_451_545.75),
        (b"$TDINDWG".as_slice(), 3.25),
        (b"$TDUSRTIMER".as_slice(), 0.5),
    ] {
        push_binary_string(&mut bytes, version, 9, name)?;
        push_binary_double_bits(&mut bytes, version, 40, value.to_bits())?;
    }
    if version != DxfAcadVersion::Ac1009 {
        for (name, value) in [
            (b"$ENDCAPS".as_slice(), 2_i16),
            (b"$HALOGAP".as_slice(), 25),
            (b"$INDEXCTL".as_slice(), 3),
            (b"$JOINSTYLE".as_slice(), 2),
            (b"$OBSLTYPE".as_slice(), 4),
        ] {
            push_binary_string(&mut bytes, version, 9, name)?;
            push_binary_i16(&mut bytes, version, 280, value)?;
        }
        for (name, group_code, value) in [
            (b"$CELWEIGHT".as_slice(), 370_i16, -1_i16),
            (b"$CEPSNTYPE".as_slice(), 380, 3),
            (b"$CSHADOW".as_slice(), 280, 2),
            (b"$SORTENTS".as_slice(), 280, 127),
        ] {
            push_binary_string(&mut bytes, version, 9, name)?;
            push_binary_i16(&mut bytes, version, group_code, value)?;
        }
        for (name, value) in [
            (b"$EXTNAMES".as_slice(), true),
            (b"$HIDETEXT".as_slice(), false),
            (b"$INTERSECTIONDISPLAY".as_slice(), true),
            (b"$LWDISPLAY".as_slice(), true),
            (b"$PSTYLEMODE".as_slice(), false),
            (b"$XCLIPFRAME".as_slice(), true),
            (b"$XEDIT".as_slice(), false),
        ] {
            push_binary_string(&mut bytes, version, 9, name)?;
            push_binary_boolean(&mut bytes, version, 290, value)?;
        }
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
    push_binary_string(&mut bytes, version, 9, b"$TDCREATE")?;
    push_binary_double_bits(&mut bytes, version, 40, angle_bits)?;
    push_binary_string(&mut bytes, version, 9, b"$EXTNAMES")?;
    push_binary_boolean_raw(&mut bytes, version, 290, u8::MAX)?;
    push_binary_string(&mut bytes, version, 9, b"$CELWEIGHT")?;
    push_binary_i16(&mut bytes, version, 370, i16::MIN)?;
    push_binary_string(&mut bytes, version, 9, b"$DIMTVP")?;
    push_binary_double_bits(&mut bytes, version, 40, angle_bits)?;
    push_binary_string(&mut bytes, version, 9, b"$DIMDSEP")?;
    push_binary_i16(&mut bytes, version, 70, i16::MIN)?;
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

fn push_binary_boolean(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: bool,
) -> Result<(), io::Error> {
    push_binary_boolean_raw(bytes, version, group_code, u8::from(value))
}

fn push_binary_boolean_raw(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: u8,
) -> Result<(), io::Error> {
    push_binary_group_code(bytes, version, group_code)?;
    bytes.push(value);
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
