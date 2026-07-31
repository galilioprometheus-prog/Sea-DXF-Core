use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfTextJustificationAxis, DxfTextLayoutIssue,
    DxfTextOcsPlacementAnchorDirectory, DxfTextOcsPlacementAnchorIssue,
    DxfTextOcsPlacementAnchorKind, DxfTextOcsPlacementAnchorSemantics, DxfTextPlacementComponent,
    DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    First,
    Horizontal,
    Vertical,
    Both,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    state: DxfSemanticValueState,
    kind: Option<DxfTextOcsPlacementAnchorKind>,
    point_bits: Option<[u64; 3]>,
}

#[test]
fn every_dialect_has_ascii_binary_anchor_parity() -> Result<(), Box<dyn Error>> {
    let rows = [Row::First, Row::Horizontal, Row::Vertical, Row::Both];
    let expected = [
        explicit(
            DxfTextOcsPlacementAnchorKind::FirstAlignment,
            [1.0, 2.0, 3.0],
        ),
        explicit(
            DxfTextOcsPlacementAnchorKind::SecondAlignment,
            [4.0, 5.0, 6.0],
        ),
        explicit(
            DxfTextOcsPlacementAnchorKind::SecondAlignment,
            [4.0, 5.0, 6.0],
        ),
        explicit(
            DxfTextOcsPlacementAnchorKind::SecondAlignment,
            [4.0, 5.0, 6.0],
        ),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.text_ocs_placement_anchor_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.text_ocs_placement_anchor_directory(&cancellation)?;

        assert_eq!(signatures(&ascii_directory)?, expected);
        assert_eq!(signatures(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn absent_invalid_duplicate_and_unsupported_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n72\n1\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n72\n1\n11\n4\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n72\n1\n11\nbad\n21\n5\n31\n6\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n72\n9\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n73\n9\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n72\n1\n11\n4\n11\n7\n21\n5\n31\n6\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.text_ocs_placement_anchor_directory(&DxfCancellationToken::default())?;

    assert_issue(
        semantics(&directory, 0)?,
        DxfTextOcsPlacementAnchorIssue::ComponentInvalid {
            anchor: DxfTextOcsPlacementAnchorKind::FirstAlignment,
            component: DxfTextPlacementComponent::X,
            issue: DxfTextSymbolScalarIssue::MissingRequiredValue,
        },
    );
    assert_issue(
        semantics(&directory, 1)?,
        DxfTextOcsPlacementAnchorIssue::ComponentAbsent {
            anchor: DxfTextOcsPlacementAnchorKind::SecondAlignment,
            component: DxfTextPlacementComponent::X,
        },
    );
    assert_issue(
        semantics(&directory, 2)?,
        DxfTextOcsPlacementAnchorIssue::ComponentAbsent {
            anchor: DxfTextOcsPlacementAnchorKind::SecondAlignment,
            component: DxfTextPlacementComponent::Y,
        },
    );
    assert!(matches!(
        semantics(&directory, 3)?.anchor().invalid_issue(),
        Some(DxfTextOcsPlacementAnchorIssue::ComponentInvalid {
            anchor: DxfTextOcsPlacementAnchorKind::SecondAlignment,
            component: DxfTextPlacementComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        })
    ));
    assert_issue(
        semantics(&directory, 4)?,
        DxfTextOcsPlacementAnchorIssue::JustificationInvalid {
            axis: DxfTextJustificationAxis::Horizontal,
            issue: DxfTextLayoutIssue::UnsupportedHorizontalCode { code: 9 },
        },
    );
    assert_issue(
        semantics(&directory, 5)?,
        DxfTextOcsPlacementAnchorIssue::JustificationInvalid {
            axis: DxfTextJustificationAxis::Vertical,
            issue: DxfTextLayoutIssue::UnsupportedVerticalCode { code: 9 },
        },
    );
    assert_issue(
        semantics(&directory, 6)?,
        DxfTextOcsPlacementAnchorIssue::ComponentInvalid {
            anchor: DxfTextOcsPlacementAnchorKind::SecondAlignment,
            component: DxfTextPlacementComponent::X,
            issue: DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2,
            },
        },
    );
    for index in [3, 4, 5, 6] {
        assert!(
            semantics(&directory, index)?
                .anchor()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfTextOcsPlacementAnchorSemantics>();
    assert_send_sync::<DxfTextOcsPlacementAnchorDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::First]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_ocs_placement_anchor_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nTEXT\n10\n1\n20\n2\n30\n3\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.text_ocs_placement_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.layout_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
    let text = directory.records()[0];
    let shape = directory.records()[1];
    assert!(directory.semantics_for_record(text)?.is_some());
    assert_eq!(directory.semantics_for_record(shape)?, None);
    assert!(
        directory
            .semantics_for_raw_record(text.record().ordinal())?
            .is_some()
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn explicit(kind: DxfTextOcsPlacementAnchorKind, point: [f64; 3]) -> Signature {
    Signature {
        state: DxfSemanticValueState::Explicit,
        kind: Some(kind),
        point_bits: Some(point.map(f64::to_bits)),
    }
}

fn signatures(directory: &DxfTextOcsPlacementAnchorDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => {
                let anchor = semantics.anchor();
                Some(Ok(Signature {
                    state: anchor.state(),
                    kind: anchor.value().map(|value| value.kind()),
                    point_bits: anchor
                        .value()
                        .map(|value| value.point().map(|number| number.to_bits())),
                }))
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfTextOcsPlacementAnchorDirectory,
    index: usize,
) -> Result<DxfTextOcsPlacementAnchorSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(invalid_test_data)?;
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_issue(
    semantics: DxfTextOcsPlacementAnchorSemantics,
    expected: DxfTextOcsPlacementAnchorIssue,
) {
    assert_eq!(semantics.anchor().invalid_issue(), Some(&expected));
}

fn ascii_fixture(version: &str, rows: &[Row]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str("0\nTEXT\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n");
        match row {
            Row::First => {}
            Row::Horizontal => entities.push_str("72\n1\n"),
            Row::Vertical => entities.push_str("73\n3\n"),
            Row::Both => entities.push_str("72\n3\n73\n2\n"),
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[Row]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for row in rows {
        push_string(&mut bytes, version, 0, b"TEXT")?;
        push_point(&mut bytes, version, [10, 20, 30], [1.0, 2.0, 3.0])?;
        push_point(&mut bytes, version, [11, 21, 31], [4.0, 5.0, 6.0])?;
        match row {
            Row::First => {}
            Row::Horizontal => push_i16(&mut bytes, version, 72, 1)?,
            Row::Vertical => push_i16(&mut bytes, version, 73, 3)?,
            Row::Both => {
                push_i16(&mut bytes, version, 72, 3)?;
                push_i16(&mut bytes, version, 73, 2)?;
            }
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_point(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    codes: [i16; 3],
    values: [f64; 3],
) -> io::Result<()> {
    for (code, value) in codes.into_iter().zip(values) {
        push_code(bytes, version, code)?;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| invalid_test_data())?);
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

fn invalid_test_data() -> io::Error {
    io::Error::other("invalid test data")
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
