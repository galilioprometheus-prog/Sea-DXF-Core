use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundarySplineEdgePointGrouping,
    DxfHatchBoundarySplineEdgePointKind, DxfHatchBoundarySplineEdgePointMemberRole,
    DxfHatchBoundarySplineEdgePointOrphan, DxfHatchBoundarySplineEdgePointTuple,
    DxfHatchBoundarySplineEdgePointTupleDirectory, DxfHatchBoundarySplineEdgePointTupleEntry,
    DxfHatchBoundarySplineEdgePointTupleIssue, DxfHatchBoundarySplineEdgeSequenceIssue,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type TupleEvidence = (
    DxfHatchBoundarySplineEdgePointKind,
    u64,
    u64,
    u64,
    Vec<(DxfHatchBoundarySplineEdgePointMemberRole, i16)>,
);

#[test]
fn every_dialect_groups_control_and_fit_points_with_exact_members() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n4\n94\n2\n73\n1\n74\n0\n95\n2\n96\n2\n\
             40\n0\n40\n1\n\
             10\n0\n20\n0\n42\n1\n10\n1\n20\n1\n\
             97\n2\n11\n0.5\n21\n0.5\n11\n0.75\n21\n0.75\n\
             12\n1\n22\n0\n13\n0\n23\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_point_tuple_directory(&DxfCancellationToken::default())?;

        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_point_tuple_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_complete_grouping(directory)?;
        }
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn orphan_and_duplicate_components_remain_in_source_order() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n92\n0\n93\n2\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n2\n\
         20\n8\n42\n9\n10\n1\n20\n2\n20\n3\n42\n4\n42\n5\n10\n6\n\
         97\n1\n21\n7\n11\n8\n21\n9\n21\n10\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_tuple_directory(&DxfCancellationToken::default())?;
    let [entry, empty_entry] = directory.entries() else {
        return Err(io::Error::other("two retained Spline edges").into());
    };
    let grouping = available(*entry)?;
    assert_eq!(grouping.control_point_count(), 2);
    assert_eq!(grouping.control_orphan_count(), 2);
    assert_eq!(grouping.fit_point_count(), 1);
    assert_eq!(grouping.fit_orphan_count(), 1);

    let edge = entry
        .sequence_entry()
        .payload_partition_entry()
        .edge_type_entry()
        .edge();
    let control = directory
        .tuples_for_edge(
            edge.ordinal(),
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
        )
        .ok_or_else(|| io::Error::other("control tuples"))?;
    assert_eq!((control[0].y_count(), control[0].weight_count()), (2, 2));
    assert_eq!((control[1].y_count(), control[1].weight_count()), (0, 0));
    assert_eq!(
        roles(
            directory
                .members_for_tuple(control[0].ordinal())
                .unwrap_or_default()
        ),
        [
            DxfHatchBoundarySplineEdgePointMemberRole::X,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
            DxfHatchBoundarySplineEdgePointMemberRole::Weight,
            DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        ]
    );

    let control_orphans = directory
        .orphans_for_edge(
            edge.ordinal(),
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
        )
        .ok_or_else(|| io::Error::other("control orphans"))?;
    assert_eq!(
        control_orphans
            .iter()
            .map(|orphan| orphan.role())
            .collect::<Vec<_>>(),
        [
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
            DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        ]
    );
    assert!(
        control_orphans[0].field().group().occurrence() < control[0].x_field().group().occurrence()
    );

    let fit = directory
        .tuples_for_edge(
            edge.ordinal(),
            DxfHatchBoundarySplineEdgePointKind::FitPoint,
        )
        .ok_or_else(|| io::Error::other("fit tuples"))?;
    assert_eq!(fit.len(), 1);
    assert_eq!((fit[0].y_count(), fit[0].weight_count()), (2, 0));
    assert_eq!(
        roles(
            directory
                .members_for_tuple(fit[0].ordinal())
                .unwrap_or_default()
        ),
        [
            DxfHatchBoundarySplineEdgePointMemberRole::X,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
        ]
    );
    assert_eq!(
        directory
            .orphans_for_edge(
                edge.ordinal(),
                DxfHatchBoundarySplineEdgePointKind::FitPoint
            )
            .unwrap_or_default()[0]
            .role(),
        DxfHatchBoundarySplineEdgePointMemberRole::Y
    );

    let empty_grouping = available(*empty_entry)?;
    assert_eq!(
        (
            empty_grouping.control_point_count(),
            empty_grouping.control_orphan_count(),
            empty_grouping.fit_point_count(),
            empty_grouping.fit_orphan_count(),
        ),
        (0, 0, 0, 0)
    );
    let empty_edge = empty_entry
        .sequence_entry()
        .payload_partition_entry()
        .edge_type_entry()
        .edge();
    assert!(
        directory
            .tuples_for_edge(
                empty_edge.ordinal(),
                DxfHatchBoundarySplineEdgePointKind::ControlPoint,
            )
            .is_some_and(<[_]>::is_empty)
    );
    assert!(
        directory
            .orphans_for_edge(
                empty_edge.ordinal(),
                DxfHatchBoundarySplineEdgePointKind::FitPoint,
            )
            .is_some_and(<[_]>::is_empty)
    );
    Ok(())
}

