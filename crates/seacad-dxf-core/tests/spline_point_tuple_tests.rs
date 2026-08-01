use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineNumber, DxfSplineNumericIssue,
    DxfSplinePointComponentCounts, DxfSplinePointComponents, DxfSplinePointKind,
    DxfSplinePointTuple, DxfSplinePointTupleDirectory, DxfSplinePointTupleEntry,
    DxfSplinePointTupleState, NoopDxfReadObserver,
};

type TupleEvidence = (
    DxfSplinePointKind,
    u64,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

#[test]
fn every_dialect_has_order_independent_ascii_binary_tuple_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.spline_point_tuple_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.spline_point_tuple_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn partial_and_invalid_components_remain_exact_and_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n20\n1\n20\n2\n10\n.\n\
0\nSPLINE\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_point_tuple_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);
    assert_eq!(directory.tuples().len(), 2);

    let first_record = directory.entries()[0].record().record().ordinal();
    let control = directory
        .tuples_for_raw_record(first_record, DxfSplinePointKind::ControlPoint)
        .ok_or(io::Error::other("control tuples"))?;
    assert_eq!(control.len(), 2);
    assert!(matches!(
        member_value(&directory, control[0].x_member())?
            .ok_or(io::Error::other("invalid x"))?
            .value(),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    ));
    let DxfSplinePointTupleState::Partial(first_components) = control[0].state() else {
        return Err(io::Error::other("expected partial tuple").into());
    };
    assert!(first_components.has_x());
    assert!(first_components.has_y());
    assert!(!first_components.has_z());

    let DxfSplinePointTupleState::Partial(second_components) = control[1].state() else {
        return Err(io::Error::other("expected y-only tuple").into());
    };
    assert!(!second_components.has_x());
    assert!(second_components.has_y());
    assert!(!second_components.has_z());
    assert_eq!(
        double(&directory, control[1].y_member())?,
        Some(2.0_f64.to_bits())
    );

    let second_record = directory.entries()[2].record().record().ordinal();
    for kind in [
        DxfSplinePointKind::ControlPoint,
        DxfSplinePointKind::FitPoint,
    ] {
        let entry = directory
            .entry_for_kind(second_record, kind)
            .ok_or(io::Error::other("empty entry"))?;
        assert!(entry.tuple_range().is_empty());
        assert_eq!(entry.component_counts().maximum(), 0);
    }
    Ok(())
}

#[test]
fn cancellation_bounds_and_public_traits_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.spline_point_tuple_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_point_tuple_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.spline_point_tuple_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.tuple(u64::MAX), None);
    assert_eq!(directory.entries_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.entry_for_kind(u64::MAX, DxfSplinePointKind::ControlPoint),
        None
    );
    assert_eq!(
        directory.tuples_for_raw_record(u64::MAX, DxfSplinePointKind::FitPoint),
        None
    );
    assert_copy::<DxfSplinePointComponents>();
    assert_copy::<DxfSplinePointComponentCounts>();
    assert_copy::<DxfSplinePointTupleState>();
    assert_copy::<DxfSplinePointTuple>();
    assert_copy::<DxfSplinePointTupleEntry>();
    assert_send_sync::<DxfSplinePointTupleDirectory>();
    assert!(size_of::<DxfSplinePointTuple>() <= 256);
    assert!(size_of::<DxfSplinePointTupleEntry>() <= 80);
    Ok(())
}

fn assert_directory(directory: &DxfSplinePointTupleDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(directory.tuples().len(), 4);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    let raw = directory.entries()[0].record().record().ordinal();
    let entries = directory
        .entries_for_raw_record(raw)
        .ok_or(io::Error::other("point entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.kind()).collect::<Vec<_>>(),
        [
            DxfSplinePointKind::ControlPoint,
            DxfSplinePointKind::FitPoint,
        ]
    );
    assert_counts(entries[0].component_counts(), 2, 2, 1);
    assert_counts(entries[1].component_counts(), 2, 1, 1);
    for entry in entries.iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert_eq!(
            directory
                .tuples_for_entry(entry.ordinal())
                .ok_or(io::Error::other("entry tuples"))?
                .len() as u64,
            entry.tuple_range().len()
        );
    }

    let control = directory
        .tuples_for_raw_record(raw, DxfSplinePointKind::ControlPoint)
        .ok_or(io::Error::other("control tuples"))?;
    assert_eq!(control[0].state(), DxfSplinePointTupleState::Complete);
    assert!(matches!(
        control[1].state(),
        DxfSplinePointTupleState::Partial(components)
            if components.has_x() && components.has_y() && !components.has_z()
    ));
    assert!(
        member_value(directory, control[0].x_member())?
            .ok_or(io::Error::other("control x"))?
            .group()
            .occurrence()
            > member_value(directory, control[0].y_member())?
                .ok_or(io::Error::other("control y"))?
                .group()
                .occurrence()
    );

    let fit = directory
        .tuples_for_raw_record(raw, DxfSplinePointKind::FitPoint)
        .ok_or(io::Error::other("fit tuples"))?;
    assert_eq!(fit[0].state(), DxfSplinePointTupleState::Complete);
    assert!(matches!(
        fit[1].state(),
        DxfSplinePointTupleState::Partial(components)
            if components.has_x() && !components.has_y() && !components.has_z()
    ));
    assert!(
        member_value(directory, fit[0].z_member())?
            .ok_or(io::Error::other("fit z"))?
            .group()
            .occurrence()
            < member_value(directory, fit[0].x_member())?
                .ok_or(io::Error::other("fit x"))?
                .group()
                .occurrence()
    );
    Ok(())
}

fn evidence(directory: &DxfSplinePointTupleDirectory) -> Result<Vec<TupleEvidence>, io::Error> {
    directory
        .tuples()
        .iter()
        .copied()
        .map(|tuple| {
            Ok((
                tuple.kind(),
                tuple.point_index(),
                double(directory, tuple.x_member())?,
                double(directory, tuple.y_member())?,
                double(directory, tuple.z_member())?,
            ))
        })
        .collect()
}

fn double(
    directory: &DxfSplinePointTupleDirectory,
    member: Option<seacad_dxf_core::DxfSplineCardMember>,
) -> Result<Option<u64>, io::Error> {
    member_value(directory, member)?
        .map(|value| match value.value() {
            Ok(DxfSplineNumber::Double(number)) => Ok(number.to_bits()),
            _ => Err(io::Error::other("expected double")),
        })
        .transpose()
}

fn member_value(
    directory: &DxfSplinePointTupleDirectory,
    member: Option<seacad_dxf_core::DxfSplineCardMember>,
) -> Result<Option<seacad_dxf_core::DxfSplineValue>, io::Error> {
    member
        .map(|member| {
            directory
                .card_directory()
                .value_for_member(member)
                .ok_or(io::Error::other("tuple member value"))
        })
        .transpose()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n20\n2\n20\n5\n30\n3\n\
10\n1\n10\n4\n31\n9\n11\n7\n21\n8\n11\n10\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [
        (20, 2.0),
        (20, 5.0),
        (30, 3.0),
        (10, 1.0),
        (10, 4.0),
        (31, 9.0),
        (11, 7.0),
        (21, 8.0),
        (11, 10.0),
    ] {
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
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        if self.armed.load(Ordering::Acquire) {
            self.token.cancel();
        }
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

fn assert_counts(counts: DxfSplinePointComponentCounts, x: u64, y: u64, z: u64) {
    assert_eq!((counts.x(), counts.y(), counts.z()), (x, y, z));
}
