use std::error::Error;

use seacad_dxf_core::{
    DXF_ENTITY_ALIAS_SCHEMA_SHA256, DXF_ENTITY_ALIASES, DXF_ENTITY_APPLICABILITY,
    DXF_ENTITY_APPLICABILITY_SCHEMA_SHA256, DXF_ENTITY_TOPIC_SCHEMA_SHA256, DXF_ENTITY_TOPICS,
    DxfAcadVersion, DxfEntityAlias, DxfEntityAliasEvidence, DxfEntityApplicability,
    DxfEntityApplicabilityEvidence, DxfEntityNameClassification, DxfEntityTopic,
    classify_exact_dxf_entity_name, dxf_entity_aliases, dxf_entity_applicability,
    dxf_entity_topics,
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

#[test]
fn reviewed_aliases_map_to_canonical_topics_with_distinct_evidence() {
    let aliases = dxf_entity_aliases();
    assert_eq!(aliases, DXF_ENTITY_ALIASES);
    assert_eq!(aliases.len(), 14);
    assert_eq!(DXF_ENTITY_ALIAS_SCHEMA_SHA256.len(), 64);

    for (ordinal, descriptor) in aliases.iter().enumerate() {
        let ordinal = u8::try_from(ordinal).unwrap_or(u8::MAX);
        assert_eq!(descriptor.alias().ordinal(), ordinal);
        assert_eq!(
            DxfEntityAlias::from_ordinal(ordinal),
            Some(descriptor.alias())
        );
        assert_eq!(
            DxfEntityAlias::from_exact_name(descriptor.dxf_name().as_bytes()),
            Some(descriptor.alias())
        );
        assert_eq!(descriptor.source_facts_sha256().len(), 64);
        assert_eq!(descriptor.alias().descriptor(), Some(descriptor));
    }
    assert_eq!(DxfEntityAlias::from_ordinal(14), None);

    assert_eq!(
        DxfEntityAlias::MPOLYGON.descriptor().map(|value| (
            value.topic(),
            value.evidence(),
            value.source_reference()
        )),
        Some((
            DxfEntityTopic::HATCH,
            DxfEntityAliasEvidence::Normative,
            "GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B"
        ))
    );
    assert_eq!(
        DxfEntityAlias::MULTILEADER.descriptor().map(|value| (
            value.topic(),
            value.evidence(),
            value.source_id()
        )),
        Some((
            DxfEntityTopic::MLEADER,
            DxfEntityAliasEvidence::BehavioralOracle,
            "autocad.entity.inventory.2027"
        ))
    );
}

#[test]
fn exact_name_classification_is_case_sensitive_and_explicitly_unknown() {
    let canonical = classify_exact_dxf_entity_name(b"SPLINE");
    assert_eq!(
        canonical,
        DxfEntityNameClassification::Canonical(DxfEntityTopic::SPLINE)
    );
    assert_eq!(canonical.topic(), Some(DxfEntityTopic::SPLINE));
    assert_eq!(canonical.exact_name(), Some("SPLINE"));

    let alias = classify_exact_dxf_entity_name(b"PDFUNDERLAY");
    assert_eq!(
        alias,
        DxfEntityNameClassification::Alias(DxfEntityAlias::PDFUNDERLAY)
    );
    assert_eq!(alias.topic(), Some(DxfEntityTopic::UNDERLAY));
    assert_eq!(alias.exact_name(), Some("PDFUNDERLAY"));

    for unknown in [
        b"pdfunderlay".as_slice(),
        b"TABLE ".as_slice(),
        b"".as_slice(),
    ] {
        let classification = classify_exact_dxf_entity_name(unknown);
        assert_eq!(classification, DxfEntityNameClassification::Unknown);
        assert_eq!(classification.topic(), None);
        assert_eq!(classification.exact_name(), None);
    }
}

#[test]
fn applicability_matrix_covers_every_reviewed_name_and_supported_dialect() {
    let matrix = dxf_entity_applicability();
    assert_eq!(matrix, DXF_ENTITY_APPLICABILITY);
    assert_eq!(
        matrix.len(),
        DXF_ENTITY_TOPICS.len() + DXF_ENTITY_ALIASES.len()
    );
    assert_eq!(DXF_ENTITY_APPLICABILITY_SCHEMA_SHA256.len(), 64);

    for descriptor in matrix {
        assert_eq!(
            descriptor.classification().applicability_descriptor(),
            Some(descriptor)
        );
        for version in DxfAcadVersion::SUPPORTED {
            assert_eq!(
                descriptor.classification().applicability(version),
                Some(descriptor.applicability(version))
            );
        }
    }
    assert_eq!(
        DxfEntityNameClassification::Unknown.applicability(DxfAcadVersion::Ac1032),
        None
    );
}

#[test]
fn official_underlay_ranges_are_typed_without_inventing_unreviewed_floors()
-> Result<(), Box<dyn Error>> {
    let dgn = classify_exact_dxf_entity_name(b"DGNUNDERLAY")
        .applicability_descriptor()
        .ok_or("reviewed DGN alias is missing its applicability row")?;
    assert_eq!(dgn.minimum_version(), Some(DxfAcadVersion::Ac1021));
    assert_eq!(dgn.maximum_version(), None);
    assert_eq!(
        dgn.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        dgn.source_reference(),
        Some("GUID-BF215599-C96C-4FFF-A2DB-21DEFFAC71C0")
    );
    assert_eq!(
        dgn.applicability(DxfAcadVersion::Ac1018),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        dgn.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::Applicable
    );

    let pdf = classify_exact_dxf_entity_name(b"PDFUNDERLAY")
        .applicability_descriptor()
        .ok_or("reviewed PDF alias is missing its applicability row")?;
    assert_eq!(pdf.minimum_version(), Some(DxfAcadVersion::Ac1024));
    assert_eq!(
        pdf.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        pdf.applicability(DxfAcadVersion::Ac1032),
        DxfEntityApplicability::Applicable
    );

    let spline = classify_exact_dxf_entity_name(b"SPLINE")
        .applicability_descriptor()
        .ok_or("canonical SPLINE topic is missing its applicability row")?;
    assert_eq!(spline.minimum_version(), None);
    assert_eq!(
        spline.evidence(),
        DxfEntityApplicabilityEvidence::NotYetReviewed
    );
    for version in DxfAcadVersion::SUPPORTED {
        assert_eq!(
            spline.applicability(version),
            DxfEntityApplicability::NotYetReviewed
        );
    }
    Ok(())
}

fn assert_send_sync<T: Send + Sync>() {}

fn assert_copy<T: Copy>() {}
