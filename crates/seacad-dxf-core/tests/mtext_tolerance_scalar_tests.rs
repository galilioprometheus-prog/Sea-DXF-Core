use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextNumericSemantics, DxfMTextToleranceScalarDirectory,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    DxfTextSymbolDoubleValue, DxfTextSymbolInt16Value, DxfTextSymbolInt32Value, DxfTextSymbolKind,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue, DxfTextSymbolValueRole,
    DxfToleranceNumericSemantics, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct ScalarSignature {
    state: DxfSemanticValueState,
    value: Option<u64>,
}

type RecordSignature = (DxfTextSymbolKind, Vec<ScalarSignature>);

#[test]
fn every_dialect_has_ascii_binary_scalar_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.mtext_tolerance_scalar_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.mtext_tolerance_scalar_directory(&DxfCancellationToken::default())?;

        let extended_codes = version != DxfAcadVersion::Ac1009;
        assert_values(&ascii_directory, extended_codes)?;
        assert_values(&binary_directory, extended_codes)?;
        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn missing_invalid_multiple_and_partial_optional_values_stay_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n10\n1\n20\n2\n40\n.\n71\n1\n71\n2\n11\n9\n42\n.\n76\n2\n76\n3\n220\n4\n220\n5\n\
0\nTOLERANCE\n10\n.\n20\n5\n30\n6\n11\n1\n21\n0\n210\n2\n210\n3\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_tolerance_scalar_directory(&DxfCancellationToken::default())?;

    let mtext = mtext(&directory, 0)?;
    assert_missing(&mtext.insertion()[2]);
    assert!(matches!(
        mtext.nominal_height().invalid_issue(),
        Some(DxfTextSymbolScalarIssue::InvalidAsciiNumber(_))
    ));
    assert_missing(mtext.reference_width());
    assert_multiple_i16(mtext.attachment(), 2);
    assert_missing_i16(mtext.drawing_direction());
    assert_eq!(mtext.x_axis()[0].state(), DxfSemanticValueState::Explicit);
    assert_eq!(mtext.x_axis()[1].state(), DxfSemanticValueState::Absent);
    assert_eq!(mtext.x_axis()[2].state(), DxfSemanticValueState::Absent);
    assert!(matches!(
        mtext.actual_width().invalid_issue(),
        Some(DxfTextSymbolScalarIssue::InvalidAsciiNumber(_))
    ));
    assert_multiple_i16(mtext.column_count(), 2);
    assert_multiple(&mtext.extrusion()[1], 2);
    assert_eq!(
        mtext.extrusion()[2].state(),
        DxfSemanticValueState::Defaulted
    );

    let tolerance = tolerance(&directory, 1)?;
    assert!(matches!(
        tolerance.insertion()[0].invalid_issue(),
        Some(DxfTextSymbolScalarIssue::InvalidAsciiNumber(_))
    ));
    assert_missing(&tolerance.x_axis()[2]);
    assert_multiple(&tolerance.extrusion()[0], 2);
    assert_eq!(
        tolerance.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    Ok(())
}

#[test]
fn ambiguity_scope_cancellation_identity_lookups_and_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_tolerance_scalar_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.mtext_tolerance_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(directory.mtext_semantics_for_raw_record(u64::MAX)?, None);
    assert_eq!(
        directory.tolerance_semantics_for_raw_record(u64::MAX)?,
        None
    );

    let mtext_record = directory.records()[0];
    let cards = directory.card_directory();
    let raw = mtext_record.record().ordinal();
    let group_50 = cards
        .card_for_role(raw, DxfTextSymbolValueRole::RotationOrColumnHeight)
        .ok_or(io::Error::other("group 50 card"))?;
    assert_eq!(group_50.member_range().len(), 2);
    let ambiguous_color = cards
        .card_for_role(raw, DxfTextSymbolValueRole::BackgroundRgbOrEntityTrueColor)
        .ok_or(io::Error::other("group 420 card"))?;
    assert_eq!(ambiguous_color.member_range().len(), 1);

    let text_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nTEXT\n10\n1\n0\nSHAPE\n10\n2\n0\nENDSEC\n0\nEOF\n";
    let text_source = DxfMemorySource::new(text_bytes, DxfResourceProfile::Safe)?;
    let text_document = open_ascii(&text_source)?;
    let scoped = text_document.mtext_tolerance_scalar_directory(&DxfCancellationToken::default())?;
    for record in scoped.records() {
        assert_eq!(scoped.mtext_semantics_for_record(*record)?, None);
        assert_eq!(scoped.tolerance_semantics_for_record(*record)?, None);
    }

    assert_copy::<DxfMTextNumericSemantics>();
    assert_copy::<DxfToleranceNumericSemantics>();
    assert_send_sync::<DxfMTextToleranceScalarDirectory>();
    Ok(())
}