#[test]
fn unavailable_sequences_stay_typed_and_non_spline_edges_stay_excluded()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n2\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n41\n9\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_tuple_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one Spline entry").into());
    };
    assert_eq!(
        entry
            .sequence_entry()
            .payload_partition_entry()
            .edge_type_entry()
            .edge()
            .ordinal(),
        1
    );
    assert!(matches!(
        entry.grouping(),
        Err(DxfHatchBoundarySplineEdgePointTupleIssue::SequenceUnavailable(
            DxfHatchBoundarySplineEdgeSequenceIssue::UnexpectedField { group }
        )) if group.group_code().value() == 41
    ));
    assert!(directory.tuples().is_empty());
    assert!(directory.members().is_empty());
    assert!(directory.orphans().is_empty());
    assert!(
        directory
            .tuples_for_edge(1, DxfHatchBoundarySplineEdgePointKind::ControlPoint)
            .is_none()
    );
    assert_eq!(directory.entry_for_edge(0), None);
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n1\n\
         10\n12345.625\n20\n2\n42\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_point_tuple_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_spline_edge_point_tuple_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.sequence_directory().source_id()
    );
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.tuple(0), Some(directory.tuples()[0]));
    assert_eq!(directory.tuple(u64::MAX), None);
    assert_eq!(directory.orphan(u64::MAX), None);
    assert!(directory.members_for_tuple(u64::MAX).is_none());
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(
        directory
            .orphans_for_edge(u64::MAX, DxfHatchBoundarySplineEdgePointKind::FitPoint)
            .is_none()
    );
    assert!(size_of::<DxfHatchBoundarySplineEdgePointTuple>() <= 128);
    assert!(size_of::<DxfHatchBoundarySplineEdgePointTupleEntry>() <= 512);
    assert!(!format!("{:?}", directory.tuples()[0]).contains("12345.625"));
    copy::<DxfHatchBoundarySplineEdgePointKind>();
    copy::<DxfHatchBoundarySplineEdgePointMemberRole>();
    copy::<DxfHatchBoundarySplineEdgePointGrouping>();
    copy::<DxfHatchBoundarySplineEdgePointTupleIssue>();
    copy::<DxfHatchBoundarySplineEdgePointTuple>();
    copy::<DxfHatchBoundarySplineEdgePointOrphan>();
    copy::<DxfHatchBoundarySplineEdgePointTupleEntry>();
    send_sync::<DxfHatchBoundarySplineEdgePointTupleDirectory>();
    Ok(())
}

