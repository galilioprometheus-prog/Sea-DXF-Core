use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineCountDirectory, DxfSplineCountDisposition,
    DxfSplineCountEntry, DxfSplineCountKind, DxfSplineCountState, NoopDxfReadObserver,
};

type CountEvidence = (
    DxfSplineCountKind,
    u8,
    i32,
    u32,
    Option<DxfSplineCountDisposition>,
);

#[test]
fn every_dialect_has_ascii_binary_count_relation_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_count_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_count_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(states(&ascii_directory)?, states(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn malformed_and_absent_counts_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n72\n1\n72\n2\n40\n0\n\
73\n-1\n10\n0\n10\n1\n74\n.\n11\n0\n0\nSPLINE\n40\n0\n10\n0\n11\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_count_directory(&DxfCancellationToken::default())?;
    let records = directory
        .scalar_directory()
        .card_directory()
        .evidence_directory()
        .records();
    assert_eq!(records.len(), 2);
    let first = records[0].record().ordinal();
    assert_eq!(
        state(&directory, first, DxfSplineCountKind::Knot)?,
        DxfSplineCountState::Multiple {
            occurrence_count: 2
        }
    );
    assert!(matches!(
        state(&directory, first, DxfSplineCountKind::ControlPoint)?,
        DxfSplineCountState::Negative {
            declared: -1,
            observed: 2,
            ..
        }
    ));
    assert!(matches!(
        state(&directory, first, DxfSplineCountKind::FitPoint)?,
        DxfSplineCountState::Invalid {
            issue: DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 },
            ..
        }
    ));

    let second = records[1].record().ordinal();
    for kind in [
        DxfSplineCountKind::Knot,
        DxfSplineCountKind::ControlPoint,
        DxfSplineCountKind::FitPoint,
    ] {
        assert_eq!(
            state(&directory, second, kind)?,
            DxfSplineCountState::Absent
        );
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
        document.spline_count_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.spline_count_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.entry_for_kind(u64::MAX, DxfSplineCountKind::Knot),
        None
    );
    assert_copy::<DxfSplineCountEntry>();
    assert_copy::<DxfSplineCountState>();
    assert_send_sync::<DxfSplineCountDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfSplineCountDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 3);
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );
    let record = directory
        .scalar_directory()
        .card_directory()
        .evidence_directory()
        .records()[0];
    let raw = record.record().ordinal();
    let entries = directory
        .entries_for_raw_record(raw)
        .ok_or(io::Error::other("count entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.kind()).collect::<Vec<_>>(),
        [
            DxfSplineCountKind::Knot,
            DxfSplineCountKind::ControlPoint,
            DxfSplineCountKind::FitPoint,
        ]
    );
    for entry in entries.iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
    }
    assert!(matches!(
        entries[0].state(),
        DxfSplineCountState::Compared {
            declared: 2,
            observed: 2,
            disposition: DxfSplineCountDisposition::Matched,
            ..
        }
    ));
    assert!(matches!(
        entries[1].state(),
        DxfSplineCountState::Compared {
            declared: 2,
            observed: 2,
            disposition: DxfSplineCountDisposition::Matched,
            ..
        }
    ));
    assert!(matches!(
        entries[2].state(),
        DxfSplineCountState::Compared {
            declared: 2,
            observed: 1,
            disposition: DxfSplineCountDisposition::Mismatched,
            ..
        }
    ));
    Ok(())
}

fn state(
    directory: &DxfSplineCountDirectory,
    raw: u64,
    kind: DxfSplineCountKind,
) -> Result<DxfSplineCountState, io::Error> {
    directory
        .entry_for_kind(raw, kind)
        .map(DxfSplineCountEntry::state)
        .ok_or(io::Error::other("count state"))
}

fn states(directory: &DxfSplineCountDirectory) -> Result<Vec<CountEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .map(|entry| match entry.state() {
            DxfSplineCountState::Compared {
                declared,
                observed,
                disposition,
                ..
            } => Ok((
                entry.kind(),
                0,
                i32::from(declared),
                observed,
                Some(disposition),
            )),
            DxfSplineCountState::Negative {
                declared, observed, ..
            } => Ok((entry.kind(), 1, i32::from(declared), observed, None)),
            DxfSplineCountState::Absent => Ok((entry.kind(), 2, 0, 0, None)),
            DxfSplineCountState::Multiple { occurrence_count } => {
                Ok((entry.kind(), 3, 0, occurrence_count, None))
            }
            DxfSplineCountState::Invalid { .. } => Ok((entry.kind(), 4, 0, 0, None)),
            _ => Err(io::Error::other("unknown count state")),
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n72\n2\n73\n2\n74\n2\n\
40\n0\n40\n1\n10\n0\n10\n1\n11\n0\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [(72, 2), (73, 2), (74, 2)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(40, 0.0), (40, 1.0), (10, 0.0), (10, 1.0), (11, 0.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
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