fn assert_values(
    directory: &DxfMTextToleranceScalarDirectory,
    extended_codes: bool,
) -> Result<(), Box<dyn Error>> {
    let mtext = mtext(directory, 0)?;
    assert_eq!(bits3(mtext.insertion()), [1.0, 2.0, 3.0].map(f64::to_bits));
    assert_eq!(double_bits(mtext.nominal_height()), Some(4.0_f64.to_bits()));
    assert_eq!(
        double_bits(mtext.reference_width()),
        Some(5.0_f64.to_bits())
    );
    assert_eq!(mtext.attachment().value(), Some(&1));
    assert_eq!(mtext.drawing_direction().value(), Some(&3));
    assert_eq!(bits3(mtext.extrusion()), [0.0, 0.0, 1.0].map(f64::to_bits));
    assert!(
        mtext
            .extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );
    assert!(
        mtext
            .x_axis()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Absent)
    );
    assert_eq!(double_bits(mtext.actual_width()), Some(4.5_f64.to_bits()));
    assert_eq!(double_bits(mtext.actual_height()), Some(8.0_f64.to_bits()));
    assert_eq!(mtext.line_spacing_style().value(), Some(&2));
    assert_eq!(
        double_bits(mtext.line_spacing_factor()),
        Some(1.25_f64.to_bits())
    );
    assert_eq!(mtext.background_fill().value(), Some(&1));
    assert_eq!(double_bits(mtext.fill_box_scale()), Some(1.5_f64.to_bits()));
    assert_eq!(mtext.background_index().value(), Some(&7));
    if extended_codes {
        assert_eq!(mtext.background_transparency().value(), Some(&123));
    } else {
        assert_eq!(
            mtext.background_transparency().state(),
            DxfSemanticValueState::Absent
        );
    }
    assert_eq!(mtext.column_type().value(), Some(&2));
    assert_eq!(mtext.column_count().value(), Some(&3));
    assert_eq!(mtext.column_flow_reversed().value(), Some(&1));
    assert_eq!(mtext.column_auto_height().value(), Some(&0));
    assert_eq!(double_bits(mtext.column_width()), Some(10.0_f64.to_bits()));
    assert_eq!(double_bits(mtext.column_gutter()), Some(2.0_f64.to_bits()));
    assert_eq!(mtext.record().kind(), DxfTextSymbolKind::MText);

    let tolerance = tolerance(directory, 1)?;
    assert_eq!(
        bits3(tolerance.insertion()),
        [10.0, 11.0, 12.0].map(f64::to_bits)
    );
    assert_eq!(bits3(tolerance.x_axis()), [1.0, 0.0, 0.0].map(f64::to_bits));
    assert_eq!(
        bits3(tolerance.extrusion()),
        [0.0, 0.0, 1.0].map(f64::to_bits)
    );
    assert!(
        tolerance
            .extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );
    assert_eq!(tolerance.record().kind(), DxfTextSymbolKind::Tolerance);
    Ok(())
}

fn signatures(
    directory: &DxfMTextToleranceScalarDirectory,
) -> Result<Vec<RecordSignature>, DxfError> {
    let mut signatures = Vec::new();
    for record in directory.records().iter().copied() {
        match record.kind() {
            DxfTextSymbolKind::MText => {
                if let Some(value) = directory.mtext_semantics_for_record(record)? {
                    let mut scalars = Vec::new();
                    scalars.extend(value.insertion().each_ref().map(double_signature));
                    scalars.push(double_signature(value.nominal_height()));
                    scalars.push(double_signature(value.reference_width()));
                    scalars.push(i16_signature(value.attachment()));
                    scalars.push(i16_signature(value.drawing_direction()));
                    scalars.extend(value.extrusion().each_ref().map(double_signature));
                    scalars.extend(value.x_axis().each_ref().map(double_signature));
                    scalars.push(double_signature(value.actual_width()));
                    scalars.push(double_signature(value.actual_height()));
                    scalars.push(i16_signature(value.line_spacing_style()));
                    scalars.push(double_signature(value.line_spacing_factor()));
                    scalars.push(i32_signature(value.background_fill()));
                    scalars.push(double_signature(value.fill_box_scale()));
                    scalars.push(i16_signature(value.background_index()));
                    scalars.push(i32_signature(value.background_transparency()));
                    scalars.push(i16_signature(value.column_type()));
                    scalars.push(i16_signature(value.column_count()));
                    scalars.push(i16_signature(value.column_flow_reversed()));
                    scalars.push(i16_signature(value.column_auto_height()));
                    scalars.push(double_signature(value.column_width()));
                    scalars.push(double_signature(value.column_gutter()));
                    signatures.push((record.kind(), scalars));
                }
            }
            DxfTextSymbolKind::Tolerance => {
                if let Some(value) = directory.tolerance_semantics_for_record(record)? {
                    let scalars = value
                        .insertion()
                        .iter()
                        .chain(value.extrusion())
                        .chain(value.x_axis())
                        .map(double_signature)
                        .collect();
                    signatures.push((record.kind(), scalars));
                }
            }
            DxfTextSymbolKind::Text | DxfTextSymbolKind::Shape => {}
            _ => {}
        }
    }
    Ok(signatures)
}

