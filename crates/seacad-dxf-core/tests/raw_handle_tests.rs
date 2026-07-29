use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandle, DxfHandleGroupClass, DxfHandleParseIssue,
    DxfMemorySource, DxfRawDocumentView, DxfRawHandleLookup, DxfRawHandleValue, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn ascii_and_binary_project_every_handle_class_with_exact_spelling() -> Result<(), Box<dyn Error>> {
    let cases = [
        (5, "000a", DxfHandleGroupClass::ObjectIdentity, 0x0a),
        (105, "B", DxfHandleGroupClass::ObjectIdentity, 0x0b),
        (320, "c", DxfHandleGroupClass::Arbitrary, 0x0c),
        (330, "D", DxfHandleGroupClass::SoftPointer, 0x0d),
        (340, "0e", DxfHandleGroupClass::HardPointer, 0x0e),
        (350, "F", DxfHandleGroupClass::SoftOwner, 0x0f),
        (360, "10", DxfHandleGroupClass::HardOwner, 0x10),
        (390, "11", DxfHandleGroupClass::HardPointer, 0x11),
        (480, "12", DxfHandleGroupClass::HardPointer, 0x12),
        (1005, "13", DxfHandleGroupClass::SoftPointer, 0x13),
    ];

    let ascii_bytes = ascii_fixture("AC1032", &cases.map(|(code, raw, _, _)| (code, raw)));
    let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    assert_cases(DxfRawDocumentView::from(&ascii), &cases)?;

    let binary_bytes = binary_fixture(
        DxfAcadVersion::Ac1032,
        &cases.map(|(code, raw, _, _)| (code, raw)),
    )?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    assert_cases(DxfRawDocumentView::from(&binary), &cases)?;

    let non_handle = occurrence_of(DxfRawDocumentView::from(&ascii), 2)?;
    match ascii.raw_handle_at(non_handle, &DxfCancellationToken::default())? {
        DxfRawHandleLookup::NotHandleGroup(group) => assert_eq!(group.group_code().value(), 2),
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    assert_eq!(
        binary.raw_handle_at(u64::MAX, &DxfCancellationToken::default())?,
        DxfRawHandleLookup::MissingOccurrence
    );
    Ok(())
}

#[test]
fn invalid_handle_spelling_remains_typed_and_source_anchored() -> Result<(), Box<dyn Error>> {
    let cases = [
        (5, "", DxfHandleParseIssue::Empty),
        (330, "00000000000000000", DxfHandleParseIssue::TooLong),
        (340, "0x1", DxfHandleParseIssue::InvalidDigit { offset: 1 }),
    ];
    let pairs = cases.map(|(code, raw, _)| (code, raw));

    let ascii_bytes = ascii_fixture("AC1032", &pairs);
    let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    assert_invalid_cases(DxfRawDocumentView::from(&ascii), &cases)?;

    let binary_bytes = binary_fixture(DxfAcadVersion::Ac1032, &pairs)?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    assert_invalid_cases(DxfRawDocumentView::from(&binary), &cases)?;
    Ok(())
}

#[test]
fn pre_r13_xdata_escape_has_ascii_binary_projection_parity() -> Result<(), Box<dyn Error>> {
    let cases = [
        (5, "a", DxfHandleGroupClass::ObjectIdentity, 0x0a),
        (1005, "00B", DxfHandleGroupClass::SoftPointer, 0x0b),
    ];
    let pairs = cases.map(|(code, raw, _, _)| (code, raw));

    let ascii_bytes = ascii_fixture("AC1009", &pairs);
    let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    assert_cases(DxfRawDocumentView::from(&ascii), &cases)?;

    let binary_bytes = binary_fixture(DxfAcadVersion::Ac1009, &pairs)?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    assert_cases(DxfRawDocumentView::from(&binary), &cases)?;
    Ok(())
}

#[test]
fn cancellation_source_identity_and_public_traits_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfRawHandleLookup>();
    assert_copy::<DxfRawHandleValue>();
    assert_send_sync::<DxfRawHandleLookup>();
    assert_send_sync::<DxfRawHandleValue>();

    let first_bytes = ascii_fixture("AC1032", &[(5, "ABC")]);
    let first_source = DxfMemorySource::new(&first_bytes, DxfResourceProfile::Safe)?;
    let first = open_ascii(&first_source)?;
    let occurrence = occurrence_of(DxfRawDocumentView::from(&first), 5)?;

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        first.raw_handle_at(occurrence, &cancelled),
        Err(DxfError::Cancelled)
    ));

    let value = require_handle(first.raw_handle_at(occurrence, &DxfCancellationToken::default())?)?;
    let second_bytes = ascii_fixture("AC1032", &[(5, "ABD")]);
    let second_source = DxfMemorySource::new(&second_bytes, DxfResourceProfile::Safe)?;
    let second = open_ascii(&second_source)?;
    let mut spelling = [0_u8; 3];
    assert!(matches!(
        value.read_raw_spelling(DxfRawDocumentView::from(&second), &mut spelling),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let read_cancel = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&first_bytes, read_cancel.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    let occurrence = occurrence_of(DxfRawDocumentView::from(&cancelling_document), 5)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.raw_handle_at(occurrence, &read_cancel),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_cases(
    view: DxfRawDocumentView<'_>,
    cases: &[(i16, &str, DxfHandleGroupClass, u64)],
) -> Result<(), Box<dyn Error>> {
    for (code, raw, expected_class, expected_handle) in cases {
        let occurrence = occurrence_of(view, *code)?;
        let value =
            require_handle(view.raw_handle_at(occurrence, &DxfCancellationToken::default())?)?;
        assert_eq!(value.source_id(), view.source_id());
        assert_eq!(value.group().occurrence(), occurrence);
        assert_eq!(value.group().group_code().value(), *code);
        assert_eq!(value.class(), *expected_class);
        assert_eq!(
            value.parse_result(),
            Ok(DxfHandle::from_u64(*expected_handle))
        );
        let mut spelling = vec![0_u8; raw.len()];
        value.read_raw_spelling(view, &mut spelling)?;
        assert_eq!(spelling, raw.as_bytes());
    }
    Ok(())
}

fn assert_invalid_cases(
    view: DxfRawDocumentView<'_>,
    cases: &[(i16, &str, DxfHandleParseIssue)],
) -> Result<(), Box<dyn Error>> {
    for (code, raw, expected_issue) in cases {
        let occurrence = occurrence_of(view, *code)?;
        let value =
            require_handle(view.raw_handle_at(occurrence, &DxfCancellationToken::default())?)?;
        assert_eq!(value.parse_result(), Err(*expected_issue));
        let mut spelling = vec![0_u8; raw.len()];
        value.read_raw_spelling(view, &mut spelling)?;
        assert_eq!(spelling, raw.as_bytes());
    }
    Ok(())
}

fn require_handle(lookup: DxfRawHandleLookup) -> Result<DxfRawHandleValue, io::Error> {
    match lookup {
        DxfRawHandleLookup::Handle(value) => Ok(value),
        other => Err(io::Error::other(format!("expected handle, got {other:?}"))),
    }
}

fn occurrence_of(view: DxfRawDocumentView<'_>, code: i16) -> Result<u64, io::Error> {
    for occurrence in 0..view.group_count() {
        if view
            .group(occurrence)
            .is_some_and(|group| group.group_code().value() == code)
        {
            return Ok(occurrence);
        }
    }
    Err(io::Error::other(format!("missing group code {code}")))
}

fn ascii_fixture(version: &str, pairs: &[(i16, &str)]) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n"
    );
    for (code, value) in pairs {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n0\nEOF\n");
    text.into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, pairs: &[(i16, &str)]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "OBJECTS"),
    ] {
        push_binary_string(&mut bytes, version, code, value.as_bytes())?;
    }
    for (code, value) in pairs {
        push_binary_string(&mut bytes, version, *code, value.as_bytes())?;
    }
    for (code, value) in [(0, "ENDSEC"), (0, "EOF")] {
        push_binary_string(&mut bytes, version, code, value.as_bytes())?;
    }
    Ok(bytes)
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        if (0..=254).contains(&group_code) {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
        } else if (1000..=1071).contains(&group_code) {
            bytes.push(u8::MAX);
            bytes.extend_from_slice(&group_code.to_le_bytes());
        } else {
            return Err(io::Error::other("group code unavailable before R13"));
        }
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
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
        if self.armed.swap(false, Ordering::AcqRel) {
            self.token.cancel();
        }
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
