use std::error::Error;

use seacad_dxf_core::{
    DXF_ENTITY_TOPIC_SCHEMA_SHA256, DXF_ENTITY_TOPICS, DxfEntityTopic, dxf_entity_topics,
};

const EXPECTED_DXF_NAMES: [&str; 45] = [
    "3DFACE",
    "3DSOLID",
    "ACAD_PROXY_ENTITY",
    "ARC",
    "ATTDEF",
    "ATTRIB",
    "BODY",
    "CIRCLE",
    "DIMENSION",
    "ELLIPSE",
    "HATCH",
    "HELIX",
    "IMAGE",
    "INSERT",
    "LEADER",
    "LIGHT",
    "LINE",
    "LWPOLYLINE",
    "MESH",
    "MLINE",
    "MLEADERSTYLE",
    "MLEADER",
    "MTEXT",
    "OLEFRAME",
    "OLE2FRAME",
    "POINT",
    "POLYLINE",
    "RAY",
    "REGION",
    "SECTION",
    "SEQEND",
    "SHAPE",
    "SOLID",
    "SPLINE",
    "SUN",
    "SURFACE",
    "TABLE",
    "TEXT",
    "TOLERANCE",
    "TRACE",
    "UNDERLAY",
    "VERTEX",
    "VIEWPORT",
    "WIPEOUT",
    "XLINE",
];

#[test]
fn canonical_inventory_has_exact_reviewed_order_and_provenance() -> Result<(), Box<dyn Error>> {
    let topics = dxf_entity_topics();
    assert_eq!(topics, DXF_ENTITY_TOPICS);
    assert_eq!(topics.len(), EXPECTED_DXF_NAMES.len());
    assert_eq!(DXF_ENTITY_TOPIC_SCHEMA_SHA256.len(), 64);

    for (ordinal, (descriptor, expected_name)) in topics.iter().zip(EXPECTED_DXF_NAMES).enumerate()
    {
        let ordinal = u8::try_from(ordinal)?;
        assert_eq!(descriptor.topic().ordinal(), ordinal);
        assert_eq!(descriptor.dxf_name(), expected_name);
        assert_eq!(
            DxfEntityTopic::from_ordinal(ordinal),
            Some(descriptor.topic())
        );
        assert_eq!(
            DxfEntityTopic::from_exact_name(expected_name.as_bytes()),
            Some(descriptor.topic())
        );
        assert_eq!(descriptor.source_id(), "autodesk.entities.2024");
        assert_eq!(
            descriptor.source_topic_id(),
            "GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484"
        );
        assert_eq!(descriptor.source_facts_sha256().len(), 64);
        assert_eq!(descriptor.topic().descriptor(), Some(descriptor));
    }
    assert_eq!(DxfEntityTopic::from_ordinal(45), None);
    Ok(())
}

#[test]
fn exact_name_lookup_is_case_sensitive_and_alias_neutral() {
    assert_eq!(
        DxfEntityTopic::from_exact_name(b"SPLINE"),
        Some(DxfEntityTopic::SPLINE)
    );
    for unreviewed in [
        b"spline".as_slice(),
        b"Spline".as_slice(),
        b"MULTILEADER".as_slice(),
        b"PDFUNDERLAY".as_slice(),
        b"UNKNOWN_FUTURE_ENTITY".as_slice(),
        b"".as_slice(),
    ] {
        assert_eq!(DxfEntityTopic::from_exact_name(unreviewed), None);
    }
}

#[test]
fn public_topics_have_stable_traits_and_ids() {
    assert_send_sync::<DxfEntityTopic>();
    assert_copy::<DxfEntityTopic>();
    assert_eq!(
        DxfEntityTopic::THREE_D_FACE
            .descriptor()
            .map(|value| value.id()),
        Some("three_d_face")
    );
    assert_eq!(
        DxfEntityTopic::XLINE.descriptor().map(|value| value.id()),
        Some("xline")
    );
}

fn assert_send_sync<T: Send + Sync>() {}

fn assert_copy<T: Copy>() {}
