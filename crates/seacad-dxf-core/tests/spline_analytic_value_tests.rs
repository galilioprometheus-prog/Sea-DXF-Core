use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSplineAnalyticValueDirectory, DxfSplineAnalyticValueEntry, DxfSplineAnalyticValueIssueKind,
    DxfSplineAnalyticValueState, NoopDxfReadObserver,
};

type ValueEvidence = (Vec<u64>, Vec<([u64; 3], bool, u64)>, Vec<([u64; 3], bool)>);

#[test]
fn every_dialect_has_ascii_binary_analytic_value_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_values =
            ascii.spline_analytic_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_values =
            binary.spline_analytic_value_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_values)?;
        assert_expected(&binary_values)?;
        assert_eq!(evidence(&ascii_values)?, evidence(&binary_values)?);
    }
    Ok(())
}

#[test]
fn invalid_components_and_weights_accumulate_without_partial_values() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n40\n.\n10\n1\n11\n2\n41\n0\n\
0\nSPLINE\n10\n0\n20\n0\n10\n1\n20\n1\n41\n2\n\
0\nSPLINE\n10\n0\n20\n0\n41\n.\n\
0\nSPLINE\n10\n0\n20\n0\n30\n.\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);

    let DxfSplineAnalyticValueState::Unavailable(first) = directory.entries()[0].state else {
        return Err(io::Error::other("unavailable accumulated values").into());
    };
    for kind in [
        DxfSplineAnalyticValueIssueKind::Knot,
        DxfSplineAnalyticValueIssueKind::ControlPoint,
        DxfSplineAnalyticValueIssueKind::FitPoint,
        DxfSplineAnalyticValueIssueKind::Weight,
    ] {
        assert!(first.contains(kind));
    }

    let DxfSplineAnalyticValueState::Unavailable(second) = directory.entries()[1].state else {
        return Err(io::Error::other("unavailable count-mismatched weights").into());
    };
    assert!(second.contains(DxfSplineAnalyticValueIssueKind::Weight));
    assert!(!second.contains(DxfSplineAnalyticValueIssueKind::ControlPoint));

    let DxfSplineAnalyticValueState::Unavailable(third) = directory.entries()[2].state else {
        return Err(io::Error::other("unavailable invalid weight").into());
    };
    assert!(third.contains(DxfSplineAnalyticValueIssueKind::Weight));

    let DxfSplineAnalyticValueState::Unavailable(fourth) = directory.entries()[3].state else {
        return Err(io::Error::other("unavailable invalid z").into());
    };
    assert!(fourth.contains(DxfSplineAnalyticValueIssueKind::ControlPoint));
    assert!(!fourth.contains(DxfSplineAnalyticValueIssueKind::Weight));
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
        document.spline_analytic_value_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_analytic_value_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.spline_analytic_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert_copy::<DxfSplineAnalyticValueEntry>();
    assert_copy::<DxfSplineAnalyticValueState>();
    assert_send_sync::<DxfSplineAnalyticValueDirectory>();
    assert!(size_of::<DxfSplineAnalyticValueEntry>() <= 96);
    Ok(())
}

fn assert_expected(directory: &DxfSplineAnalyticValueDirectory) -> Result<(), Box<dyn Error>> {
    let entries = directory.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].ordinal, 0);
    assert_eq!(directory.entry(0), Some(entries[0]));
    let raw = entries[0].record.record().ordinal();
    assert_eq!(directory.entry_for_raw_record(raw), Some(entries[0]));

    let DxfSplineAnalyticValueState::Available {
        knot_range,
        control_point_range,
        fit_point_range,
    } = entries[0].state
    else {
        return Err(io::Error::other("first analytic values").into());
    };
    assert_eq!(knot_range.len(), 4);
    assert_eq!(control_point_range.len(), 2);
    assert_eq!(fit_point_range.len(), 1);
    let knots = directory
        .knots(knot_range)
        .ok_or(io::Error::other("knots"))?;
    assert_eq!(
        knots
            .iter()
            .map(|knot| knot.value.to_f64())
            .collect::<Vec<_>>(),
        [0.0, 0.0, 1.0, 1.0]
    );
    assert!(
        knots
            .iter()
            .all(|knot| { knot.evidence.group().group_code().value() == 40 })
    );
    let controls = directory
        .control_points(control_point_range)
        .ok_or(io::Error::other("controls"))?;
    assert_eq!(controls[0].weight.to_f64(), 1.0);
    assert_eq!(controls[1].weight.to_f64(), 1.0);
    assert!(controls[0].point.explicit_z);
    assert!(!controls[1].point.explicit_z);
    assert_eq!(controls[1].point.z.to_f64(), 0.0);
    let fits = directory
        .fit_points(fit_point_range)
        .ok_or(io::Error::other("fits"))?;
    assert_eq!(fits[0].x.to_f64(), 5.0);
    assert_eq!(fits[0].y.to_f64(), 6.0);
    assert!(!fits[0].explicit_z);

    let DxfSplineAnalyticValueState::Available {
        knot_range,
        control_point_range,
        fit_point_range,
    } = entries[1].state
    else {
        return Err(io::Error::other("second analytic values").into());
    };
    assert!(knot_range.is_empty());
    assert!(fit_point_range.is_empty());
    let controls = directory
        .control_points(control_point_range)
        .ok_or(io::Error::other("weighted controls"))?;
    assert_eq!(controls[0].weight.to_f64(), 2.0);
    assert_eq!(controls[1].weight.to_f64(), 0.5);
    Ok(())
}

fn evidence(directory: &DxfSplineAnalyticValueDirectory) -> Result<Vec<ValueEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            let DxfSplineAnalyticValueState::Available {
                knot_range,
                control_point_range,
                fit_point_range,
            } = entry.state
            else {
                return Err(io::Error::other("available evidence"));
            };
            let knots = directory
                .knots(knot_range)
                .ok_or(io::Error::other("knot range"))?
                .iter()
                .map(|knot| knot.value.to_bits())
                .collect();
            let controls = directory
                .control_points(control_point_range)
                .ok_or(io::Error::other("control range"))?
                .iter()
                .map(|control| {
                    (
                        point_bits(control.point),
                        control.point.explicit_z,
                        control.weight.to_bits(),
                    )
                })
                .collect();
            let fits = directory
                .fit_points(fit_point_range)
                .ok_or(io::Error::other("fit range"))?
                .iter()
                .map(|point| (point_bits(*point), point.explicit_z))
                .collect();
            Ok((knots, controls, fits))
        })
        .collect()
}

fn point_bits(point: seacad_dxf_core::DxfSplineAnalyticPoint) -> [u64; 3] {
    [point.x.to_bits(), point.y.to_bits(), point.z.to_bits()]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n40\n0\n40\n0\n40\n1\n40\n1\n10\n1\n10\n3\n20\n2\n20\n4\n30\n7\n11\n5\n21\n6\n\
0\nSPLINE\n10\n8\n10\n10\n20\n9\n20\n11\n41\n2\n41\n0.5\n\
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
    for (code, value) in [
        (40, 0.0),
        (40, 0.0),
        (40, 1.0),
        (40, 1.0),
        (10, 1.0),
        (10, 3.0),
        (20, 2.0),
        (20, 4.0),
        (30, 7.0),
        (11, 5.0),
        (21, 6.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"SPLINE")?;
    for (code, value) in [
        (10, 8.0),
        (10, 10.0),
        (20, 9.0),
        (20, 11.0),
        (41, 2.0),
        (41, 0.5),
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
