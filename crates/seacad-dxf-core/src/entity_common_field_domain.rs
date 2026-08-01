//! Reviewed scalar domains for common entity-field edits.

use std::num::NonZeroU8;

use crate::{DxfDouble, DxfEntityEditValue, DxfEntityEditValueKind, DxfEntityField};

/// Model-space or paper-space placement encoded by common group 67.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntitySpace {
    Model,
    Paper,
}

impl DxfEntitySpace {
    #[must_use]
    pub const fn from_raw(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Model),
            1 => Some(Self::Paper),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        match self {
            Self::Model => 0,
            Self::Paper => 1,
        }
    }
}

/// Indexed entity color, including BYBLOCK/BYLAYER and layer-off ACI values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityIndexedColor {
    ByBlock,
    Aci { index: NonZeroU8, layer_off: bool },
    ByLayer,
}

impl DxfEntityIndexedColor {
    #[must_use]
    pub fn from_raw(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::ByBlock),
            256 => Some(Self::ByLayer),
            1..=255 => Some(Self::Aci {
                index: NonZeroU8::new(u8::try_from(value).ok()?)?,
                layer_off: false,
            }),
            -255..=-1 => Some(Self::Aci {
                index: NonZeroU8::new(u8::try_from(value.unsigned_abs()).ok()?)?,
                layer_off: true,
            }),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        match self {
            Self::ByBlock => 0,
            Self::Aci {
                index,
                layer_off: false,
            } => index.get() as i16,
            Self::Aci {
                index,
                layer_off: true,
            } => -(index.get() as i16),
            Self::ByLayer => 256,
        }
    }
}

/// One public `AcDb::LineWeight` value stored by common group 370.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityLineweight(i16);

impl DxfEntityLineweight {
    pub const BY_LINEWEIGHT_DEFAULT: Self = Self(-3);
    pub const BY_BLOCK: Self = Self(-2);
    pub const BY_LAYER: Self = Self(-1);

    #[must_use]
    pub const fn from_raw(value: i16) -> Option<Self> {
        match value {
            -3..=-1
            | 0
            | 5
            | 9
            | 13
            | 15
            | 18
            | 20
            | 25
            | 30
            | 35
            | 40
            | 50
            | 53
            | 60
            | 70
            | 80
            | 90
            | 100
            | 106
            | 120
            | 140
            | 158
            | 200
            | 211 => Some(Self(value)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        self.0
    }
}

/// Object visibility encoded by common group 60.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityVisibility {
    Visible,
    Invisible,
}

impl DxfEntityVisibility {
    #[must_use]
    pub const fn from_raw(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Visible),
            1 => Some(Self::Invisible),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        match self {
            Self::Visible => 0,
            Self::Invisible => 1,
        }
    }
}

/// Obsolete-but-supported common shadow mode encoded by group 284.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityShadowMode {
    CastsAndReceives,
    Casts,
    Receives,
    Ignores,
}

impl DxfEntityShadowMode {
    #[must_use]
    pub const fn from_raw(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::CastsAndReceives),
            1 => Some(Self::Casts),
            2 => Some(Self::Receives),
            3 => Some(Self::Ignores),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        match self {
            Self::CastsAndReceives => 0,
            Self::Casts => 1,
            Self::Receives => 2,
            Self::Ignores => 3,
        }
    }
}

/// A group-420 RGB value whose high byte is proven zero.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityTrueColor(u32);

impl DxfEntityTrueColor {
    #[must_use]
    pub const fn from_raw(value: i32) -> Option<Self> {
        if value >= 0 && value <= 0x00ff_ffff {
            Some(Self(value as u32))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0 as i32
    }

    #[must_use]
    pub const fn red(self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    #[must_use]
    pub const fn green(self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    #[must_use]
    pub const fn blue(self) -> u8 {
        (self.0 & 0xff) as u8
    }
}

/// Typed value returned for a reviewed common scalar edit domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldDomainValue {
    Space(DxfEntitySpace),
    IndexedColor(DxfEntityIndexedColor),
    Lineweight(DxfEntityLineweight),
    LinetypeScale(DxfDouble),
    Visibility(DxfEntityVisibility),
    ProxyGraphicsSize(u32),
    TrueColor(DxfEntityTrueColor),
    ShadowMode(DxfEntityShadowMode),
}

/// Why one reviewed common scalar edit value is outside its public domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldDomainIssue {
    ValueKindMismatch {
        field: DxfEntityField,
        expected: DxfEntityEditValueKind,
        observed: DxfEntityEditValueKind,
    },
    UnsupportedInt16 {
        field: DxfEntityField,
        value: i16,
    },
    UnsupportedInt32 {
        field: DxfEntityField,
        value: i32,
    },
    InvalidDouble {
        field: DxfEntityField,
        value: DxfDouble,
    },
}

/// Classification of one explicit value against reviewed common-field domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldDomainOutcome {
    Unreviewed { field: DxfEntityField },
    Valid(DxfEntityCommonFieldDomainValue),
    Invalid(DxfEntityCommonFieldDomainIssue),
}

