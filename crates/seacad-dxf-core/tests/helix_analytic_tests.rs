use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHelixAnalyticData, DxfHelixAnalyticDirectory,
    DxfHelixAnalyticEntry, DxfHelixAnalyticIssueKind, DxfHelixAnalyticState, DxfHelixAxisRelation,
    DxfHelixHeightRelation, DxfHelixRadiusDomain, DxfHelixSubclassPathState, DxfHelixTurnsDomain,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSplineRecordKind, NoopDxfReadObserver,
};

type AnalyticEvidence = (u16, i16, u64, u64, [u64; 2], bool, bool, bool, bool);

#[test]
fn every_dialect_composes_embedded_spline_with_helix_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.helix_analytic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.helix_analytic_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_directory)?;
        assert_expected(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn subclass_spline_and_helix_relation_failures_accumulate_without_partial_data()
-> Result<(), Box<dyn Error>> {
    let valid_helix = "10\n0\n20\n0\n30\n0\n11\n3\n21\n0\n31\n0\n\
12\n0\n22\n0\n32\n1\n40\n2\n41\n4\n42\n0.5\n";
    let valid_spline = "70\n0\n71\n2\n72\n6\n73\n3\n74\n0\n\
40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
10\n0\n20\n0\n10\n1\n20\n2\n10\n3\n20\n0\n";
    let bytes = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nHELIX\n100\nAcDbHelix\n{valid_helix}\
0\nHELIX\n100\nAcDbHelix\n{valid_helix}100\nAcDbSpline\n{valid_spline}\
0\nHELIX\n100\nAcDbSpline\n70\n0\n71\n2\n72\n6\n73\n3\n40\n0\n\
100\nAcDbHelix\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n\
12\n0\n22\n0\n32\n0\n40\n-1\n41\n0\n42\n0\n\
0\nHELIX\n100\nAcDbSpline\n{valid_spline}100\nAcDbSpline\n100\nAcDbHelix\n{valid_helix}\
0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.helix_analytic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);

    let first = unavailable(directory.entries()[0])?;
    assert!(first.contains(DxfHelixAnalyticIssueKind::SubclassPath));
    assert!(first.contains(DxfHelixAnalyticIssueKind::EmbeddedSpline));
    assert!(!first.contains(DxfHelixAnalyticIssueKind::Axis));
    assert_eq!(
        directory.entries()[0].subclass_path(),
        DxfHelixSubclassPathState::Unavailable {
            spline_marker_count: 0,
            helix_marker_count: 1,
            first_spline_marker_occurrence: None,
            first_helix_marker_occurrence: Some(8),
        }
    );

    let second = unavailable(directory.entries()[1])?;
    assert!(second.contains(DxfHelixAnalyticIssueKind::SubclassPath));
    assert!(!second.contains(DxfHelixAnalyticIssueKind::EmbeddedSpline));
    let DxfHelixSubclassPathState::Unavailable {
        spline_marker_count: 1,
        helix_marker_count: 1,
        first_spline_marker_occurrence: Some(spline),
        first_helix_marker_occurrence: Some(helix),
    } = directory.entries()[1].subclass_path()
    else {
        return Err(io::Error::other("reversed subclass state").into());
    };
    assert!(helix < spline);

    let third = unavailable(directory.entries()[2])?;
    for issue in [
        DxfHelixAnalyticIssueKind::EmbeddedSpline,
        DxfHelixAnalyticIssueKind::Axis,
        DxfHelixAnalyticIssueKind::Radius,
        DxfHelixAnalyticIssueKind::Turns,
    ] {
        assert!(third.contains(issue), "missing {issue:?}");
    }
    assert!(!third.contains(DxfHelixAnalyticIssueKind::SubclassPath));
    assert!(!third.contains(DxfHelixAnalyticIssueKind::Height));

    let fourth = unavailable(directory.entries()[3])?;
    assert!(fourth.contains(DxfHelixAnalyticIssueKind::SubclassPath));
    assert!(!fourth.contains(DxfHelixAnalyticIssueKind::EmbeddedSpline));
    assert!(matches!(
        directory.entries()[3].subclass_path(),
        DxfHelixSubclassPathState::Unavailable {
            spline_marker_count: 2,
            helix_marker_count: 1,
            ..
        }
    ));
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_analytic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_analytic_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.helix_analytic_directory(&DxfCancellationToken::default())?;
    let entry = directory.entries()[0];
    let raw = entry.record().entity().record().ordinal();
    assert_eq!(directory.entry(0), Some(entry));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_raw_record(raw), Some(entry));
    assert_eq!(directory.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.relation_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.spline_directory().source_id()
    );
    assert_copy::<DxfHelixAnalyticData>();
    assert_copy::<DxfHelixAnalyticEntry>();
    assert_copy::<DxfHelixAnalyticState>();
    assert_send_sync::<DxfHelixAnalyticDirectory>();
    let entry_size = size_of::<DxfHelixAnalyticEntry>();
    assert!(entry_size <= 512, "analytic entry size: {entry_size}");
    Ok(())
}

