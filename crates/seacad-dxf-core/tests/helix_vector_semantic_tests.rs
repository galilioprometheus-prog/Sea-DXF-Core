use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HELIX_VECTOR_KINDS, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfHelixCoordinateIssue,
    DxfHelixVectorDirectory, DxfHelixVectorKind, DxfHelixVectorSemantics, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

type VectorEvidence = (
    DxfHelixVectorKind,
    [DxfSemanticValueState; 3],
    Option<[u64; 3]>,
    [Option<DxfHelixCoordinateIssue>; 3],
);

#[test]
fn every_dialect_has_ascii_binary_helix_vector_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_vectors = ascii.helix_vector_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_vectors = binary.helix_vector_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_vectors)?;
        assert_expected(&binary_vectors)?;
        assert_eq!(evidence(&ascii_vectors), evidence(&binary_vectors));
    }
    Ok(())
}

#[test]
fn partial_duplicate_invalid_and_nonfinite_components_fail_without_defaults()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n\
0\nHELIX\n100\nAcDbHelix\n10\n1\n10\n2\n30\n.\n11\n3\n21\n4\n31\n5\n\
12\n0\n22\n0\n32\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let vectors = document.helix_vector_directory(&DxfCancellationToken::default())?;
    assert_eq!(vectors.entries().len(), DXF_HELIX_VECTOR_KINDS.len() * 2);
    for entry in &vectors.entries()[..DXF_HELIX_VECTOR_KINDS.len()] {
        assert!(
            entry
                .components()
                .iter()
                .all(|component| component.state() == DxfSemanticValueState::Absent)
        );
        assert_eq!(entry.vector_value(), None);
    }

    let second_raw = vectors.card_directory().evidence_directory().records()[1]
        .entity()
        .record()
        .ordinal();
    let axis_base = vector_for(&vectors, second_raw, DxfHelixVectorKind::AxisBase)?;
    assert_eq!(axis_base.vector_value(), None);
    assert_eq!(
        axis_base.components()[0].invalid_issue(),
        Some(&DxfHelixCoordinateIssue::MultipleComponents {
            occurrence_count: 2
        })
    );
    assert_eq!(axis_base.components()[0].raw_provenance(), None);
    assert_eq!(
        axis_base.components()[1].state(),
        DxfSemanticValueState::Absent
    );
    assert!(matches!(
        axis_base.components()[2].invalid_issue(),
        Some(DxfHelixCoordinateIssue::InvalidAsciiNumber(_))
    ));
    assert!(axis_base.components()[2].raw_provenance().is_some());

    let start = vector_for(&vectors, second_raw, DxfHelixVectorKind::StartPoint)?;
    assert_eq!(
        start.vector_value().map(bits),
        Some([3.0_f64.to_bits(), 4.0_f64.to_bits(), 5.0_f64.to_bits()])
    );
    let axis = vector_for(&vectors, second_raw, DxfHelixVectorKind::AxisVector)?;
    assert_eq!(axis.vector_value().map(bits), Some([0.0_f64.to_bits(); 3]));

    let binary_bytes = binary_nonfinite_fixture()?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    let binary_vectors = binary.helix_vector_directory(&DxfCancellationToken::default())?;
    let raw = binary_vectors
        .card_directory()
        .evidence_directory()
        .records()[0]
        .entity()
        .record()
        .ordinal();
    let axis = vector_for(&binary_vectors, raw, DxfHelixVectorKind::AxisVector)?;
    assert!(matches!(
        axis.components()[1].invalid_issue(),
        Some(DxfHelixCoordinateIssue::NonFiniteDouble(value)) if value.to_f64().is_infinite()
    ));
    assert_eq!(axis.vector_value(), None);
    Ok(())
}

