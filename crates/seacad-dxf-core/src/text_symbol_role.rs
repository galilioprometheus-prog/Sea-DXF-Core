//! Static Autodesk field-role registry for text-and-symbol records.

use crate::{DxfTextSymbolKind, DxfTextSymbolValueRole};

const TEXT_ROLES: [DxfTextSymbolValueRole; 19] = [
    DxfTextSymbolValueRole::Thickness,
    DxfTextSymbolValueRole::FirstAlignmentX,
    DxfTextSymbolValueRole::FirstAlignmentY,
    DxfTextSymbolValueRole::FirstAlignmentZ,
    DxfTextSymbolValueRole::TextHeight,
    DxfTextSymbolValueRole::Content,
    DxfTextSymbolValueRole::Rotation,
    DxfTextSymbolValueRole::WidthFactor,
    DxfTextSymbolValueRole::ObliqueAngle,
    DxfTextSymbolValueRole::StyleName,
    DxfTextSymbolValueRole::GenerationFlags,
    DxfTextSymbolValueRole::HorizontalJustification,
    DxfTextSymbolValueRole::SecondAlignmentX,
    DxfTextSymbolValueRole::SecondAlignmentY,
    DxfTextSymbolValueRole::SecondAlignmentZ,
    DxfTextSymbolValueRole::ExtrusionX,
    DxfTextSymbolValueRole::ExtrusionY,
    DxfTextSymbolValueRole::ExtrusionZ,
    DxfTextSymbolValueRole::VerticalJustification,
];

const MTEXT_ROLES: [DxfTextSymbolValueRole; 33] = [
    DxfTextSymbolValueRole::InsertionX,
    DxfTextSymbolValueRole::InsertionY,
    DxfTextSymbolValueRole::InsertionZ,
    DxfTextSymbolValueRole::NominalHeight,
    DxfTextSymbolValueRole::ReferenceWidth,
    DxfTextSymbolValueRole::Attachment,
    DxfTextSymbolValueRole::DrawingDirection,
    DxfTextSymbolValueRole::Content,
    DxfTextSymbolValueRole::AdditionalContent,
    DxfTextSymbolValueRole::StyleName,
    DxfTextSymbolValueRole::ExtrusionX,
    DxfTextSymbolValueRole::ExtrusionY,
    DxfTextSymbolValueRole::ExtrusionZ,
    DxfTextSymbolValueRole::XAxisX,
    DxfTextSymbolValueRole::XAxisY,
    DxfTextSymbolValueRole::XAxisZ,
    DxfTextSymbolValueRole::ActualWidth,
    DxfTextSymbolValueRole::ActualHeight,
    DxfTextSymbolValueRole::RotationOrColumnHeight,
    DxfTextSymbolValueRole::LineSpacingStyle,
    DxfTextSymbolValueRole::LineSpacingFactor,
    DxfTextSymbolValueRole::BackgroundFill,
    DxfTextSymbolValueRole::BackgroundRgbOrEntityTrueColor,
    DxfTextSymbolValueRole::BackgroundNameOrEntityColorName,
    DxfTextSymbolValueRole::FillBoxScale,
    DxfTextSymbolValueRole::BackgroundIndex,
    DxfTextSymbolValueRole::BackgroundTransparency,
    DxfTextSymbolValueRole::ColumnType,
    DxfTextSymbolValueRole::ColumnCount,
    DxfTextSymbolValueRole::ColumnFlowReversed,
    DxfTextSymbolValueRole::ColumnAutoHeight,
    DxfTextSymbolValueRole::ColumnWidth,
    DxfTextSymbolValueRole::ColumnGutter,
];

const SHAPE_ROLES: [DxfTextSymbolValueRole; 12] = [
    DxfTextSymbolValueRole::Thickness,
    DxfTextSymbolValueRole::InsertionX,
    DxfTextSymbolValueRole::InsertionY,
    DxfTextSymbolValueRole::InsertionZ,
    DxfTextSymbolValueRole::ShapeSize,
    DxfTextSymbolValueRole::ShapeName,
    DxfTextSymbolValueRole::Rotation,
    DxfTextSymbolValueRole::WidthFactor,
    DxfTextSymbolValueRole::ObliqueAngle,
    DxfTextSymbolValueRole::ExtrusionX,
    DxfTextSymbolValueRole::ExtrusionY,
    DxfTextSymbolValueRole::ExtrusionZ,
];