fn assert_complete_grouping(
    directory: &DxfHatchBoundarySplineEdgePointTupleDirectory,
) -> Result<(), io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one Spline point entry"));
    };
    assert_eq!(entry.ordinal(), 0);
    let edge = entry
        .sequence_entry()
        .payload_partition_entry()
        .edge_type_entry()
        .edge();
    assert_eq!(edge.ordinal(), 1);
    let grouping = available(*entry)?;
    assert_eq!(grouping.control_point_count(), 2);
    assert_eq!(grouping.control_orphan_count(), 0);
    assert_eq!(grouping.fit_point_count(), 2);
    assert_eq!(grouping.fit_orphan_count(), 0);
    assert_eq!(directory.tuples().len(), 4);
    assert_eq!(directory.members().len(), 9);
    assert!(directory.orphans().is_empty());

    let control = directory
        .tuples_for_edge(
            edge.ordinal(),
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
        )
        .ok_or_else(|| io::Error::other("control tuples"))?;
    assert_eq!(
        (
            control[0].point_index(),
            control[0].y_count(),
            control[0].weight_count()
        ),
        (0, 1, 1)
    );
    assert_eq!(
        (
            control[1].point_index(),
            control[1].y_count(),
            control[1].weight_count()
        ),
        (1, 1, 0)
    );
    let fit = directory
        .tuples_for_edge(
            edge.ordinal(),
            DxfHatchBoundarySplineEdgePointKind::FitPoint,
        )
        .ok_or_else(|| io::Error::other("fit tuples"))?;
    assert_eq!(
        (
            fit[0].point_index(),
            fit[0].y_count(),
            fit[0].weight_count()
        ),
        (0, 1, 0)
    );
    assert_eq!(
        (
            fit[1].point_index(),
            fit[1].y_count(),
            fit[1].weight_count()
        ),
        (1, 1, 0)
    );
    for tuple in directory.tuples().iter().copied() {
        assert_eq!(directory.tuple(tuple.ordinal()), Some(tuple));
        let members = directory
            .members_for_tuple(tuple.ordinal())
            .ok_or_else(|| io::Error::other("tuple members"))?;
        assert_eq!(
            members[0].role(),
            DxfHatchBoundarySplineEdgePointMemberRole::X
        );
        assert_eq!(members[0].field(), tuple.x_field());
        assert!(members.windows(2).all(|pair| {
            pair[0].field().group().occurrence() < pair[1].field().group().occurrence()
        }));
    }
    Ok(())
}

fn evidence(
    directory: &DxfHatchBoundarySplineEdgePointTupleDirectory,
) -> Result<Vec<TupleEvidence>, io::Error> {
    directory
        .tuples()
        .iter()
        .copied()
        .map(|tuple| {
            let members = directory
                .members_for_tuple(tuple.ordinal())
                .ok_or_else(|| io::Error::other("tuple members"))?;
            Ok((
                tuple.kind(),
                tuple.point_index(),
                tuple.y_count(),
                tuple.weight_count(),
                members
                    .iter()
                    .map(|member| (member.role(), member.field().group().group_code().value()))
                    .collect(),
            ))
        })
        .collect()
}

fn available(
    entry: DxfHatchBoundarySplineEdgePointTupleEntry,
) -> Result<DxfHatchBoundarySplineEdgePointGrouping, io::Error> {
    entry
        .grouping()
        .map_err(|_| io::Error::other("available Spline edge point grouping"))
}

fn roles(
    members: &[seacad_dxf_core::DxfHatchBoundarySplineEdgePointTupleMember],
) -> Vec<DxfHatchBoundarySplineEdgePointMemberRole> {
    members.iter().map(|member| member.role()).collect()
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
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
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, 2)?;
    push_i16(&mut bytes, version, 72, 1)?;
    for (code, value) in [(10, 1.0), (20, 2.0), (11, 3.0), (21, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 2)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 2)?;
    push_i32(&mut bytes, version, 96, 2)?;
    for value in [0.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y, weight) in [(0.0, 0.0, Some(1.0)), (1.0, 1.0, None)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
        if let Some(weight) = weight {
            push_double(&mut bytes, version, 42, weight)?;
        }
    }
    push_i32(&mut bytes, version, 97, 2)?;
    for (x, y) in [(0.5, 0.5), (0.75, 0.75)] {
        push_double(&mut bytes, version, 11, x)?;
        push_double(&mut bytes, version, 21, y)?;
    }
    for (code, value) in [(12, 1.0), (22, 0.0), (13, 0.0), (23, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 97, 0)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn copy<T: Copy>() {}

fn send_sync<T: Send + Sync>() {}