/// Classifies an explicit edit without reading or changing a document.
#[must_use]
pub fn classify_entity_common_field_edit_domain(
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> DxfEntityCommonFieldDomainOutcome {
    if field == DxfEntityField::PAPER_SPACE {
        return classify_i16(field, value, |raw| {
            DxfEntitySpace::from_raw(raw).map(DxfEntityCommonFieldDomainValue::Space)
        });
    }
    if field == DxfEntityField::COLOR {
        return classify_i16(field, value, |raw| {
            DxfEntityIndexedColor::from_raw(raw).map(DxfEntityCommonFieldDomainValue::IndexedColor)
        });
    }
    if field == DxfEntityField::LINEWEIGHT {
        return classify_i16(field, value, |raw| {
            DxfEntityLineweight::from_raw(raw).map(DxfEntityCommonFieldDomainValue::Lineweight)
        });
    }
    if field == DxfEntityField::LINETYPE_SCALE {
        return match value {
            DxfEntityEditValue::Double(raw) if raw.is_finite() && raw.to_f64() >= 0.0 => {
                DxfEntityCommonFieldDomainOutcome::Valid(
                    DxfEntityCommonFieldDomainValue::LinetypeScale(raw),
                )
            }
            DxfEntityEditValue::Double(raw) => DxfEntityCommonFieldDomainOutcome::Invalid(
                DxfEntityCommonFieldDomainIssue::InvalidDouble { field, value: raw },
            ),
            other => kind_mismatch(field, DxfEntityEditValueKind::Double, other.kind()),
        };
    }
    if field == DxfEntityField::VISIBILITY {
        return classify_i16(field, value, |raw| {
            DxfEntityVisibility::from_raw(raw).map(DxfEntityCommonFieldDomainValue::Visibility)
        });
    }
    if field == DxfEntityField::PROXY_GRAPHICS_SIZE {
        return match value {
            DxfEntityEditValue::Int32(raw) => match u32::try_from(raw) {
                Ok(size) => DxfEntityCommonFieldDomainOutcome::Valid(
                    DxfEntityCommonFieldDomainValue::ProxyGraphicsSize(size),
                ),
                Err(_) => unsupported_i32(field, raw),
            },
            other => kind_mismatch(field, DxfEntityEditValueKind::Int32, other.kind()),
        };
    }
    if field == DxfEntityField::TRUE_COLOR {
        return match value {
            DxfEntityEditValue::Int32(raw) => match DxfEntityTrueColor::from_raw(raw) {
                Some(color) => DxfEntityCommonFieldDomainOutcome::Valid(
                    DxfEntityCommonFieldDomainValue::TrueColor(color),
                ),
                None => unsupported_i32(field, raw),
            },
            other => kind_mismatch(field, DxfEntityEditValueKind::Int32, other.kind()),
        };
    }
    if field == DxfEntityField::SHADOW {
        return classify_i16(field, value, |raw| {
            DxfEntityShadowMode::from_raw(raw).map(DxfEntityCommonFieldDomainValue::ShadowMode)
        });
    }
    DxfEntityCommonFieldDomainOutcome::Unreviewed { field }
}

fn classify_i16(
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
    classify: impl FnOnce(i16) -> Option<DxfEntityCommonFieldDomainValue>,
) -> DxfEntityCommonFieldDomainOutcome {
    match value {
        DxfEntityEditValue::Int16(raw) => match classify(raw) {
            Some(value) => DxfEntityCommonFieldDomainOutcome::Valid(value),
            None => DxfEntityCommonFieldDomainOutcome::Invalid(
                DxfEntityCommonFieldDomainIssue::UnsupportedInt16 { field, value: raw },
            ),
        },
        other => kind_mismatch(field, DxfEntityEditValueKind::Int16, other.kind()),
    }
}

const fn unsupported_i32(field: DxfEntityField, value: i32) -> DxfEntityCommonFieldDomainOutcome {
    DxfEntityCommonFieldDomainOutcome::Invalid(DxfEntityCommonFieldDomainIssue::UnsupportedInt32 {
        field,
        value,
    })
}

const fn kind_mismatch(
    field: DxfEntityField,
    expected: DxfEntityEditValueKind,
    observed: DxfEntityEditValueKind,
) -> DxfEntityCommonFieldDomainOutcome {
    DxfEntityCommonFieldDomainOutcome::Invalid(DxfEntityCommonFieldDomainIssue::ValueKindMismatch {
        field,
        expected,
        observed,
    })
}
