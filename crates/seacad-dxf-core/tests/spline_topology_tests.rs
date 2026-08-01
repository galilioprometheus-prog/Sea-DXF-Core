use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSplineDegreeControlRelation, DxfSplineDegreeState,
    DxfSplineInvariantDisposition, DxfSplineKnotOrderState, DxfSplineNurbsCountRelation,
    DxfSplinePeriodicClosedRelation, DxfSplineTopologyDirectory, DxfSplineTopologyEntry,
    NoopDxfReadObserver,
};

type TopologyEvidence = (
    (u8, i32),
    DxfSplineKnotOrderState,
    DxfSplineDegreeControlRelation,
    DxfSplineNurbsCountRelation,
    DxfSplinePeriodicClosedRelation,
);

#[test]
fn every_dialect_has_ascii_binary_topology_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_topology = ascii.spline_topology_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_topology = binary.spline_topology_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_topology)?;
        assert_expected(&binary_topology)?;
        assert_eq!(evidence(&ascii_topology)?, evidence(&binary_topology)?);
    }
    Ok(())
}

#[test]
fn degree_and_knot_failures_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n\
0\nSPLINE\n71\n1\n71\n2\n\
0\nSPLINE\n71\n.\n\
0\nSPLINE\n71\n0\n\
0\nSPLINE\n71\n-1\n\
0\nSPLINE\n70\n0\n71\n3\n10\n0\n10\n1\n40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
0\nSPLINE\n70\n0\n71\n2\n10\n0\n10\n1\n10\n2\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
0\nSPLINE\n70\n0\n71\n1\n10\n0\n10\n1\n40\n0\n40\n.\n40\n1\n40\n1\n\
0\nSPLINE\n70\n0\n71\n1\n10\n0\n10\n1\n40\n0\n40\n1\n40\n0.5\n40\n1\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let topology = document.spline_topology_directory(&DxfCancellationToken::default())?;
    let entries = topology.entries();
    assert_eq!(entries.len(), 9);

    assert_eq!(entries[0].degree(), DxfSplineDegreeState::Absent);
    assert_eq!(entries[0].knot_order(), DxfSplineKnotOrderState::Empty);
    assert_eq!(
        entries[0].periodic_closed(),
        DxfSplinePeriodicClosedRelation::UnavailableFlags
    );
    assert!(matches!(
        entries[1].degree(),
        DxfSplineDegreeState::Multiple {
            occurrence_count: 2
        }
    ));
    assert!(matches!(
        entries[2].degree(),
        DxfSplineDegreeState::Invalid {
            issue: DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 },
            ..
        }
    ));
    assert!(matches!(
        entries[3].degree(),
        DxfSplineDegreeState::NonPositive { degree: 0, .. }
    ));
    assert!(matches!(
        entries[4].degree(),
        DxfSplineDegreeState::NonPositive { degree: -1, .. }
    ));

    assert_eq!(
        entries[5].degree_control(),
        DxfSplineDegreeControlRelation::Compared {
            degree: 3,
            control_point_count: 2,
            minimum_control_point_count: 4,
            disposition: DxfSplineInvariantDisposition::Contradictory
        }
    );
    assert!(matches!(
        entries[5].nurbs_count(),
        DxfSplineNurbsCountRelation::Compared {
            expected_knot_count: 6,
            disposition: DxfSplineInvariantDisposition::Satisfied,
            ..
        }
    ));
    assert!(matches!(
        entries[6].nurbs_count(),
        DxfSplineNurbsCountRelation::Compared {
            knot_count: 5,
            expected_knot_count: 6,
            disposition: DxfSplineInvariantDisposition::Contradictory,
            ..
        }
    ));
    assert_eq!(
        entries[7].knot_order(),
        DxfSplineKnotOrderState::Unavailable {
            knot_count: 4,
            invalid_count: 1
        }
    );
    assert_eq!(
        entries[8].knot_order(),
        DxfSplineKnotOrderState::Decreasing {
            knot_count: 4,
            first_decrease_index: 2
        }
    );
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
        document.spline_topology_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.spline_topology_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let topology = document.spline_topology_directory(&DxfCancellationToken::default())?;
    assert_eq!(topology.entry(u64::MAX), None);
    assert_eq!(topology.entry_for_raw_record(u64::MAX), None);
    assert_eq!(
        topology.source_id(),
        topology.relation_directory().source_id()
    );
    assert_copy::<DxfSplineTopologyEntry>();
    assert_copy::<DxfSplineDegreeState>();
    assert_copy::<DxfSplineKnotOrderState>();
    assert_copy::<DxfSplineNurbsCountRelation>();
    assert_send_sync::<DxfSplineTopologyDirectory>();
    assert!(size_of::<DxfSplineTopologyEntry>() <= 200);
    Ok(())
}