fn mtext(
    directory: &DxfMTextToleranceScalarDirectory,
    index: usize,
) -> Result<DxfMTextNumericSemantics, Box<dyn Error>> {
    let record = record(directory, index)?;
    directory
        .mtext_semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("MTEXT semantics").into())
}

fn tolerance(
    directory: &DxfMTextToleranceScalarDirectory,
    index: usize,
) -> Result<DxfToleranceNumericSemantics, Box<dyn Error>> {
    let record = record(directory, index)?;
    directory
        .tolerance_semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("TOLERANCE semantics").into())
}

fn record(
    directory: &DxfMTextToleranceScalarDirectory,
    index: usize,
) -> Result<DxfTextSymbolRecordEntry, Box<dyn Error>> {
    directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(|| io::Error::other("record").into())
}

fn double_signature(value: &DxfTextSymbolDoubleValue) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        value: double_bits(value),
    }
}

fn i16_signature(value: &DxfTextSymbolInt16Value) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        value: value.value().map(|value| *value as u16 as u64),
    }
}

fn i32_signature(value: &DxfTextSymbolInt32Value) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        value: value.value().map(|value| *value as u32 as u64),
    }
}

fn double_bits(value: &DxfTextSymbolDoubleValue) -> Option<u64> {
    value.value().map(|number| number.to_bits())
}

fn bits3(values: &[DxfTextSymbolDoubleValue; 3]) -> [u64; 3] {
    values.map(|value| double_bits(&value).unwrap_or(u64::MAX))
}

fn assert_missing(value: &DxfTextSymbolDoubleValue) {
    assert_eq!(value.state(), DxfSemanticValueState::Invalid);
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfTextSymbolScalarIssue::MissingRequiredValue)
    );
    assert_eq!(value.raw_provenance(), None);
}

fn assert_missing_i16(value: &DxfTextSymbolInt16Value) {
    assert_eq!(value.state(), DxfSemanticValueState::Invalid);
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfTextSymbolScalarIssue::MissingRequiredValue)
    );
    assert_eq!(value.raw_provenance(), None);
}

fn assert_multiple(value: &DxfTextSymbolDoubleValue, count: u32) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfTextSymbolScalarIssue::MultipleValues {
            occurrence_count: count
        })
    );
    assert!(value.raw_provenance().is_some());
}

fn assert_multiple_i16(value: &DxfTextSymbolInt16Value, count: u32) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfTextSymbolScalarIssue::MultipleValues {
            occurrence_count: count
        })
    );
    assert!(value.raw_provenance().is_some());
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    let extended_codes = if version == DxfAcadVersion::Ac1009.code() {
        ""
    } else {
        "441\n123\n420\n16711680\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n10\n1\n20\n2\n30\n3\n40\n4\n41\n5\n71\n1\n72\n3\n\
42\n4.5\n43\n8\n73\n2\n44\n1.25\n90\n1\n45\n1.5\n63\n7\n{extended_codes}\
75\n2\n76\n3\n78\n1\n79\n0\n48\n10\n49\n2\n50\n.25\n50\n.5\n1\nA\n\
0\nTOLERANCE\n10\n10\n20\n11\n30\n12\n11\n1\n21\n0\n31\n0\n3\nISO\n1\nA\n\
0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"MTEXT")?;
    push_vector(&mut bytes, version, [10, 20, 30], [1.0, 2.0, 3.0])?;
    for (code, value) in [
        (40, 4.0),
        (41, 5.0),
        (42, 4.5),
        (43, 8.0),
        (44, 1.25),
        (45, 1.5),
        (48, 10.0),
        (49, 2.0),
        (50, 0.25),
        (50, 0.5),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (71, 1),
        (72, 3),
        (73, 2),
        (63, 7),
        (75, 2),
        (76, 3),
        (78, 1),
        (79, 0),
    ] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 90, 1)?;
    if version != DxfAcadVersion::Ac1009 {
        push_i32(&mut bytes, version, 441, 123)?;
        push_i32(&mut bytes, version, 420, 16_711_680)?;
    }
    push_string(&mut bytes, version, 1, b"A")?;
    push_string(&mut bytes, version, 0, b"TOLERANCE")?;
    push_vector(&mut bytes, version, [10, 20, 30], [10.0, 11.0, 12.0])?;
    push_vector(&mut bytes, version, [11, 21, 31], [1.0, 0.0, 0.0])?;
    push_string(&mut bytes, version, 3, b"ISO")?;
    push_string(&mut bytes, version, 1, b"A")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_vector(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    codes: [i16; 3],
    values: [f64; 3],
) -> io::Result<()> {
    for (code, value) in codes.into_iter().zip(values) {
        push_double(bytes, version, code, value)?;
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

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
