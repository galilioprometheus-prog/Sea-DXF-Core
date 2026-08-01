use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityCommonFieldDomainDirectory,
    DxfEntityCommonFieldDomainIssue, DxfEntityCommonFieldDomainSemanticIssue,
    DxfEntityCommonFieldDomainSemanticValue, DxfEntityCommonFieldDomainSemantics,
    DxfEntityCommonFieldDomainValue, DxfEntityField, DxfEntityFieldSemanticIssue,
    DxfEntityFieldSemantics, DxfEntityIndexedColor, DxfEntityLineweight, DxfEntityShadowMode,
    DxfEntitySpace, DxfEntityVisibility, DxfError, DxfMemorySource, DxfRawDocumentFormat,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValue, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_reviewed_domain_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version, FixtureMode::Valid)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_domains = ascii.entity_common_field_domain_directory(&token())?;

        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version, FixtureMode::Valid)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_domains = binary.entity_common_field_domain_directory(&token())?;

        assert_valid_domains(&ascii_domains, version)?;
        assert_valid_domains(&binary_domains, version)?;
        assert_eq!(signature(&ascii_domains)?, signature(&binary_domains)?);
    }
    Ok(())
}

#[test]
fn out_of_domain_raw_scalars_are_invalid_with_exact_provenance() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, FixtureMode::InvalidDomains)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        match format {
            DxfRawDocumentFormat::Ascii => {
                let document = open_ascii(&source)?;
                assert_invalid_domains(&document.entity_common_field_domain_directory(&token())?)?;
            }
            DxfRawDocumentFormat::Binary => {
                let document = open_binary(&source)?;
                assert_invalid_domains(&document.entity_common_field_domain_directory(&token())?)?;
            }
            _ => return Err(io::Error::other("test format").into()),
        }
    }
    Ok(())
}

#[test]
fn scalar_failures_defaults_absence_and_unreviewed_sequences_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n5\n10\n100\nAcDbEntity\n8\nSECRET\n67\n.\n\
62\n1\n62\n2\n310\n0102\n310\n03\n100\nAcDbLine\n10\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let domains = document.entity_common_field_domain_directory(&token())?;
    let entity = only_entity(&domains)?;

    let paper = reviewed(&domains, entity, DxfEntityField::PAPER_SPACE)?;
    assert!(matches!(
        paper,
        DxfSemanticValue::Invalid {
            issue: DxfEntityCommonFieldDomainSemanticIssue::Field(
                DxfEntityFieldSemanticIssue::InvalidAsciiNumber(
                    DxfAsciiNumericIssue::InvalidSyntax { .. }
                )
            ),
            raw: Some(_),
            ..
        }
    ));
    let color = reviewed(&domains, entity, DxfEntityField::COLOR)?;
    assert!(matches!(
        color,
        DxfSemanticValue::Invalid {
            issue: DxfEntityCommonFieldDomainSemanticIssue::Field(
                DxfEntityFieldSemanticIssue::MultipleValues {
                    occurrence_count: 2
                }
            ),
            raw: None,
            ..
        }
    ));
    assert_eq!(
        reviewed(&domains, entity, DxfEntityField::VISIBILITY)?.value(),
        Some(&DxfEntityCommonFieldDomainValue::Visibility(
            DxfEntityVisibility::Visible
        ))
    );
    assert_eq!(
        reviewed(&domains, entity, DxfEntityField::LINETYPE_SCALE)?.state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        reviewed(&domains, entity, DxfEntityField::TRUE_COLOR)?.state(),
        DxfSemanticValueState::Absent
    );
    assert!(matches!(
        entry(&domains, entity, DxfEntityField::LAYER)?.semantics(),
        DxfEntityCommonFieldDomainSemantics::Unreviewed(DxfEntityFieldSemantics::Singleton(_))
    ));
    assert_eq!(
        entry(&domains, entity, DxfEntityField::PROXY_GRAPHICS_DATA)?.semantics(),
        DxfEntityCommonFieldDomainSemantics::Unreviewed(DxfEntityFieldSemantics::OpaqueSequence {
            occurrence_count: 2
        })
    );
    assert!(!format!("{domains:?}").contains("SECRET"));
    Ok(())
}

