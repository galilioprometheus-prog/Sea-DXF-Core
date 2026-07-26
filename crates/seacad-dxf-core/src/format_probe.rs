use crate::{DxfByteSource, DxfError, DxfResource, DxfResourceProfile};

/// Exact 22-byte sentinel Autodesk assigns to Binary DXF files.
pub const DXF_BINARY_SENTINEL: [u8; 22] = *b"AutoCAD Binary DXF\r\n\x1a\0";

/// Physical representation detected before DXF record validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPhysicalFormat {
    /// Not the binary sentinel; record framing must still validate the input.
    AsciiCandidate,
    /// Exact Autodesk Binary DXF sentinel is present.
    Binary,
    /// Empty, NUL-bearing in the inspected prefix, or a truncated sentinel.
    Unknown,
}

/// Probes only the physical representation; it does not validate DXF records.
pub fn probe_dxf_physical_format(
    source: &dyn DxfByteSource,
    profile: DxfResourceProfile,
) -> Result<DxfPhysicalFormat, DxfError> {
    let source_len = source.len();
    let limit = profile.limits().max_source_bytes();
    if source_len > limit {
        return Err(DxfError::resource_limit(
            DxfResource::SourceBytes,
            limit,
            source_len,
        ));
    }
    if source_len == 0 {
        return Ok(DxfPhysicalFormat::Unknown);
    }

    let prefix_len_u64 = source_len.min(DXF_BINARY_SENTINEL.len() as u64);
    let prefix_len = usize::try_from(prefix_len_u64).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: prefix_len_u64,
    })?;
    let mut prefix = [0_u8; DXF_BINARY_SENTINEL.len()];
    source.read_exact_at(0, &mut prefix[..prefix_len])?;
    let observed = &prefix[..prefix_len];

    if observed == DXF_BINARY_SENTINEL {
        Ok(DxfPhysicalFormat::Binary)
    } else if observed == &DXF_BINARY_SENTINEL[..prefix_len] || observed.contains(&0) {
        Ok(DxfPhysicalFormat::Unknown)
    } else {
        Ok(DxfPhysicalFormat::AsciiCandidate)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{DXF_BINARY_SENTINEL, DxfPhysicalFormat, probe_dxf_physical_format};
    use crate::{DxfByteSource, DxfError, DxfMemorySource, DxfResource, DxfResourceProfile};

    struct OneByteSource {
        bytes: &'static [u8],
    }

    impl DxfByteSource for OneByteSource {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            if destination.is_empty() || offset >= self.len() {
                return Ok(0);
            }
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            destination[0] = self.bytes[start];
            Ok(1)
        }
    }

    struct OversizedSource {
        read_calls: AtomicU64,
    }

    impl DxfByteSource for OversizedSource {
        fn len(&self) -> u64 {
            DxfResourceProfile::Safe.limits().max_source_bytes() + 1
        }

        fn read_at(&self, _offset: u64, _destination: &mut [u8]) -> Result<usize, DxfError> {
            self.read_calls.fetch_add(1, Ordering::Relaxed);
            Ok(0)
        }
    }

    #[test]
    fn exact_binary_sentinel_is_authoritative_even_with_partial_reads() -> Result<(), DxfError> {
        assert_eq!(DXF_BINARY_SENTINEL.len(), 22);
        let source = OneByteSource {
            bytes: &DXF_BINARY_SENTINEL,
        };
        assert_eq!(
            probe_dxf_physical_format(&source, DxfResourceProfile::Safe)?,
            DxfPhysicalFormat::Binary
        );

        let mut with_payload = DXF_BINARY_SENTINEL.to_vec();
        with_payload.extend_from_slice(&[0, 1, 2, 3]);
        let source = DxfMemorySource::new(&with_payload, DxfResourceProfile::Safe)?;
        assert_eq!(
            probe_dxf_physical_format(&source, DxfResourceProfile::Safe)?,
            DxfPhysicalFormat::Binary
        );
        Ok(())
    }

    #[test]
    fn non_binary_text_is_only_an_ascii_candidate() -> Result<(), DxfError> {
        for bytes in [
            b"  0\r\nSECTION\r\n".as_slice(),
            b"not a validated DXF".as_slice(),
            b"\xef\xbb\xbf  0\nSECTION\n".as_slice(),
            b"\x80\x81\n".as_slice(),
        ] {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            assert_eq!(
                probe_dxf_physical_format(&source, DxfResourceProfile::Safe)?,
                DxfPhysicalFormat::AsciiCandidate
            );
        }
        Ok(())
    }

    #[test]
    fn empty_truncated_sentinel_and_nul_prefix_are_unknown() -> Result<(), DxfError> {
        for prefix_len in 0..DXF_BINARY_SENTINEL.len() {
            let source =
                DxfMemorySource::new(&DXF_BINARY_SENTINEL[..prefix_len], DxfResourceProfile::Safe)?;
            assert_eq!(
                probe_dxf_physical_format(&source, DxfResourceProfile::Safe)?,
                DxfPhysicalFormat::Unknown
            );
        }

        let source = DxfMemorySource::new(b"0\nSECT\0ION\n", DxfResourceProfile::Safe)?;
        assert_eq!(
            probe_dxf_physical_format(&source, DxfResourceProfile::Safe)?,
            DxfPhysicalFormat::Unknown
        );
        Ok(())
    }

    #[test]
    fn probe_rechecks_the_selected_source_limit_before_reading() {
        let source = OversizedSource {
            read_calls: AtomicU64::new(0),
        };
        let result = probe_dxf_physical_format(&source, DxfResourceProfile::Safe);
        assert!(matches!(
            result,
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::SourceBytes,
                ..
            })
        ));
        assert_eq!(source.read_calls.load(Ordering::Relaxed), 0);
    }
}