const TOLERANCE_ROLES: [DxfTextSymbolValueRole; 11] = [
    DxfTextSymbolValueRole::DimensionStyleName,
    DxfTextSymbolValueRole::InsertionX,
    DxfTextSymbolValueRole::InsertionY,
    DxfTextSymbolValueRole::InsertionZ,
    DxfTextSymbolValueRole::Content,
    DxfTextSymbolValueRole::ExtrusionX,
    DxfTextSymbolValueRole::ExtrusionY,
    DxfTextSymbolValueRole::ExtrusionZ,
    DxfTextSymbolValueRole::XAxisX,
    DxfTextSymbolValueRole::XAxisY,
    DxfTextSymbolValueRole::XAxisZ,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DxfTextSymbolWireType {
    Text,
    Double,
    Int16,
    Int32,
}

pub(crate) const fn value_role(
    kind: DxfTextSymbolKind,
    code: i16,
) -> Option<(DxfTextSymbolValueRole, DxfTextSymbolWireType)> {
    match kind {
        DxfTextSymbolKind::Text => text_role(code),
        DxfTextSymbolKind::MText => mtext_role(code),
        DxfTextSymbolKind::Shape => shape_role(code),
        DxfTextSymbolKind::Tolerance => tolerance_role(code),
    }
}

pub(crate) const fn roles_for_kind(kind: DxfTextSymbolKind) -> &'static [DxfTextSymbolValueRole] {
    match kind {
        DxfTextSymbolKind::Text => &TEXT_ROLES,
        DxfTextSymbolKind::MText => &MTEXT_ROLES,
        DxfTextSymbolKind::Shape => &SHAPE_ROLES,
        DxfTextSymbolKind::Tolerance => &TOLERANCE_ROLES,
    }
}

const fn text_role(code: i16) -> Option<(DxfTextSymbolValueRole, DxfTextSymbolWireType)> {
    use DxfTextSymbolValueRole as R;
    use DxfTextSymbolWireType as W;
    match code {
        39 => Some((R::Thickness, W::Double)),
        10 => Some((R::FirstAlignmentX, W::Double)),
        20 => Some((R::FirstAlignmentY, W::Double)),
        30 => Some((R::FirstAlignmentZ, W::Double)),
        40 => Some((R::TextHeight, W::Double)),
        1 => Some((R::Content, W::Text)),
        50 => Some((R::Rotation, W::Double)),
        41 => Some((R::WidthFactor, W::Double)),
        51 => Some((R::ObliqueAngle, W::Double)),
        7 => Some((R::StyleName, W::Text)),
        71 => Some((R::GenerationFlags, W::Int16)),
        72 => Some((R::HorizontalJustification, W::Int16)),
        11 => Some((R::SecondAlignmentX, W::Double)),
        21 => Some((R::SecondAlignmentY, W::Double)),
        31 => Some((R::SecondAlignmentZ, W::Double)),
        210 => Some((R::ExtrusionX, W::Double)),
        220 => Some((R::ExtrusionY, W::Double)),
        230 => Some((R::ExtrusionZ, W::Double)),
        73 => Some((R::VerticalJustification, W::Int16)),
        _ => None,
    }
}