#[test]
fn cancellation_lookup_provenance_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_vector_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_vector_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let vectors = document.helix_vector_directory(&DxfCancellationToken::default())?;
    assert_eq!(vectors.entry(u64::MAX), None);
    assert_eq!(vectors.entries_for_raw_record(u64::MAX), None);
    assert_eq!(
        vectors.vector_for_kind(u64::MAX, DxfHelixVectorKind::AxisBase),
        None
    );
    assert_eq!(vectors.source_id(), vectors.card_directory().source_id());
    let first = vectors.entries()[0];
    for (component, field_id) in
        first
            .components()
            .iter()
            .zip(["axis_base_x", "axis_base_y", "axis_base_z"])
    {
        assert_eq!(
            component.field_provenance().schema_namespace(),
            "entity.helix"
        );
        assert_eq!(component.field_provenance().schema_field_id(), field_id);
        assert_eq!(
            component.field_provenance().document_source_id(),
            vectors.source_id()
        );
        assert!(component.raw_provenance().is_some());
    }
    assert_copy::<DxfHelixVectorSemantics>();
    assert_send_sync::<DxfHelixVectorDirectory>();
    assert!(size_of::<DxfHelixVectorSemantics>() <= 512);
    Ok(())
}

fn assert_expected(vectors: &DxfHelixVectorDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(vectors.entries().len(), DXF_HELIX_VECTOR_KINDS.len());
    let record = vectors.card_directory().evidence_directory().records()[0];
    let raw = record.entity().record().ordinal();
    let entries = vectors
        .entries_for_raw_record(raw)
        .ok_or(io::Error::other("vector entries"))?;
    assert_eq!(
        entries.iter().map(|entry| entry.kind()).collect::<Vec<_>>(),
        DXF_HELIX_VECTOR_KINDS
    );
    for (kind, expected) in [
        (DxfHelixVectorKind::AxisBase, [1.0, 2.0, 3.0]),
        (DxfHelixVectorKind::StartPoint, [4.0, 5.0, 6.0]),
        (DxfHelixVectorKind::AxisVector, [0.0, 0.0, 1.0]),
    ] {
        let entry = vector_for(vectors, raw, kind)?;
        assert_eq!(
            entry.vector_value().map(bits),
            Some(expected.map(f64::to_bits))
        );
        assert!(
            entry
                .components()
                .iter()
                .all(|component| component.state() == DxfSemanticValueState::Explicit)
        );
    }
    Ok(())
}

fn vector_for(
    vectors: &DxfHelixVectorDirectory,
    raw: u64,
    kind: DxfHelixVectorKind,
) -> Result<DxfHelixVectorSemantics, io::Error> {
    vectors
        .vector_for_kind(raw, kind)
        .ok_or(io::Error::other("vector semantic"))
}

fn evidence(vectors: &DxfHelixVectorDirectory) -> Vec<VectorEvidence> {
    vectors
        .entries()
        .iter()
        .map(|entry| {
            let components = entry.components();
            (
                entry.kind(),
                components.each_ref().map(|component| component.state()),
                entry.vector_value().map(bits),
                components
                    .each_ref()
                    .map(|component| component.invalid_issue().copied()),
            )
        })
        .collect()
}

fn bits(values: [seacad_dxf_core::DxfDouble; 3]) -> [u64; 3] {
    values.map(seacad_dxf_core::DxfDouble::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n31\n6\n10\n1\n22\n0\n\
11\n4\n30\n3\n12\n0\n21\n5\n20\n2\n32\n1\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    for (code, value) in [
        (31, 6.0),
        (10, 1.0),
        (22, 0.0),
        (11, 4.0),
        (30, 3.0),
        (12, 0.0),
        (21, 5.0),
        (20, 2.0),
        (32, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_nonfinite_fixture() -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_prefix(version)?;
    push_double(&mut bytes, version, 12, 0.0)?;
    push_double_bits(&mut bytes, version, 22, f64::INFINITY.to_bits())?;
    push_double(&mut bytes, version, 32, 1.0)?;
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HELIX"),
        (100, b"AcDbHelix"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    Ok(bytes)
}

fn binary_suffix(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_double_bits(bytes, version, code, value.to_bits())
}

fn push_double_bits(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    bits: u64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

struct CancellingSource<'a> {
    bytes: &'a [u8],
    token: DxfCancellationToken,
    armed: AtomicBool,
}

impl<'a> CancellingSource<'a> {
    fn new(bytes: &'a [u8], token: DxfCancellationToken) -> Self {
        Self {
            bytes,
            token,
            armed: AtomicBool::new(false),
        }
    }

    fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
}

impl DxfByteSource for CancellingSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        if self.armed.load(Ordering::Acquire) {
            self.token.cancel();
        }
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