fn assert_expected(directory: &DxfSplineTopologyDirectory) -> Result<(), Box<dyn Error>> {
    let entries = directory.entries();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].ordinal(), 0);
    assert_eq!(directory.entry(0), Some(entries[0]));
    let raw = entries[0].record().record().ordinal();
    assert_eq!(directory.entry_for_raw_record(raw), Some(entries[0]));
    assert!(matches!(
        entries[0].degree(),
        DxfSplineDegreeState::Explicit { degree: 3, .. }
    ));
    assert_eq!(
        entries[0].knot_order(),
        DxfSplineKnotOrderState::Nondecreasing { knot_count: 8 }
    );
    assert!(matches!(
        entries[0].degree_control(),
        DxfSplineDegreeControlRelation::Compared {
            minimum_control_point_count: 4,
            disposition: DxfSplineInvariantDisposition::Satisfied,
            ..
        }
    ));
    assert!(matches!(
        entries[0].nurbs_count(),
        DxfSplineNurbsCountRelation::Compared {
            expected_knot_count: 8,
            disposition: DxfSplineInvariantDisposition::Satisfied,
            ..
        }
    ));
    assert_eq!(
        entries[0].periodic_closed(),
        DxfSplinePeriodicClosedRelation::Open
    );

    assert_eq!(
        entries[1].knot_order(),
        DxfSplineKnotOrderState::Decreasing {
            knot_count: 6,
            first_decrease_index: 3
        }
    );
    assert_eq!(
        entries[1].periodic_closed(),
        DxfSplinePeriodicClosedRelation::NonPeriodicClosed
    );
    assert_eq!(
        entries[2].periodic_closed(),
        DxfSplinePeriodicClosedRelation::PeriodicClosed
    );
    assert_eq!(
        entries[3].periodic_closed(),
        DxfSplinePeriodicClosedRelation::PeriodicWithoutClosed
    );
    Ok(())
}

fn evidence(directory: &DxfSplineTopologyDirectory) -> Result<Vec<TopologyEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            Ok((
                degree_evidence(entry.degree())?,
                entry.knot_order(),
                entry.degree_control(),
                entry.nurbs_count(),
                entry.periodic_closed(),
            ))
        })
        .collect()
}

fn degree_evidence(state: DxfSplineDegreeState) -> Result<(u8, i32), io::Error> {
    match state {
        DxfSplineDegreeState::Absent => Ok((0, 0)),
        DxfSplineDegreeState::Multiple { occurrence_count } => Ok((1, occurrence_count as i32)),
        DxfSplineDegreeState::Invalid { .. } => Ok((2, 0)),
        DxfSplineDegreeState::NonPositive { degree, .. } => Ok((3, i32::from(degree))),
        DxfSplineDegreeState::Explicit { degree, .. } => Ok((4, i32::from(degree))),
        _ => Err(io::Error::other("unknown degree state")),
    }
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSPLINE\n70\n0\n71\n3\n10\n0\n10\n1\n10\n2\n10\n3\n40\n0\n40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n40\n1\n\
0\nSPLINE\n70\n1\n71\n2\n10\n0\n10\n1\n10\n2\n40\n0\n40\n0\n40\n1\n40\n0.5\n40\n1\n40\n1\n\
0\nSPLINE\n70\n3\n71\n1\n10\n0\n10\n1\n40\n0\n40\n0\n40\n1\n40\n1\n\
0\nSPLINE\n70\n2\n71\n1\n10\n0\n10\n1\n40\n0\n40\n0\n40\n1\n40\n1\n\
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
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_record(
        &mut bytes,
        version,
        0,
        3,
        &[0.0, 1.0, 2.0, 3.0],
        &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
    )?;
    push_record(
        &mut bytes,
        version,
        1,
        2,
        &[0.0, 1.0, 2.0],
        &[0.0, 0.0, 1.0, 0.5, 1.0, 1.0],
    )?;
    for flags in [3, 2] {
        push_record(
            &mut bytes,
            version,
            flags,
            1,
            &[0.0, 1.0],
            &[0.0, 0.0, 1.0, 1.0],
        )?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_record(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    flags: i16,
    degree: i16,
    controls: &[f64],
    knots: &[f64],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"SPLINE")?;
    push_i16(bytes, version, 70, flags)?;
    push_i16(bytes, version, 71, degree)?;
    for value in controls {
        push_double(bytes, version, 10, *value)?;
    }
    for value in knots {
        push_double(bytes, version, 40, *value)?;
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