const fn mtext_role(code: i16) -> Option<(DxfTextSymbolValueRole, DxfTextSymbolWireType)> {
    use DxfTextSymbolValueRole as R;
    use DxfTextSymbolWireType as W;
    match code {
        10 => Some((R::InsertionX, W::Double)),
        20 => Some((R::InsertionY, W::Double)),
        30 => Some((R::InsertionZ, W::Double)),
        40 => Some((R::NominalHeight, W::Double)),
        41 => Some((R::ReferenceWidth, W::Double)),
        71 => Some((R::Attachment, W::Int16)),
        72 => Some((R::DrawingDirection, W::Int16)),
        1 => Some((R::Content, W::Text)),
        3 => Some((R::AdditionalContent, W::Text)),
        7 => Some((R::StyleName, W::Text)),
        210 => Some((R::ExtrusionX, W::Double)),
        220 => Some((R::ExtrusionY, W::Double)),
        230 => Some((R::ExtrusionZ, W::Double)),
        11 => Some((R::XAxisX, W::Double)),
        21 => Some((R::XAxisY, W::Double)),
        31 => Some((R::XAxisZ, W::Double)),
        42 => Some((R::ActualWidth, W::Double)),
        43 => Some((R::ActualHeight, W::Double)),
        50 => Some((R::RotationOrColumnHeight, W::Double)),
        73 => Some((R::LineSpacingStyle, W::Int16)),
        44 => Some((R::LineSpacingFactor, W::Double)),
        90 => Some((R::BackgroundFill, W::Int32)),
        420..=429 => Some((R::BackgroundRgbOrEntityTrueColor, W::Int32)),
        430..=439 => Some((R::BackgroundNameOrEntityColorName, W::Text)),
        45 => Some((R::FillBoxScale, W::Double)),
        63 => Some((R::BackgroundIndex, W::Int16)),
        441 => Some((R::BackgroundTransparency, W::Int32)),
        75 => Some((R::ColumnType, W::Int16)),
        76 => Some((R::ColumnCount, W::Int16)),
        78 => Some((R::ColumnFlowReversed, W::Int16)),
        79 => Some((R::ColumnAutoHeight, W::Int16)),
        48 => Some((R::ColumnWidth, W::Double)),
        49 => Some((R::ColumnGutter, W::Double)),
        _ => None,
    }
}

const fn shape_role(code: i16) -> Option<(DxfTextSymbolValueRole, DxfTextSymbolWireType)> {
    use DxfTextSymbolValueRole as R;
    use DxfTextSymbolWireType as W;
    match code {
        39 => Some((R::Thickness, W::Double)),
        10 => Some((R::InsertionX, W::Double)),
        20 => Some((R::InsertionY, W::Double)),
        30 => Some((R::InsertionZ, W::Double)),
        40 => Some((R::ShapeSize, W::Double)),
        2 => Some((R::ShapeName, W::Text)),
        50 => Some((R::Rotation, W::Double)),
        41 => Some((R::WidthFactor, W::Double)),
        51 => Some((R::ObliqueAngle, W::Double)),
        210 => Some((R::ExtrusionX, W::Double)),
        220 => Some((R::ExtrusionY, W::Double)),
        230 => Some((R::ExtrusionZ, W::Double)),
        _ => None,
    }
}

