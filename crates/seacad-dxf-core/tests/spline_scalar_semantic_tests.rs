use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_SPLINE_SCALAR_ROLES, DxfAcadVersion, DxfAsciiNumericIssue,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSplineNumber, DxfSplineScalarDirectory,
    DxfSplineScalarEntry, DxfSplineScalarState, DxfSplineValueRole, NoopDxfReadObserver,
};

type StateEvidence = (
    DxfSplineValueRole,
    u8,
    Option<DxfSplineNumber>,
    Option<DxfAsciiNumericIssue>,
    u32,
);

#[test]
fn every_dialect_has_ascii_binary_scalar_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_scalar_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_scalar_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(states(&ascii_directory)?, states(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn absent_invalid_and_duplicate_states_do_not_invent_counts() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n71\n2\n71\n3\n72\n.\n74\n-1\n\
42\n1e-9999\n43\n1\n43\n2\n0\nSPLINE\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 14);
    let first = directory.card_directory().evidence_directory().records()[0];
    let raw = first.record().ordinal();
    assert_eq!(
        state(&directory, raw, DxfSplineValueRole::Degree)?,
        DxfSplineScalarState::Multiple {
            occurrence_count: 2
        }
    );
    assert!(matches!(
        state(&directory, raw, DxfSplineValueRole::KnotCount)?,
        DxfSplineScalarState::Invalid {
            issue: DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 },
            ..
        }
    ));
    assert_eq!(
        state(&directory, raw, DxfSplineValueRole::ControlPointCount)?,
        DxfSplineScalarState::Absent
    );
    assert!(matches!(
        state(&directory, raw, DxfSplineValueRole::FitPointCount)?,
        DxfSplineScalarState::Explicit(value)
            if value.value() == Ok(DxfSplineNumber::Int16(-1))
    ));
    assert!(matches!(
        state(&directory, raw, DxfSplineValueRole::KnotTolerance)?,
        DxfSplineScalarState::Invalid {
            issue: DxfAsciiNumericIssue::OutOfRange,
            ..
        }
    ));
    assert_eq!(
        state(&directory, raw, DxfSplineValueRole::ControlPointTolerance)?,
        DxfSplineScalarState::Multiple {
            occurrence_count: 2
        }
    );
    assert_default(
        state(&directory, raw, DxfSplineValueRole::FitTolerance)?,
        1.0e-10,
    );

    let empty_raw = directory.card_directory().evidence_directory().records()[1]
        .record()
        .ordinal();
    for role in [
        DxfSplineValueRole::Degree,
        DxfSplineValueRole::KnotCount,
        DxfSplineValueRole::ControlPointCount,
        DxfSplineValueRole::FitPointCount,
    ] {
        assert_eq!(
            state(&directory, empty_raw, role)?,
            DxfSplineScalarState::Absent
        );
    }
    for (role, expected) in [
        (DxfSplineValueRole::KnotTolerance, 1.0e-7),
        (DxfSplineValueRole::ControlPointTolerance, 1.0e-7),
        (DxfSplineValueRole::FitTolerance, 1.0e-10),
    ] {
        assert_default(state(&directory, empty_raw, role)?, expected);
    }
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.spline_scalar_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.spline_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.entry_for_role(u64::MAX, DxfSplineValueRole::Degree),
        None
    );
    assert_copy::<DxfSplineScalarEntry>();
    assert_copy::<DxfSplineScalarState>();
    assert_send_sync::<DxfSplineScalarDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfSplineScalarDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), DXF_SPLINE_SCALAR_ROLES.len());
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    let record = directory.card_directory().evidence_directory().records()[0];
    let entries = directory
        .entries_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("scalar entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.role()).collect::<Vec<_>>(),
        DXF_SPLINE_SCALAR_ROLES
    );
    for entry in entries.iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
    }
    assert!(matches!(
        entries[0].state(),
        DxfSplineScalarState::Explicit(_)
    ));
    assert!(matches!(
        entries[1].state(),
        DxfSplineScalarState::Explicit(_)
    ));
    assert!(matches!(
        entries[2].state(),
        DxfSplineScalarState::Explicit(_)
    ));
    assert!(matches!(
        entries[3].state(),
        DxfSplineScalarState::Explicit(_)
    ));
    assert!(matches!(
        entries[4].state(),
        DxfSplineScalarState::Explicit(_)
    ));
    assert_default(entries[5].state(), 1.0e-7);
    assert_default(entries[6].state(), 1.0e-10);
    Ok(())
}

fn state(
    directory: &DxfSplineScalarDirectory,
    raw: u64,
    role: DxfSplineValueRole,
) -> Result<DxfSplineScalarState, io::Error> {
    directory
        .entry_for_role(raw, role)
        .map(DxfSplineScalarEntry::state)
        .ok_or(io::Error::other("scalar state"))
}

fn assert_default(state: DxfSplineScalarState, expected: f64) {
    assert_eq!(
        state,
        DxfSplineScalarState::Defaulted(DxfSplineNumber::Double(
            seacad_dxf_core::DxfDouble::from_f64(expected)
        ))
    );
}

fn states(directory: &DxfSplineScalarDirectory) -> Result<Vec<StateEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            let state = match entry.state() {
                DxfSplineScalarState::Absent => (0, None, None, 0),
                DxfSplineScalarState::Defaulted(value) => (1, Some(value), None, 0),
                DxfSplineScalarState::Multiple { occurrence_count } => {
                    (2, None, None, occurrence_count)
                }
                DxfSplineScalarState::Invalid { issue, .. } => (3, None, Some(issue), 0),
                DxfSplineScalarState::Explicit(value) => (
                    4,
                    Some(
                        value
                            .value()
                            .map_err(|_| io::Error::other("explicit scalar"))?,
                    ),
                    None,
                    0,
                ),
                _ => return Err(io::Error::other("unknown scalar state")),
            };
            Ok((entry.role(), state.0, state.1, state.2, state.3))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nSPLINE\n71\n3\n72\n2\n73\n2\n74\n1\n42\n0.0000002\n0\nENDSEC\n0\nEOF\n"
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
        (0, b"SPLINE"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(71, 3), (72, 2), (73, 2), (74, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 42, 2.0e-7)?;
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