#[test]
fn cancellation_source_identity_lookup_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityCommonFieldDomainDirectory>();
    assert_copy::<seacad_dxf_core::DxfEntityCommonFieldDomainEntry>();
    assert_copy::<DxfEntityCommonFieldDomainSemantics>();
    let entry_size = std::mem::size_of::<seacad_dxf_core::DxfEntityCommonFieldDomainEntry>();
    assert!(entry_size <= 512, "domain entry size: {entry_size}");

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureMode::Valid,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_common_field_domain_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let domains = document.entity_common_field_domain_directory(&token())?;
    assert_eq!(domains.entry(u64::MAX), None);
    assert_eq!(domains.source_id(), document.source_id());
    assert_eq!(domains.source_directory().source_id(), document.source_id());
    assert_eq!(domains.entries().len(), 19);
    assert_eq!(
        domains.entries_for_entity(only_entity(&domains)?)?.len(),
        19
    );

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1027,
        FixtureMode::Valid,
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_domains = other.entity_common_field_domain_directory(&token())?;
    assert!(matches!(
        domains.entries_for_entity(only_entity(&other_domains)?),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_valid_domains(
    domains: &DxfEntityCommonFieldDomainDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(domains.entries().len(), 19);
    let entity = only_entity(domains)?;
    assert_eq!(
        reviewed(domains, entity, DxfEntityField::PAPER_SPACE)?.value(),
        Some(&DxfEntityCommonFieldDomainValue::Space(
            DxfEntitySpace::Paper
        ))
    );
    assert_eq!(
        reviewed(domains, entity, DxfEntityField::COLOR)?.value(),
        Some(&DxfEntityCommonFieldDomainValue::IndexedColor(
            DxfEntityIndexedColor::from_raw(-255).ok_or(io::Error::other("color"))?
        ))
    );
    assert_eq!(
        reviewed(domains, entity, DxfEntityField::VISIBILITY)?.value(),
        Some(&DxfEntityCommonFieldDomainValue::Visibility(
            DxfEntityVisibility::Invisible
        ))
    );
    assert!(matches!(
        reviewed(domains, entity, DxfEntityField::PROXY_GRAPHICS_SIZE)?.value(),
        Some(DxfEntityCommonFieldDomainValue::ProxyGraphicsSize(0))
    ));
    if version == DxfAcadVersion::Ac1009 {
        for field in [DxfEntityField::LINEWEIGHT, DxfEntityField::SHADOW] {
            assert!(matches!(
                reviewed(domains, entity, field)?,
                DxfSemanticValue::Invalid {
                    issue: DxfEntityCommonFieldDomainSemanticIssue::Field(
                        DxfEntityFieldSemanticIssue::MissingRequired
                    ),
                    raw: None,
                    ..
                }
            ));
        }
        assert_eq!(
            reviewed(domains, entity, DxfEntityField::TRUE_COLOR)?.state(),
            DxfSemanticValueState::Absent
        );
    } else {
        assert_eq!(
            reviewed(domains, entity, DxfEntityField::LINEWEIGHT)?.value(),
            Some(&DxfEntityCommonFieldDomainValue::Lineweight(
                DxfEntityLineweight::from_raw(211).ok_or(io::Error::other("lineweight"))?
            ))
        );
        assert_eq!(
            reviewed(domains, entity, DxfEntityField::SHADOW)?.value(),
            Some(&DxfEntityCommonFieldDomainValue::ShadowMode(
                DxfEntityShadowMode::Ignores
            ))
        );
    }
    assert!(matches!(
        entry(domains, entity, DxfEntityField::LAYER)?.semantics(),
        DxfEntityCommonFieldDomainSemantics::Unreviewed(_)
    ));
    Ok(())
}

fn assert_invalid_domains(
    domains: &DxfEntityCommonFieldDomainDirectory,
) -> Result<(), Box<dyn Error>> {
    let entity = only_entity(domains)?;
    for field in [
        DxfEntityField::PAPER_SPACE,
        DxfEntityField::COLOR,
        DxfEntityField::LINEWEIGHT,
        DxfEntityField::LINETYPE_SCALE,
        DxfEntityField::VISIBILITY,
        DxfEntityField::PROXY_GRAPHICS_SIZE,
        DxfEntityField::TRUE_COLOR,
        DxfEntityField::SHADOW,
    ] {
        let value = reviewed(domains, entity, field)?;
        let DxfSemanticValue::Invalid {
            issue: DxfEntityCommonFieldDomainSemanticIssue::Domain(issue),
            raw: Some(raw),
            ..
        } = value
        else {
            return Err(io::Error::other(format!("expected invalid domain: {field:?}")).into());
        };
        let issue_field = match issue {
            DxfEntityCommonFieldDomainIssue::UnsupportedInt16 { field, .. }
            | DxfEntityCommonFieldDomainIssue::UnsupportedInt32 { field, .. }
            | DxfEntityCommonFieldDomainIssue::InvalidDouble { field, .. }
            | DxfEntityCommonFieldDomainIssue::ValueKindMismatch { field, .. } => field,
            _ => return Err(io::Error::other("unknown domain issue").into()),
        };
        assert_eq!(issue_field, field);
        assert!(!raw.value_span().is_empty());
    }
    Ok(())
}

type Signature = Vec<(DxfEntityField, bool, u8)>;

fn signature(domains: &DxfEntityCommonFieldDomainDirectory) -> Result<Signature, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in domains.entries().iter().copied() {
        let (reviewed, state) = match entry.semantics() {
            DxfEntityCommonFieldDomainSemantics::Reviewed(value) => (true, value.state() as u8),
            DxfEntityCommonFieldDomainSemantics::Unreviewed(
                DxfEntityFieldSemantics::Singleton(value),
            ) => (false, value.state() as u8),
            DxfEntityCommonFieldDomainSemantics::Unreviewed(
                DxfEntityFieldSemantics::OpaqueSequence { .. },
            ) => (false, u8::MAX),
            _ => return Err(io::Error::other("unknown domain semantics").into()),
        };
        result.push((entry.field(), reviewed, state));
    }
    Ok(result)
}

fn reviewed(
    domains: &DxfEntityCommonFieldDomainDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityCommonFieldDomainSemanticValue, Box<dyn Error>> {
    match entry(domains, entity, field)?.semantics() {
        DxfEntityCommonFieldDomainSemantics::Reviewed(value) => Ok(value),
        _ => Err(io::Error::other("reviewed domain").into()),
    }
}

fn entry(
    domains: &DxfEntityCommonFieldDomainDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityCommonFieldDomainEntry, Box<dyn Error>> {
    domains
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("domain entry").into())
}

fn only_entity(
    domains: &DxfEntityCommonFieldDomainDirectory,
) -> Result<seacad_dxf_core::DxfEntityRef, Box<dyn Error>> {
    let [entity] = domains
        .source_directory()
        .evidence_directory()
        .entity_directory()
        .entities()
    else {
        return Err(io::Error::other("one entity").into());
    };
    Ok(*entity)
}

#[derive(Clone, Copy)]
enum FixtureMode {
    Valid,
    InvalidDomains,
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Int32(i32),
    Double(f64),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    mode: FixtureMode,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    match mode {
        FixtureMode::Valid => groups.extend([
            (67, Value::Int16(1)),
            (8, Value::Text(b"SECRET")),
            (62, Value::Int16(-255)),
            (48, Value::Double(0.0)),
            (60, Value::Int16(1)),
            (92, Value::Int32(0)),
        ]),
        FixtureMode::InvalidDomains => groups.extend([
            (67, Value::Int16(2)),
            (8, Value::Text(b"SECRET")),
            (62, Value::Int16(257)),
            (370, Value::Int16(-4)),
            (48, Value::Double(-1.0)),
            (60, Value::Int16(2)),
            (92, Value::Int32(-1)),
            (420, Value::Int32(0x01_00_00_00)),
            (284, Value::Int16(4)),
        ]),
    }
    if version != DxfAcadVersion::Ac1009 && matches!(mode, FixtureMode::Valid) {
        groups.extend([
            (370, Value::Int16(211)),
            (420, Value::Int32(0x12_34_56)),
            (284, Value::Int16(3)),
        ]);
    }
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([
        (10, Value::Double(0.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.extend_from_slice(b"\r\n");
    }
    bytes
}

fn binary_groups(
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        match value {
            Value::Text(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
    }
    Ok(bytes)
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