const fn tolerance_role(code: i16) -> Option<(DxfTextSymbolValueRole, DxfTextSymbolWireType)> {
    use DxfTextSymbolValueRole as R;
    use DxfTextSymbolWireType as W;
    match code {
        3 => Some((R::DimensionStyleName, W::Text)),
        10 => Some((R::InsertionX, W::Double)),
        20 => Some((R::InsertionY, W::Double)),
        30 => Some((R::InsertionZ, W::Double)),
        1 => Some((R::Content, W::Text)),
        210 => Some((R::ExtrusionX, W::Double)),
        220 => Some((R::ExtrusionY, W::Double)),
        230 => Some((R::ExtrusionZ, W::Double)),
        11 => Some((R::XAxisX, W::Double)),
        21 => Some((R::XAxisY, W::Double)),
        31 => Some((R::XAxisZ, W::Double)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{DxfTextSymbolWireType as W, value_role};
    use crate::{DxfTextSymbolKind as K, DxfTextSymbolValueRole as R};

    #[test]
    fn every_documented_family_role_and_wire_domain_is_frozen() {
        assert_roles(
            K::Text,
            &[
                (39, R::Thickness, W::Double),
                (10, R::FirstAlignmentX, W::Double),
                (20, R::FirstAlignmentY, W::Double),
                (30, R::FirstAlignmentZ, W::Double),
                (40, R::TextHeight, W::Double),
                (1, R::Content, W::Text),
                (50, R::Rotation, W::Double),
                (41, R::WidthFactor, W::Double),
                (51, R::ObliqueAngle, W::Double),
                (7, R::StyleName, W::Text),
                (71, R::GenerationFlags, W::Int16),
                (72, R::HorizontalJustification, W::Int16),
                (11, R::SecondAlignmentX, W::Double),
                (21, R::SecondAlignmentY, W::Double),
                (31, R::SecondAlignmentZ, W::Double),
                (210, R::ExtrusionX, W::Double),
                (220, R::ExtrusionY, W::Double),
                (230, R::ExtrusionZ, W::Double),
                (73, R::VerticalJustification, W::Int16),
            ],
        );
        assert_roles(
            K::MText,
            &[
                (10, R::InsertionX, W::Double),
                (20, R::InsertionY, W::Double),
                (30, R::InsertionZ, W::Double),
                (40, R::NominalHeight, W::Double),
                (41, R::ReferenceWidth, W::Double),
                (71, R::Attachment, W::Int16),
                (72, R::DrawingDirection, W::Int16),
                (1, R::Content, W::Text),
                (3, R::AdditionalContent, W::Text),
                (7, R::StyleName, W::Text),
                (210, R::ExtrusionX, W::Double),
                (220, R::ExtrusionY, W::Double),
                (230, R::ExtrusionZ, W::Double),
                (11, R::XAxisX, W::Double),
                (21, R::XAxisY, W::Double),
                (31, R::XAxisZ, W::Double),
                (42, R::ActualWidth, W::Double),
                (43, R::ActualHeight, W::Double),
                (50, R::RotationOrColumnHeight, W::Double),
                (73, R::LineSpacingStyle, W::Int16),
                (44, R::LineSpacingFactor, W::Double),
                (90, R::BackgroundFill, W::Int32),
                (420, R::BackgroundRgbOrEntityTrueColor, W::Int32),
                (429, R::BackgroundRgbOrEntityTrueColor, W::Int32),
                (430, R::BackgroundNameOrEntityColorName, W::Text),
                (439, R::BackgroundNameOrEntityColorName, W::Text),
                (45, R::FillBoxScale, W::Double),
                (63, R::BackgroundIndex, W::Int16),
                (441, R::BackgroundTransparency, W::Int32),
                (75, R::ColumnType, W::Int16),
                (76, R::ColumnCount, W::Int16),
                (78, R::ColumnFlowReversed, W::Int16),
                (79, R::ColumnAutoHeight, W::Int16),
                (48, R::ColumnWidth, W::Double),
                (49, R::ColumnGutter, W::Double),
            ],
        );
        assert_roles(
            K::Shape,
            &[
                (39, R::Thickness, W::Double),
                (10, R::InsertionX, W::Double),
                (20, R::InsertionY, W::Double),
                (30, R::InsertionZ, W::Double),
                (40, R::ShapeSize, W::Double),
                (2, R::ShapeName, W::Text),
                (50, R::Rotation, W::Double),
                (41, R::WidthFactor, W::Double),
                (51, R::ObliqueAngle, W::Double),
                (210, R::ExtrusionX, W::Double),
                (220, R::ExtrusionY, W::Double),
                (230, R::ExtrusionZ, W::Double),
            ],
        );
        assert_roles(
            K::Tolerance,
            &[
                (3, R::DimensionStyleName, W::Text),
                (10, R::InsertionX, W::Double),
                (20, R::InsertionY, W::Double),
                (30, R::InsertionZ, W::Double),
                (1, R::Content, W::Text),
                (210, R::ExtrusionX, W::Double),
                (220, R::ExtrusionY, W::Double),
                (230, R::ExtrusionZ, W::Double),
                (11, R::XAxisX, W::Double),
                (21, R::XAxisY, W::Double),
                (31, R::XAxisZ, W::Double),
            ],
        );
        assert_eq!(value_role(K::Text, 90), None);
        assert_eq!(value_role(K::Shape, 71), None);
        assert_eq!(value_role(K::Tolerance, 40), None);
    }

    fn assert_roles(kind: K, expected: &[(i16, R, W)]) {
        for (code, role, wire) in expected {
            assert_eq!(value_role(kind, *code), Some((*role, *wire)));
        }
    }
}