fn assert_expected(directory: &DxfHelixAnalyticDirectory) -> Result<(), Box<dyn Error>> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one HELIX analytic entry").into());
    };
    let raw = entry.record().entity().record().ordinal();
    let spline = directory
        .spline_directory()
        .entry_for_raw_record(raw)
        .ok_or(io::Error::other("embedded spline entry"))?;
    assert_eq!(spline.record.kind(), DxfSplineRecordKind::Helix);
    assert!(matches!(
        entry.subclass_path(),
        DxfHelixSubclassPathState::Ordered { .. }
    ));
    let DxfHelixAnalyticState::Available = entry.state() else {
        return Err(io::Error::other("available HELIX analytic data").into());
    };
    let data = entry
        .data()
        .ok_or(io::Error::other("available HELIX analytic payload"))?;
    assert_eq!(data.embedded_spline.degree, 2);
    assert_eq!(data.embedded_spline.knot_range.len(), 6);
    assert_eq!(data.embedded_spline.control_point_range.len(), 3);
    assert!(matches!(
        data.axis,
        DxfHelixAxisRelation::Compared {
            exactly_perpendicular: true,
            ..
        }
    ));
    assert!(matches!(
        data.radius,
        DxfHelixRadiusDomain::NonNegative { .. }
    ));
    assert!(matches!(
        data.turns,
        DxfHelixTurnsDomain::WithinCommandLimit { .. }
    ));
    assert!(matches!(
        data.height,
        DxfHelixHeightRelation::Compared { flat: false, .. }
    ));
    Ok(())
}

fn unavailable(
    entry: DxfHelixAnalyticEntry,
) -> Result<seacad_dxf_core::DxfHelixAnalyticIssues, io::Error> {
    if entry.data().is_some() {
        return Err(io::Error::other("partial unavailable payload"));
    }
    match entry.state() {
        DxfHelixAnalyticState::Unavailable(issues) => Ok(issues),
        DxfHelixAnalyticState::Available => Err(io::Error::other("unavailable state")),
        _ => Err(io::Error::other("unknown analytic state")),
    }
}

fn evidence(directory: &DxfHelixAnalyticDirectory) -> Result<Vec<AnalyticEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| {
            let DxfHelixAnalyticState::Available = entry.state() else {
                return Err(io::Error::other("available analytic evidence"));
            };
            let data = entry
                .data()
                .ok_or(io::Error::other("available analytic payload"))?;
            Ok((
                data.embedded_spline.degree,
                data.embedded_spline.flags.raw(),
                data.embedded_spline.knot_range.len(),
                data.embedded_spline.control_point_range.len(),
                [
                    data.embedded_spline.parameter_start.to_bits(),
                    data.embedded_spline.parameter_end.to_bits(),
                ],
                matches!(data.axis, DxfHelixAxisRelation::Compared { .. }),
                matches!(data.radius, DxfHelixRadiusDomain::NonNegative { .. }),
                matches!(data.turns, DxfHelixTurnsDomain::WithinCommandLimit { .. }),
                matches!(data.height, DxfHelixHeightRelation::Compared { .. }),
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbEntity\n100\nAcDbSpline\n\
70\n0\n71\n2\n72\n6\n73\n3\n74\n0\n\
40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
10\n0\n20\n0\n10\n1\n20\n2\n10\n3\n20\n0\n\
100\nAcDbHelix\n90\n1\n91\n0\n10\n0\n20\n0\n30\n0\n\
11\n3\n21\n0\n31\n0\n12\n0\n22\n0\n32\n1\n40\n2\n41\n4\n42\n0.5\n\
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
        (0, b"HELIX"),
        (100, b"AcDbEntity"),
        (100, b"AcDbSpline"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 0), (71, 2), (72, 6), (73, 3), (74, 0)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for value in [0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y) in [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
    }
    push_string(&mut bytes, version, 100, b"AcDbHelix")?;
    push_i32(&mut bytes, version, 90, 1)?;
    push_i32(&mut bytes, version, 91, 0)?;
    for (code, value) in [
        (10, 0.0),
        (20, 0.0),
        (30, 0.0),
        (11, 3.0),
        (21, 0.0),
        (31, 0.0),
        (12, 0.0),
        (22, 0.0),
        (32, 1.0),
        (40, 2.0),
        (41, 4.0),
        (42, 0.5),
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
