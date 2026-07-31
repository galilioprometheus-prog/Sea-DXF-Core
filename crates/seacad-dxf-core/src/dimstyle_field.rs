//! Reviewed DIMSTYLE-record field registry.

/// Wire family used by one documented DIMSTYLE-specific group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleWireKind {
    Text,
    Double,
    Int16,
    Handle,
}

/// One documented DIMSTYLE-specific field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleField {
    group_code: i16,
    name: &'static str,
    wire_kind: DxfDimStyleWireKind,
}

impl DxfDimStyleField {
    const fn new(group_code: i16, name: &'static str, wire_kind: DxfDimStyleWireKind) -> Self {
        Self {
            group_code,
            name,
            wire_kind,
        }
    }

    #[must_use]
    pub const fn group_code(self) -> i16 {
        self.group_code
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn wire_kind(self) -> DxfDimStyleWireKind {
        self.wire_kind
    }
}

use DxfDimStyleWireKind::{Double, Handle, Int16, Text};

const FIELDS: [DxfDimStyleField; 68] = [
    field(3, "DIMPOST", Text),
    field(4, "DIMAPOST", Text),
    field(5, "DIMBLK_OBSOLETE", Text),
    field(6, "DIMBLK1_OBSOLETE", Text),
    field(7, "DIMBLK2_OBSOLETE", Text),
    field(40, "DIMSCALE", Double),
    field(41, "DIMASZ", Double),
    field(42, "DIMEXO", Double),
    field(43, "DIMDLI", Double),
    field(44, "DIMEXE", Double),
    field(45, "DIMRND", Double),
    field(46, "DIMDLE", Double),
    field(47, "DIMTP", Double),
    field(48, "DIMTM", Double),
    field(70, "STANDARD_FLAGS", Int16),
    field(71, "DIMTOL", Int16),
    field(72, "DIMLIM", Int16),
    field(73, "DIMTIH", Int16),
    field(74, "DIMTOH", Int16),
    field(75, "DIMSE1", Int16),
    field(76, "DIMSE2", Int16),
    field(77, "DIMTAD", Int16),
    field(78, "DIMZIN", Int16),
    field(79, "DIMAZIN", Int16),
    field(140, "DIMTXT", Double),
    field(141, "DIMCEN", Double),
    field(142, "DIMTSZ", Double),
    field(143, "DIMALTF", Double),
    field(144, "DIMLFAC", Double),
    field(145, "DIMTVP", Double),
    field(146, "DIMTFAC", Double),
    field(147, "DIMGAP", Double),
    field(148, "DIMALTRND", Double),
    field(170, "DIMALT", Int16),
    field(171, "DIMALTD", Int16),
    field(172, "DIMTOFL", Int16),
    field(173, "DIMSAH", Int16),
    field(174, "DIMTIX", Int16),
    field(175, "DIMSOXD", Int16),
    field(176, "DIMCLRD", Int16),
    field(177, "DIMCLRE", Int16),
    field(178, "DIMCLRT", Int16),
    field(179, "DIMADEC", Int16),
    field(270, "DIMUNIT_OBSOLETE", Int16),
    field(271, "DIMDEC", Int16),
    field(272, "DIMTDEC", Int16),
    field(273, "DIMALTU", Int16),
    field(274, "DIMALTTD", Int16),
    field(275, "DIMAUNIT", Int16),
    field(276, "DIMFRAC", Int16),
    field(277, "DIMLUNIT", Int16),
    field(278, "DIMDSEP", Int16),
    field(279, "DIMTMOVE", Int16),
    field(280, "DIMJUST", Int16),
    field(281, "DIMSD1", Int16),
    field(282, "DIMSD2", Int16),
    field(283, "DIMTOLJ", Int16),
    field(284, "DIMTZIN", Int16),
    field(285, "DIMALTZ", Int16),
    field(286, "DIMALTTZ", Int16),
    field(287, "DIMFIT_OBSOLETE", Int16),
    field(288, "DIMUPT", Int16),
    field(289, "DIMATFIT", Int16),
    field(340, "DIMTXSTY", Handle),
    field(341, "DIMLDRBLK", Handle),
    field(342, "DIMBLK", Handle),
    field(343, "DIMBLK1", Handle),
    field(344, "DIMBLK2", Handle),
];

const fn field(
    group_code: i16,
    name: &'static str,
    wire_kind: DxfDimStyleWireKind,
) -> DxfDimStyleField {
    DxfDimStyleField::new(group_code, name, wire_kind)
}

/// Returns every reviewed DIMSTYLE-specific field in ascending group-code order.
#[must_use]
pub const fn dxf_dimstyle_fields() -> &'static [DxfDimStyleField] {
    &FIELDS
}

#[must_use]
pub(crate) fn field_for_group_code(group_code: i16) -> Option<DxfDimStyleField> {
    FIELDS
        .binary_search_by_key(&group_code, |field| field.group_code())
        .ok()
        .and_then(|index| FIELDS.get(index).copied())
}
