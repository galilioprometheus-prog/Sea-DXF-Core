//! Family-specific construction of TEXT and SHAPE scalar semantics.

use super::{
    DxfShapeNumericSemantics, DxfTextNumericSemantics, DxfTextShapeDoubleValue,
    DxfTextShapeInt16Value, ONE, SHAPE_NAMESPACE, TEXT_NAMESPACE, ZERO,
};
use crate::{
    DxfDouble, DxfError, DxfTextSymbolCardDirectory, DxfTextSymbolRecordEntry,
    DxfTextSymbolValueRole,
    text_symbol_scalar_value::{DxfScalarRule, semantic_double, semantic_i16},
};

pub(super) fn text_semantics(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
) -> Result<DxfTextNumericSemantics, DxfError> {
    Ok(DxfTextNumericSemantics {
        record,
        first_alignment: required_vector(
            cards,
            record,
            TEXT_NAMESPACE,
            [
                DxfTextSymbolValueRole::FirstAlignmentX,
                DxfTextSymbolValueRole::FirstAlignmentY,
                DxfTextSymbolValueRole::FirstAlignmentZ,
            ],
            [
                "first_alignment_x",
                "first_alignment_y",
                "first_alignment_z",
            ],
        )?,
        text_height: required_double(
            cards,
            record,
            DxfTextSymbolValueRole::TextHeight,
            TEXT_NAMESPACE,
            "text_height",
        )?,
        thickness: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::Thickness,
            TEXT_NAMESPACE,
            "thickness",
            ZERO,
        )?,
        rotation: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::Rotation,
            TEXT_NAMESPACE,
            "rotation_degrees",
            ZERO,
        )?,
        width_factor: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::WidthFactor,
            TEXT_NAMESPACE,
            "width_factor",
            ONE,
        )?,
        oblique_angle: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::ObliqueAngle,
            TEXT_NAMESPACE,
            "oblique_angle_degrees",
            ZERO,
        )?,
        generation_flags: i16_value(
            cards,
            record,
            DxfTextSymbolValueRole::GenerationFlags,
            TEXT_NAMESPACE,
            "generation_flags",
            DxfScalarRule::Defaulted(0),
        )?,
        horizontal_justification: i16_value(
            cards,
            record,
            DxfTextSymbolValueRole::HorizontalJustification,
            TEXT_NAMESPACE,
            "horizontal_justification",
            DxfScalarRule::Defaulted(0),
        )?,
        second_alignment: optional_vector(
            cards,
            record,
            TEXT_NAMESPACE,
            [
                DxfTextSymbolValueRole::SecondAlignmentX,
                DxfTextSymbolValueRole::SecondAlignmentY,
                DxfTextSymbolValueRole::SecondAlignmentZ,
            ],
            [
                "second_alignment_x",
                "second_alignment_y",
                "second_alignment_z",
            ],
        )?,
        extrusion: defaulted_extrusion(cards, record, TEXT_NAMESPACE)?,
        vertical_justification: i16_value(
            cards,
            record,
            DxfTextSymbolValueRole::VerticalJustification,
            TEXT_NAMESPACE,
            "vertical_justification",
            DxfScalarRule::Defaulted(0),
        )?,
    })
}

pub(super) fn shape_semantics(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
) -> Result<DxfShapeNumericSemantics, DxfError> {
    Ok(DxfShapeNumericSemantics {
        record,
        insertion: required_vector(
            cards,
            record,
            SHAPE_NAMESPACE,
            [
                DxfTextSymbolValueRole::InsertionX,
                DxfTextSymbolValueRole::InsertionY,
                DxfTextSymbolValueRole::InsertionZ,
            ],
            ["insertion_x", "insertion_y", "insertion_z"],
        )?,
        size: required_double(
            cards,
            record,
            DxfTextSymbolValueRole::ShapeSize,
            SHAPE_NAMESPACE,
            "size",
        )?,
        thickness: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::Thickness,
            SHAPE_NAMESPACE,
            "thickness",
            ZERO,
        )?,
        rotation: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::Rotation,
            SHAPE_NAMESPACE,
            "rotation_degrees",
            ZERO,
        )?,
        width_factor: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::WidthFactor,
            SHAPE_NAMESPACE,
            "width_factor",
            ONE,
        )?,
        oblique_angle: defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::ObliqueAngle,
            SHAPE_NAMESPACE,
            "oblique_angle_degrees",
            ZERO,
        )?,
        extrusion: defaulted_extrusion(cards, record, SHAPE_NAMESPACE)?,
    })
}

fn required_vector(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
    roles: [DxfTextSymbolValueRole; 3],
    fields: [&'static str; 3],
) -> Result<[DxfTextShapeDoubleValue; 3], DxfError> {
    Ok([
        required_double(cards, record, roles[0], namespace, fields[0])?,
        required_double(cards, record, roles[1], namespace, fields[1])?,
        required_double(cards, record, roles[2], namespace, fields[2])?,
    ])
}

fn optional_vector(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
    roles: [DxfTextSymbolValueRole; 3],
    fields: [&'static str; 3],
) -> Result<[DxfTextShapeDoubleValue; 3], DxfError> {
    Ok([
        semantic_double(
            cards,
            record,
            roles[0],
            namespace,
            fields[0],
            DxfScalarRule::Optional,
        )?,
        semantic_double(
            cards,
            record,
            roles[1],
            namespace,
            fields[1],
            DxfScalarRule::Optional,
        )?,
        semantic_double(
            cards,
            record,
            roles[2],
            namespace,
            fields[2],
            DxfScalarRule::Optional,
        )?,
    ])
}

fn defaulted_extrusion(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
) -> Result<[DxfTextShapeDoubleValue; 3], DxfError> {
    Ok([
        defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionX,
            namespace,
            "extrusion_x",
            ZERO,
        )?,
        defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionY,
            namespace,
            "extrusion_y",
            ZERO,
        )?,
        defaulted_double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionZ,
            namespace,
            "extrusion_z",
            ONE,
        )?,
    ])
}

fn required_double(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
) -> Result<DxfTextShapeDoubleValue, DxfError> {
    semantic_double(
        cards,
        record,
        role,
        namespace,
        field,
        DxfScalarRule::Required,
    )
}

fn defaulted_double(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
    default: DxfDouble,
) -> Result<DxfTextShapeDoubleValue, DxfError> {
    semantic_double(
        cards,
        record,
        role,
        namespace,
        field,
        DxfScalarRule::Defaulted(default),
    )
}

fn i16_value(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
    rule: DxfScalarRule<i16>,
) -> Result<DxfTextShapeInt16Value, DxfError> {
    semantic_i16(cards, record, role, namespace, field, rule)
}
