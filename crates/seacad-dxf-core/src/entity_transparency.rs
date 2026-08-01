//! Reviewed common entity-transparency wire semantics.

const METHOD_SHIFT: u32 = 24;
const METHOD_PAYLOAD_MASK: u32 = 0x00ff_ffff;
const ALPHA_MASK: u32 = 0x0000_00ff;
const RESERVED_ALPHA_MASK: u32 = METHOD_PAYLOAD_MASK ^ ALPHA_MASK;

/// Public transparency method represented by the high wire byte of group 440.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityTransparencyMethod {
    ByLayer,
    ByBlock,
    ByAlpha,
}

impl DxfEntityTransparencyMethod {
    /// ObjectARX `kByLayer` method byte.
    pub const BY_LAYER_RAW: u8 = 0;
    /// ObjectARX `kByBlock` method byte.
    pub const BY_BLOCK_RAW: u8 = 1;
    /// ObjectARX `kByAlpha` method byte.
    pub const BY_ALPHA_RAW: u8 = 2;

    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::ByLayer => Self::BY_LAYER_RAW,
            Self::ByBlock => Self::BY_BLOCK_RAW,
            Self::ByAlpha => Self::BY_ALPHA_RAW,
        }
    }
}

/// Why a group-440 integer is not one reviewed transparency encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityTransparencyIssue {
    UnsupportedMethod { method: u8 },
    ReservedPayload { method: u8, payload: u32 },
}

/// Exact public method and alpha encoded by common group 440.
///
/// Alpha follows ObjectARX: zero is fully transparent and 255 is fully opaque.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityTransparency {
    ByLayer,
    ByBlock,
    ByAlpha { alpha: u8 },
}

impl DxfEntityTransparency {
    pub const BY_LAYER_RAW: i32 = 0x0000_0000;
    pub const BY_BLOCK_RAW: i32 = 0x0100_0000;
    pub const BY_ALPHA_PREFIX: i32 = 0x0200_0000;

    /// Classifies one exact signed DXF Int32 without accepting reserved bits.
    pub const fn classify_raw(value: i32) -> Result<Self, DxfEntityTransparencyIssue> {
        let bits = value as u32;
        let method = (bits >> METHOD_SHIFT) as u8;
        let payload = bits & METHOD_PAYLOAD_MASK;
        match method {
            DxfEntityTransparencyMethod::BY_LAYER_RAW => {
                if payload == 0 {
                    Ok(Self::ByLayer)
                } else {
                    Err(DxfEntityTransparencyIssue::ReservedPayload { method, payload })
                }
            }
            DxfEntityTransparencyMethod::BY_BLOCK_RAW => {
                if payload == 0 {
                    Ok(Self::ByBlock)
                } else {
                    Err(DxfEntityTransparencyIssue::ReservedPayload { method, payload })
                }
            }
            DxfEntityTransparencyMethod::BY_ALPHA_RAW => {
                if payload & RESERVED_ALPHA_MASK == 0 {
                    Ok(Self::ByAlpha {
                        alpha: (payload & ALPHA_MASK) as u8,
                    })
                } else {
                    Err(DxfEntityTransparencyIssue::ReservedPayload { method, payload })
                }
            }
            _ => Err(DxfEntityTransparencyIssue::UnsupportedMethod { method }),
        }
    }

    #[must_use]
    pub const fn from_raw(value: i32) -> Option<Self> {
        match Self::classify_raw(value) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i32 {
        match self {
            Self::ByLayer => Self::BY_LAYER_RAW,
            Self::ByBlock => Self::BY_BLOCK_RAW,
            Self::ByAlpha { alpha } => Self::BY_ALPHA_PREFIX | alpha as i32,
        }
    }

    #[must_use]
    pub const fn method(self) -> DxfEntityTransparencyMethod {
        match self {
            Self::ByLayer => DxfEntityTransparencyMethod::ByLayer,
            Self::ByBlock => DxfEntityTransparencyMethod::ByBlock,
            Self::ByAlpha { .. } => DxfEntityTransparencyMethod::ByAlpha,
        }
    }

    /// Returns alpha only when the method is explicitly `ByAlpha`.
    #[must_use]
    pub const fn alpha(self) -> Option<u8> {
        match self {
            Self::ByAlpha { alpha } => Some(alpha),
            Self::ByLayer | Self::ByBlock => None,
        }
    }

    #[must_use]
    pub const fn is_by_layer(self) -> bool {
        matches!(self, Self::ByLayer)
    }

    #[must_use]
    pub const fn is_by_block(self) -> bool {
        matches!(self, Self::ByBlock)
    }

    #[must_use]
    pub const fn is_by_alpha(self) -> bool {
        matches!(self, Self::ByAlpha { .. })
    }

    #[must_use]
    pub const fn is_clear(self) -> bool {
        matches!(self, Self::ByAlpha { alpha: 0 })
    }

    #[must_use]
    pub const fn is_solid(self) -> bool {
        matches!(self, Self::ByAlpha { alpha: u8::MAX })
    }
}
