use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfFillMeshEvidenceDirectory, DxfFillMeshFamily,
    DxfFillMeshField, DxfFillMeshRecordEntry, DxfFillMeshSubclassEntry, DxfMemorySource,
    DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_fill_mesh_evidence_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.fill_mesh_evidence_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.fill_mesh_evidence_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            field_shape(&ascii_directory),
            field_shape(&binary_directory)
        );
        assert_eq!(field_shape(&ascii_directory), expected_shape());
    }
    Ok(())
}

#[test]
fn duplicate_subclasses_and_colliding_nested_fields_remain_raw() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n91\n900\n100\nAcDbHatch\n91\n2\n\
10\n1\n20\n2\n102\n{APP\n91\n999\n102\n}\n100\nAcDbHatch\n91\n3\n\
100\nAcDbHatchX\n91\n901\n0\nMESH\n90\n800\n100\nAcDbSubDMesh\n\
91\n1\n92\n4\n90\n3\n90\n0\n90\n1\n90\n2\n1001\nAPP\n1000\nopaque\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.fill_mesh_evidence_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.subclasses().len(), 3);

    let hatch = directory.records()[0];
    let hatch_subclasses = directory
        .subclasses_for_raw_record(hatch.entity().record().ordinal())
        .ok_or(io::Error::other("hatch subclasses"))?;
    assert_eq!(hatch_subclasses.len(), 2);
    assert_eq!(codes(directory.fields_for_subclass(0)), vec![91, 10, 20]);
    assert_eq!(codes(directory.fields_for_subclass(1)), vec![91]);

    let mesh = directory.records()[1];
    let mesh_subclasses = directory
        .subclasses_for_raw_record(mesh.entity().record().ordinal())
        .ok_or(io::Error::other("mesh subclasses"))?;
    assert_eq!(mesh_subclasses.len(), 1);
    assert_eq!(
        codes(directory.fields_for_subclass(2)),
        vec![91, 92, 90, 90, 90, 90]
    );
    assert!(
        directory
            .fields()
            .iter()
            .all(|field| !matches!(field.group().group_code().value(), 102 | 1000..=i16::MAX))
    );
    assert!(
        directory
            .fields()
            .windows(2)
            .all(|pair| { pair[0].group().occurrence() < pair[1].group().occurrence() })
    );
    Ok(())
}

#[test]
fn matching_is_exact_subclass_gated_and_complete_section_limited() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nMESH\n100\nAcDbSubDMesh\n71\n1\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nhatch\n100\nAcDbHatch\n91\n2\n\
0\nHATCH \n100\nAcDbHatch\n91\n3\n0\nHATCH\n91\n4\n100\nacdbhatch\n91\n5\n\
0\nHATCH\n91\n6\n100\nAcDbHatch\n91\n7\n0\nMESH\n100\nAcDbSubDMeshX\n71\n2\n\
0\nMESH\n100\nAcDbSubDMesh\n71\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.fill_mesh_evidence_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 5);
    assert_eq!(directory.subclasses().len(), 3);
    assert_eq!(directory.fields().len(), 3);
    assert_eq!(directory.records()[0].family(), DxfFillMeshFamily::Mesh);
    assert_eq!(
        directory.records()[0].entity().record().section_kind(),
        DxfRawRecordSectionKind::Blocks
    );
    assert!(directory.records()[1].subclass_range().is_empty());
    assert_eq!(directory.records()[2].family(), DxfFillMeshFamily::Hatch);
    assert!(directory.records()[3].subclass_range().is_empty());
    assert_eq!(directory.records()[4].family(), DxfFillMeshFamily::Mesh);
    assert_eq!(codes(directory.fields_for_subclass(1)), vec![91]);
    assert_eq!(codes(directory.fields_for_subclass(2)), vec![71]);
    Ok(())
}

#[test]
fn cancellation_lookups_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.fill_mesh_evidence_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.fill_mesh_evidence_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.fill_mesh_evidence_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.subclasses_for_raw_record(u64::MAX), None);
    assert_eq!(directory.fields_for_subclass(u64::MAX), None);
    assert_eq!(directory.field_for_group(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.entity_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.application_group_directory().source_id()
    );
    assert_copy::<DxfFillMeshField>();
    assert_copy::<DxfFillMeshRecordEntry>();
    assert_copy::<DxfFillMeshSubclassEntry>();
    assert_send_sync::<DxfFillMeshEvidenceDirectory>();
    assert!(size_of::<DxfFillMeshField>() <= 96);
    assert!(size_of::<DxfFillMeshRecordEntry>() <= 256);
    assert!(size_of::<DxfFillMeshSubclassEntry>() <= 256);
    Ok(())
}

fn assert_directory(directory: &DxfFillMeshEvidenceDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.subclasses().len(), 2);
    assert_eq!(directory.fields().len(), 6);
    for record in directory.records().iter().copied() {
        let subclasses = directory
            .subclasses_for_raw_record(record.entity().record().ordinal())
            .ok_or(io::Error::other("subclass slice"))?;
        assert_eq!(subclasses.len(), 1);
        assert_eq!(subclasses[0].family(), record.family());
        assert!(!subclasses[0].marker().value_payload_span().is_empty());
    }
    for (ordinal, subclass) in directory.subclasses().iter().copied().enumerate() {
        let fields = directory
            .fields_for_subclass(ordinal as u64)
            .ok_or(io::Error::other("field slice"))?;
        assert_eq!(fields.len() as u64, subclass.field_range().len());
        for field in fields.iter().copied() {
            assert_eq!(field.family(), subclass.family());
            assert_eq!(field.subclass_ordinal(), ordinal as u64);
            assert_eq!(
                directory.field_for_group(field.group().occurrence()),
                Some(field)
            );
            assert!(!field.group().value_payload_span().is_empty());
        }
    }
    Ok(())
}

fn field_shape(directory: &DxfFillMeshEvidenceDirectory) -> Vec<(DxfFillMeshFamily, i16)> {
    directory
        .fields()
        .iter()
        .map(|field| (field.family(), field.group().group_code().value()))
        .collect()
}

fn expected_shape() -> Vec<(DxfFillMeshFamily, i16)> {
    vec![
        (DxfFillMeshFamily::Hatch, 70),
        (DxfFillMeshFamily::Hatch, 91),
        (DxfFillMeshFamily::Hatch, 98),
        (DxfFillMeshFamily::Mesh, 71),
        (DxfFillMeshFamily::Mesh, 91),
        (DxfFillMeshFamily::Mesh, 90),
    ]
}

fn codes(fields: Option<&[DxfFillMeshField]>) -> Vec<i16> {
    fields
        .unwrap_or_default()
        .iter()
        .map(|field| field.group().group_code().value())
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbEntity\n8\n0\n100\nAcDbHatch\n\
70\n1\n91\n0\n98\n0\n0\nMESH\n100\nAcDbEntity\n8\n0\n100\nAcDbSubDMesh\n\
71\n2\n91\n1\n90\n0\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbEntity"),
        (8, b"0"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 70, 1)?;
    push_i32(&mut bytes, version, 91, 0)?;
    push_i32(&mut bytes, version, 98, 0)?;
    for (code, value) in [
        (0, b"MESH".as_slice()),
        (100, b"AcDbEntity"),
        (8, b"0"),
        (100, b"AcDbSubDMesh"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 71, 2)?;
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 90, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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
