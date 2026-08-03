use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataApplication, DxfEntityXDataApplicationState,
    DxfEntityXDataDirectory, DxfEntityXDataOccurrence, DxfEntityXDataOccurrenceKind, DxfError,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_entity_xdata_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_directory(DxfRawDocumentView::from(&ascii))?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_directory(DxfRawDocumentView::from(&binary))?;
    }
    Ok(())
}

#[test]
fn duplicate_app_names_orphans_and_interruption_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = document.entity_xdata_directory(&DxfCancellationToken::default())?;

    let first = directory.applications()[0];
    let duplicate = directory.applications()[1];
    assert_eq!(exact_value(view, first.application_name())?, b"APP_ONE");
    assert_eq!(exact_value(view, duplicate.application_name())?, b"APP_ONE");
    assert_ne!(first.ordinal(), duplicate.ordinal());

    let first_entity = first.entity();
    let first_occurrences = directory.occurrences_for_entity(first_entity)?;
    assert!(matches!(
        first_occurrences[0].kind(),
        DxfEntityXDataOccurrenceKind::Orphan
    ));
    assert_eq!(first_occurrences[0].group().group_code().value(), 1000);

    let interrupted = directory.applications()[2];
    assert_eq!(
        interrupted.state(),
        DxfEntityXDataApplicationState::Interrupted
    );
    let line_occurrences = directory.occurrences_for_entity(interrupted.entity())?;
    assert!(line_occurrences.iter().any(|entry| {
        entry.group().group_code().value() == 1040
            && entry.kind() == DxfEntityXDataOccurrenceKind::Orphan
    }));
    Ok(())
}

#[test]
fn directory_is_source_bound_cancellable_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataDirectory>();
    assert_copy::<DxfEntityXDataApplication>();
    assert_copy::<DxfEntityXDataOccurrence>();
    let application_size = std::mem::size_of::<DxfEntityXDataApplication>();
    let occurrence_size = std::mem::size_of::<DxfEntityXDataOccurrence>();
    assert!(
        application_size <= 192,
        "XDATA application metadata grew to {application_size} bytes"
    );
    assert!(
        occurrence_size <= 160,
        "XDATA occurrence metadata grew to {occurrence_size} bytes"
    );

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.application(u64::MAX), None);
    assert_eq!(directory.occurrence(u64::MAX), None);
    assert_eq!(directory.occurrence_for_group(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("APP_ONE"));
    assert!(!debug.contains("PAYLOAD"));

    let other_bytes = ascii_fixture(DxfAcadVersion::Ac1027);
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_directory = other_document.entity_xdata_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        other_directory.occurrences_for_application(directory.applications()[0]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_xdata_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.applications().len(), 4);
    assert_eq!(directory.occurrences().len(), 13);
    assert_eq!(
        directory
            .applications()
            .iter()
            .map(|entry| exact_value(view, entry.application_name()))
            .collect::<Result<Vec<_>, _>>()?,
        [
            b"APP_ONE".to_vec(),
            b"APP_ONE".to_vec(),
            b"BROKEN".to_vec(),
            b"RESTART".to_vec()
        ]
    );
    assert_eq!(
        directory
            .applications()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfEntityXDataApplicationState::Contiguous,
            DxfEntityXDataApplicationState::Contiguous,
            DxfEntityXDataApplicationState::Interrupted,
            DxfEntityXDataApplicationState::Contiguous,
        ]
    );
    assert_eq!(
        directory
            .applications()
            .iter()
            .map(|entry| entry.occurrence_range().len())
            .collect::<Vec<_>>(),
        [5, 2, 2, 2]
    );
    for (ordinal, application) in directory.applications().iter().copied().enumerate() {
        assert_eq!(
            directory.application(u64::try_from(ordinal)?),
            Some(application)
        );
        assert_eq!(application.application_name().group_code().value(), 1001);
        let members = directory.occurrences_for_application(application)?;
        assert_eq!(
            members.first().map(|entry| entry.group()),
            Some(application.application_name())
        );
        assert!(matches!(
            members.first().map(|entry| entry.kind()),
            Some(DxfEntityXDataOccurrenceKind::ApplicationName { application_ordinal })
                if u64::from(application_ordinal) == application.ordinal()
        ));
        assert!(members.iter().skip(1).all(|entry| matches!(
            entry.kind(),
            DxfEntityXDataOccurrenceKind::ApplicationValue { application_ordinal }
                if u64::from(application_ordinal) == application.ordinal()
        )));
    }
    for (ordinal, occurrence) in directory.occurrences().iter().copied().enumerate() {
        assert_eq!(
            directory.occurrence(u64::try_from(ordinal)?),
            Some(occurrence)
        );
        assert_eq!(
            directory.occurrence_for_group(occurrence.group().occurrence()),
            Some(occurrence)
        );
        assert!((1000..=1071).contains(&occurrence.group().group_code().value()));
    }
    assert_eq!(
        directory
            .applications_for_entity(directory.applications()[0].entity())?
            .len(),
        2
    );
    assert_eq!(
        directory
            .applications_for_entity(directory.applications()[2].entity())?
            .len(),
        2
    );
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
         0\nSECTION\n2\nENTITIES\n0\nPOINT\n8\n0\n10\n1\n20\n2\n30\n3\n\
         1000\nORPHAN\n1001\nAPP_ONE\n1000\nPAYLOAD\n1002\n{{\n1070\n7\n1002\n}}\n\
         1001\nAPP_ONE\n1005\nA\n0\nLINE\n8\n0\n1001\nBROKEN\n1000\nBEFORE\n\
         8\nAFTER_XDATA\n1040\n1.5\n102\n{{CUSTOM\n1001\nHIDDEN\n1005\nB\n102\n}}\n\
         1001\nRESTART\n1071\n2\n0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POINT"),
        (8, b"0"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(10, 1.0), (20, 2.0), (30, 3.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (1000, b"ORPHAN".as_slice()),
        (1001, b"APP_ONE"),
        (1000, b"PAYLOAD"),
        (1002, b"{"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 1070, 7)?;
    for (code, value) in [
        (1002, b"}".as_slice()),
        (1001, b"APP_ONE"),
        (1005, b"A"),
        (0, b"LINE"),
        (8, b"0"),
        (1001, b"BROKEN"),
        (1000, b"BEFORE"),
        (8, b"AFTER_XDATA"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 1040, 1.5)?;
    for (code, value) in [
        (102, b"{CUSTOM".as_slice()),
        (1001, b"HIDDEN"),
        (1005, b"B"),
        (102, b"}"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 1001, b"RESTART")?;
    push_i32(&mut bytes, version, 1071, 2)?;
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| invalid_test_data())?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else if version == DxfAcadVersion::Ac1009 {
        return Err(invalid_test_data());
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn exact_value(
    view: DxfRawDocumentView<'_>,
    group: seacad_dxf_core::DxfRawGroup,
) -> Result<Vec<u8>, DxfError> {
    let span = group.value_payload_span();
    let mut value = vec![0_u8; usize::try_from(span.len()).map_err(|_| invalid_core_data())?];
    view.read_span(span, &mut value)?;
    Ok(value)
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

fn invalid_core_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
