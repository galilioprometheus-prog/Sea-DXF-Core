use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSplineAnalyticData, DxfSplineAnalyticDirectory, DxfSplineAnalyticEntry,
    DxfSplineAnalyticIssueKind, DxfSplineAnalyticState, DxfSplineVectorSemanticState,
    NoopDxfReadObserver,
};

type AnalyticEvidence = (u16, i16, u64, u64, u64, [u64; 2]);

#[test]
fn every_dialect_has_ascii_binary_analytic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_analytic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_analytic_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_directory)?;
        assert_expected(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn readiness_issues_accumulate_without_partial_analytic_data() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n10\n0\n20\n0\n40\n.\n\
0\nSPLINE\n70\n0\n71\n1\n72\n4\n73\n2\n10\n0\n20\n0\n10\n1\n20\n1\n40\n0\n40\n0\n40\n0\n40\n0\n\
0\nSPLINE\n70\n16\n71\n1\n72\n4\n73\n2\n10\n0\n20\n0\n10\n1\n20\n1\n40\n0\n40\n0\n40\n1\n40\n1\n22\n1\n\
0\nSPLINE\n70\n0\n70\n0\n71\n1\n71\n1\n72\n4\n73\n2\n10\n0\n20\n0\n10\n1\n20\n1\n40\n0\n40\n0\n40\n1\n40\n1\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);

    let DxfSplineAnalyticState::Unavailable(first) = directory.entries()[0].state else {
        return Err(io::Error::other("first unavailable analytic state").into());
    };
    for issue in [
        DxfSplineAnalyticIssueKind::Values,
        DxfSplineAnalyticIssueKind::Flags,
        DxfSplineAnalyticIssueKind::Degree,
        DxfSplineAnalyticIssueKind::KnotCount,
        DxfSplineAnalyticIssueKind::ControlPointCount,
        DxfSplineAnalyticIssueKind::KnotOrder,
        DxfSplineAnalyticIssueKind::DegreeControl,
        DxfSplineAnalyticIssueKind::NurbsCount,
        DxfSplineAnalyticIssueKind::Normal,
    ] {
        assert!(first.contains(issue), "missing {issue:?}");
    }

    let DxfSplineAnalyticState::Unavailable(second) = directory.entries()[1].state else {
        return Err(io::Error::other("second unavailable analytic state").into());
    };
    assert!(second.contains(DxfSplineAnalyticIssueKind::KnotMultiplicity));
    assert!(second.contains(DxfSplineAnalyticIssueKind::ParameterDomain));
    assert!(!second.contains(DxfSplineAnalyticIssueKind::Values));

    let DxfSplineAnalyticState::Unavailable(third) = directory.entries()[2].state else {
        return Err(io::Error::other("third unavailable analytic state").into());
    };
    assert!(third.contains(DxfSplineAnalyticIssueKind::LinearPlanar));
    assert!(third.contains(DxfSplineAnalyticIssueKind::OptionalVector));
    assert!(!third.contains(DxfSplineAnalyticIssueKind::ParameterDomain));

    let DxfSplineAnalyticState::Unavailable(fourth) = directory.entries()[3].state else {
        return Err(io::Error::other("fourth unavailable analytic state").into());
    };
    assert!(fourth.contains(DxfSplineAnalyticIssueKind::Flags));
    assert!(fourth.contains(DxfSplineAnalyticIssueKind::Degree));
    assert!(!fourth.contains(DxfSplineAnalyticIssueKind::Values));
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.spline_analytic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_analytic_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.value_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.topology_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.count_directory().source_id()
    );
    assert_copy::<DxfSplineAnalyticData>();
    assert_copy::<DxfSplineAnalyticEntry>();
    assert_copy::<DxfSplineAnalyticState>();
    assert_send_sync::<DxfSplineAnalyticDirectory>();
    let entry_size = size_of::<DxfSplineAnalyticEntry>();
    assert!(entry_size <= 256, "analytic entry size: {entry_size}");
    Ok(())
}

fn assert_expected(directory: &DxfSplineAnalyticDirectory) -> Result<(), Box<dyn Error>> {
    let entries = directory.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ordinal, 0);
    assert_eq!(directory.entry(0), Some(entries[0]));
    let raw = entries[0].record.record().ordinal();
    assert_eq!(directory.entry_for_raw_record(raw), Some(entries[0]));
    let DxfSplineAnalyticState::Available(data) = entries[0].state else {
        return Err(io::Error::other("available analytic data").into());
    };
    assert_eq!(data.degree, 2);
    assert_eq!(data.flags.raw(), 0);
    assert_eq!(data.parameter_start.to_f64(), 0.0);
    assert_eq!(data.parameter_end.to_f64(), 1.0);
    assert_eq!(data.knot_range.len(), 6);
    assert_eq!(data.control_point_range.len(), 3);
    assert!(data.fit_point_range.is_empty());
    assert_eq!(data.start_tangent, DxfSplineVectorSemanticState::Absent);
    assert_eq!(data.end_tangent, DxfSplineVectorSemanticState::Absent);
    assert_eq!(data.normal, DxfSplineVectorSemanticState::Absent);
    let knots = directory
        .value_directory()
        .knots(data.knot_range)
        .ok_or(io::Error::other("knot range"))?;
    assert_eq!(knots.len(), 6);
    let controls = directory
        .value_directory()
        .control_points(data.control_point_range)
        .ok_or(io::Error::other("control range"))?;
    assert_eq!(controls.len(), 3);
    assert!(controls.iter().all(|control| !control.point.explicit_z));
    Ok(())
}

fn evidence(directory: &DxfSplineAnalyticDirectory) -> Result<Vec<AnalyticEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            let DxfSplineAnalyticState::Available(data) = entry.state else {
                return Err(io::Error::other("available analytic evidence"));
            };
            Ok((
                data.degree,
                data.flags.raw(),
                data.knot_range.len(),
                data.control_point_range.len(),
                data.fit_point_range.len(),
                [data.parameter_start.to_bits(), data.parameter_end.to_bits()],
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n70\n0\n71\n2\n72\n6\n73\n3\n\
40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
10\n0\n20\n0\n10\n1\n20\n2\n10\n3\n20\n0\n\
0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [(70, 0), (71, 2), (72, 6), (73, 3)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for value in [0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y) in [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
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
