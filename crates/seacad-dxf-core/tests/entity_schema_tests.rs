use std::error::Error;

use seacad_dxf_core::{
    DXF_ENTITY_ALIAS_SCHEMA_SHA256, DXF_ENTITY_ALIASES, DXF_ENTITY_APPLICABILITY,
    DXF_ENTITY_APPLICABILITY_SCHEMA_SHA256, DXF_ENTITY_COMMON_FIELD_SCHEMA_SHA256,
    DXF_ENTITY_COMMON_FIELDS, DXF_ENTITY_TOPIC_SCHEMA_SHA256, DXF_ENTITY_TOPICS, DxfAcadVersion,
    DxfEntityAlias, DxfEntityAliasEvidence, DxfEntityApplicability, DxfEntityApplicabilityEvidence,
    DxfEntityCoordinateSpace, DxfEntityField, DxfEntityFieldApplicability,
    DxfEntityFieldCardinality, DxfEntityFieldDefault, DxfEntityFieldScope, DxfEntityFieldWireType,
    DxfEntityFieldWriteOrder, DxfEntityNameClassification, DxfEntityTopic,
    classify_exact_dxf_entity_name, dxf_entity_aliases, dxf_entity_applicability,
    dxf_entity_common_fields, dxf_entity_topics,
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
fn official_reviewed_ranges_are_typed_without_inventing_unreviewed_floors()
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

    let acad_table = classify_exact_dxf_entity_name(b"ACAD_TABLE")
        .applicability_descriptor()
        .ok_or("reviewed ACAD_TABLE alias is missing its applicability row")?;
    assert_eq!(acad_table.minimum_version(), Some(DxfAcadVersion::Ac1018));
    assert_eq!(acad_table.maximum_version(), None);
    assert_eq!(
        acad_table.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        acad_table.source_reference(),
        Some("GUID-4570302D-8416-402B-902C-5948068B4B7E")
    );
    assert_eq!(
        acad_table.applicability(DxfAcadVersion::Ac1015),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        acad_table.applicability(DxfAcadVersion::Ac1018),
        DxfEntityApplicability::Applicable
    );

    let table_topic = classify_exact_dxf_entity_name(b"TABLE")
        .applicability_descriptor()
        .ok_or("reviewed TABLE topic is missing its applicability row")?;
    assert_eq!(table_topic.minimum_version(), None);
    assert_eq!(
        table_topic.evidence(),
        DxfEntityApplicabilityEvidence::NotYetReviewed
    );

    let helix = classify_exact_dxf_entity_name(b"HELIX")
        .applicability_descriptor()
        .ok_or("reviewed HELIX topic is missing its applicability row")?;
    assert_eq!(helix.minimum_version(), Some(DxfAcadVersion::Ac1021));
    assert_eq!(helix.maximum_version(), None);
    assert_eq!(
        helix.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        helix.source_reference(),
        Some("GUID-CC6BE90C-5ABE-4DE5-9390-B36FDCFF798B")
    );
    assert_eq!(
        helix.applicability(DxfAcadVersion::Ac1018),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        helix.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::Applicable
    );

    let light = classify_exact_dxf_entity_name(b"LIGHT")
        .applicability_descriptor()
        .ok_or("reviewed LIGHT topic is missing its applicability row")?;
    assert_eq!(light.minimum_version(), Some(DxfAcadVersion::Ac1021));
    assert_eq!(light.maximum_version(), None);
    assert_eq!(
        light.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        light.source_reference(),
        Some("GUID-CE870800-C598-483B-81A0-5AA0208F1851")
    );
    assert_eq!(
        light.applicability(DxfAcadVersion::Ac1018),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        light.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::Applicable
    );

    let lwpolyline = classify_exact_dxf_entity_name(b"LWPOLYLINE")
        .applicability_descriptor()
        .ok_or("reviewed LWPOLYLINE topic is missing its applicability row")?;
    assert_eq!(lwpolyline.minimum_version(), Some(DxfAcadVersion::Ac1014));
    assert_eq!(lwpolyline.maximum_version(), None);
    assert_eq!(
        lwpolyline.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        lwpolyline.source_reference(),
        Some("GUID-0A3004D1-1BF6-468A-9F69-4D0BA88857F2")
    );
    assert_eq!(
        lwpolyline.applicability(DxfAcadVersion::Ac1012),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        lwpolyline.applicability(DxfAcadVersion::Ac1014),
        DxfEntityApplicability::Applicable
    );

    let mesh = classify_exact_dxf_entity_name(b"MESH")
        .applicability_descriptor()
        .ok_or("reviewed MESH topic is missing its applicability row")?;
    assert_eq!(mesh.minimum_version(), Some(DxfAcadVersion::Ac1024));
    assert_eq!(mesh.maximum_version(), None);
    assert_eq!(
        mesh.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        mesh.source_reference(),
        Some("GUID-73981F72-60DD-46E7-BED1-BAF9692490A5")
    );
    assert_eq!(
        mesh.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        mesh.applicability(DxfAcadVersion::Ac1024),
        DxfEntityApplicability::Applicable
    );

    let mleader = classify_exact_dxf_entity_name(b"MLEADER")
        .applicability_descriptor()
        .ok_or("reviewed MLEADER topic is missing its applicability row")?;
    assert_eq!(mleader.minimum_version(), Some(DxfAcadVersion::Ac1021));
    assert_eq!(mleader.maximum_version(), None);
    assert_eq!(
        mleader.evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        mleader.source_reference(),
        Some("GUID-CE870800-C598-483B-81A0-5AA0208F1851")
    );
    assert_eq!(
        mleader.applicability(DxfAcadVersion::Ac1018),
        DxfEntityApplicability::NotApplicable
    );
    assert_eq!(
        mleader.applicability(DxfAcadVersion::Ac1021),
        DxfEntityApplicability::Applicable
    );

    let multileader = classify_exact_dxf_entity_name(b"MULTILEADER")
        .applicability_descriptor()
        .ok_or("reviewed MULTILEADER alias is missing its applicability row")?;
    assert_eq!(multileader.minimum_version(), None);
    assert_eq!(
        multileader.evidence(),
        DxfEntityApplicabilityEvidence::NotYetReviewed
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

#[test]
fn common_field_registry_freezes_group_wire_cardinality_and_provenance()
-> Result<(), Box<dyn Error>> {
    let fields = dxf_entity_common_fields();
    assert_eq!(fields, DXF_ENTITY_COMMON_FIELDS);
    assert_eq!(fields.len(), 19);
    assert_eq!(DXF_ENTITY_COMMON_FIELD_SCHEMA_SHA256.len(), 64);
    let expected = [
        ("handle", 5, DxfEntityFieldWireType::Handle, 0),
        ("owner", 330, DxfEntityFieldWireType::Handle, 2),
        (
            "extension_dictionary",
            360,
            DxfEntityFieldWireType::Handle,
            1,
        ),
        ("paper_space", 67, DxfEntityFieldWireType::Int16, 3),
        ("layout", 410, DxfEntityFieldWireType::ExactText, 4),
        ("layer", 8, DxfEntityFieldWireType::ExactText, 5),
        ("linetype", 6, DxfEntityFieldWireType::ExactText, 6),
        ("material", 347, DxfEntityFieldWireType::Handle, 7),
        ("color", 62, DxfEntityFieldWireType::Int16, 8),
        ("lineweight", 370, DxfEntityFieldWireType::Int16, 9),
        ("linetype_scale", 48, DxfEntityFieldWireType::Double, 10),
        ("visibility", 60, DxfEntityFieldWireType::Int16, 11),
        ("proxy_graphics_size", 92, DxfEntityFieldWireType::Int32, 12),
        (
            "proxy_graphics_data",
            310,
            DxfEntityFieldWireType::BinaryChunk,
            13,
        ),
        ("true_color", 420, DxfEntityFieldWireType::Int32, 14),
        ("color_name", 430, DxfEntityFieldWireType::ExactText, 15),
        ("transparency", 440, DxfEntityFieldWireType::Int32, 16),
        ("plot_style", 390, DxfEntityFieldWireType::Handle, 17),
        ("shadow", 284, DxfEntityFieldWireType::Int16, 18),
    ];
    for (ordinal, (descriptor, (id, group_code, wire_type, write_order))) in
        fields.iter().zip(expected).enumerate()
    {
        let ordinal = u8::try_from(ordinal)?;
        assert_eq!(descriptor.field().ordinal(), ordinal);
        assert_eq!(descriptor.id(), id);
        assert_eq!(descriptor.group_code(), group_code);
        assert_eq!(descriptor.wire_type(), wire_type);
        assert_eq!(descriptor.write_order().ordinal(), write_order);
        assert_eq!(
            DxfEntityField::from_ordinal(ordinal),
            Some(descriptor.field())
        );
        assert_eq!(
            DxfEntityField::from_group_code(group_code),
            Some(descriptor.field())
        );
        assert_eq!(descriptor.field().descriptor(), Some(descriptor));
        assert_eq!(
            descriptor.coordinate_space(),
            DxfEntityCoordinateSpace::NotApplicable
        );
        assert_eq!(
            descriptor.applicability(),
            DxfEntityFieldApplicability::NotYetReviewed
        );
        assert_eq!(descriptor.source_id(), "autodesk.common_entity_codes.2024");
        assert_eq!(
            descriptor.source_reference(),
            "GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD"
        );
        assert_eq!(descriptor.source_facts_sha256().len(), 64);
        assert!(
            descriptor
                .source_fact()
                .starts_with(&format!("group:{group_code}"))
        );
    }
    assert_eq!(DxfEntityField::from_ordinal(19), None);
    assert_eq!(DxfEntityField::from_group_code(999), None);
    assert_copy::<DxfEntityFieldWriteOrder>();
    assert_send_sync::<DxfEntityFieldWriteOrder>();
    Ok(())
}

#[test]
fn common_field_defaults_scopes_and_sequence_shape_are_explicit() {
    assert_eq!(
        DxfEntityField::LINETYPE
            .descriptor()
            .map(|descriptor| descriptor.default()),
        Some(DxfEntityFieldDefault::ExactText("BYLAYER"))
    );
    assert_eq!(
        DxfEntityField::MATERIAL
            .descriptor()
            .map(|descriptor| descriptor.default()),
        Some(DxfEntityFieldDefault::ByLayer)
    );
    assert_eq!(
        DxfEntityField::COLOR
            .descriptor()
            .map(|descriptor| descriptor.default()),
        Some(DxfEntityFieldDefault::Int16(256))
    );
    assert_eq!(
        DxfEntityField::LINETYPE_SCALE
            .descriptor()
            .map(|descriptor| descriptor.default()),
        Some(DxfEntityFieldDefault::DoubleBits(1.0_f64.to_bits()))
    );
    assert_eq!(
        DxfEntityField::PROXY_GRAPHICS_DATA
            .descriptor()
            .map(|descriptor| (descriptor.cardinality(), descriptor.scope())),
        Some((
            DxfEntityFieldCardinality::OptionalSequence,
            DxfEntityFieldScope::AcDbEntity
        ))
    );
    assert_eq!(
        DxfEntityField::EXTENSION_DICTIONARY
            .descriptor()
            .map(|descriptor| descriptor.scope()),
        Some(DxfEntityFieldScope::ExtensionDictionaryApplicationGroup)
    );
    assert_send_sync::<DxfEntityField>();
    assert_copy::<DxfEntityField>();
}

fn assert_send_sync<T: Send + Sync>() {}

fn assert_copy<T: Copy>() {}
