//! Family-specific construction of MTEXT and TOLERANCE scalar semantics.

use super::{
    DxfMTextNumericSemantics, DxfToleranceNumericSemantics, MTEXT_NAMESPACE, TOLERANCE_NAMESPACE,
};
use crate::{
    DxfDouble, DxfError, DxfTextSymbolCardDirectory, DxfTextSymbolDoubleValue,
    DxfTextSymbolInt16Value, DxfTextSymbolInt32Value, DxfTextSymbolRecordEntry,
    DxfTextSymbolValueRole,
    text_symbol_scalar_value::{DxfScalarRule, semantic_double, semantic_i16, semantic_i32},
};

const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());

pub(super) fn mtext_semantics(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
) -> Result<DxfMTextNumericSemantics, DxfError> {
    Ok(DxfMTextNumericSemantics {
        record,
        insertion: vector(
            cards,
            record,
            MTEXT_NAMESPACE,
            [
                DxfTextSymbolValueRole::InsertionX,
                DxfTextSymbolValueRole::InsertionY,
                DxfTextSymbolValueRole::InsertionZ,
            ],
            ["insertion_x", "insertion_y", "insertion_z"],
            DxfScalarRule::Required,
        )?,
        nominal_height: double(
            cards,
            record,
            DxfTextSymbolValueRole::NominalHeight,
            MTEXT_NAMESPACE,
            "nominal_height",
            DxfScalarRule::Required,
        )?,
        reference_width: double(
            cards,
            record,
            DxfTextSymbolValueRole::ReferenceWidth,
            MTEXT_NAMESPACE,
            "reference_width",
            DxfScalarRule::Required,
        )?,
        attachment: i16_value(
            cards,
            record,
            DxfTextSymbolValueRole::Attachment,
            MTEXT_NAMESPACE,
            "attachment",
            DxfScalarRule::Required,
        )?,
        drawing_direction: i16_value(
            cards,
            record,
            DxfTextSymbolValueRole::DrawingDirection,
            MTEXT_NAMESPACE,
            "drawing_direction",
            DxfScalarRule::Required,
        )?,
        extrusion: extrusion(cards, record, MTEXT_NAMESPACE)?,
        x_axis: vector(
            cards,
            record,
            MTEXT_NAMESPACE,
            [
                DxfTextSymbolValueRole::XAxisX,
                DxfTextSymbolValueRole::XAxisY,
                DxfTextSymbolValueRole::XAxisZ,
            ],
            ["x_axis_x", "x_axis_y", "x_axis_z"],
            DxfScalarRule::Optional,
        )?,
        actual_width: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::ActualWidth,
            MTEXT_NAMESPACE,
            "actual_width",
        )?,
        actual_height: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::ActualHeight,
            MTEXT_NAMESPACE,
            "actual_height",
        )?,
        line_spacing_style: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::LineSpacingStyle,
            MTEXT_NAMESPACE,
            "line_spacing_style",
        )?,
        line_spacing_factor: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::LineSpacingFactor,
            MTEXT_NAMESPACE,
            "line_spacing_factor",
        )?,
        background_fill: optional_i32(
            cards,
            record,
            DxfTextSymbolValueRole::BackgroundFill,
            MTEXT_NAMESPACE,
            "background_fill",
        )?,
        fill_box_scale: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::FillBoxScale,
            MTEXT_NAMESPACE,
            "fill_box_scale",
        )?,
        background_index: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::BackgroundIndex,
            MTEXT_NAMESPACE,
            "background_index",
        )?,
        background_transparency: optional_i32(
            cards,
            record,
            DxfTextSymbolValueRole::BackgroundTransparency,
            MTEXT_NAMESPACE,
            "background_transparency",
        )?,
        column_type: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnType,
            MTEXT_NAMESPACE,
            "column_type",
        )?,
        column_count: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnCount,
            MTEXT_NAMESPACE,
            "column_count",
        )?,
        column_flow_reversed: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnFlowReversed,
            MTEXT_NAMESPACE,
            "column_flow_reversed",
        )?,
        column_auto_height: optional_i16(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnAutoHeight,
            MTEXT_NAMESPACE,
            "column_auto_height",
        )?,
        column_width: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnWidth,
            MTEXT_NAMESPACE,
            "column_width",
        )?,
        column_gutter: optional_double(
            cards,
            record,
            DxfTextSymbolValueRole::ColumnGutter,
            MTEXT_NAMESPACE,
            "column_gutter",
        )?,
    })
}

pub(super) fn tolerance_semantics(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
) -> Result<DxfToleranceNumericSemantics, DxfError> {
    Ok(DxfToleranceNumericSemantics {
        record,
        insertion: vector(
            cards,
            record,
            TOLERANCE_NAMESPACE,
            [
                DxfTextSymbolValueRole::InsertionX,
                DxfTextSymbolValueRole::InsertionY,
                DxfTextSymbolValueRole::InsertionZ,
            ],
            ["insertion_x", "insertion_y", "insertion_z"],
            DxfScalarRule::Required,
        )?,
        extrusion: extrusion(cards, record, TOLERANCE_NAMESPACE)?,
        x_axis: vector(
            cards,
            record,
            TOLERANCE_NAMESPACE,
            [
                DxfTextSymbolValueRole::XAxisX,
                DxfTextSymbolValueRole::XAxisY,
                DxfTextSymbolValueRole::XAxisZ,
            ],
            ["x_axis_x", "x_axis_y", "x_axis_z"],
            DxfScalarRule::Required,
        )?,
    })
}

fn vector(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
    roles: [DxfTextSymbolValueRole; 3],
    fields: [&'static str; 3],
    rule: DxfScalarRule<DxfDouble>,
) -> Result<[DxfTextSymbolDoubleValue; 3], DxfError> {
    Ok([
        double(cards, record, roles[0], namespace, fields[0], rule)?,
        double(cards, record, roles[1], namespace, fields[1], rule)?,
        double(cards, record, roles[2], namespace, fields[2], rule)?,
    ])
}

fn extrusion(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
) -> Result<[DxfTextSymbolDoubleValue; 3], DxfError> {
    Ok([
        double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionX,
            namespace,
            "extrusion_x",
            DxfScalarRule::Defaulted(ZERO),
        )?,
        double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionY,
            namespace,
            "extrusion_y",
            DxfScalarRule::Defaulted(ZERO),
        )?,
        double(
            cards,
            record,
            DxfTextSymbolValueRole::ExtrusionZ,
            namespace,
            "extrusion_z",
            DxfScalarRule::Defaulted(ONE),
        )?,
    ])
}

fn optional_double(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
) -> Result<DxfTextSymbolDoubleValue, DxfError> {
    double(
        cards,
        record,
        role,
        namespace,
        field,
        DxfScalarRule::Optional,
    )
}

fn optional_i16(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
) -> Result<DxfTextSymbolInt16Value, DxfError> {
    i16_value(
        cards,
        record,
        role,
        namespace,
        field,
        DxfScalarRule::Optional,
    )
}

fn optional_i32(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
) -> Result<DxfTextSymbolInt32Value, DxfError> {
    semantic_i32(
        cards,
        record,
        role,
        namespace,
        field,
        DxfScalarRule::Optional,
    )
}

fn double(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
    rule: DxfScalarRule<DxfDouble>,
) -> Result<DxfTextSymbolDoubleValue, DxfError> {
    semantic_double(cards, record, role, namespace, field, rule)
}

fn i16_value(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field: &'static str,
    rule: DxfScalarRule<i16>,
) -> Result<DxfTextSymbolInt16Value, DxfError> {
    semantic_i16(cards, record, role, namespace, field, rule)
}
