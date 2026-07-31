//! Internal scalar projection helpers for embedded MTEXT columns.

use std::io;

use crate::{
    DxfDouble, DxfError, DxfIoOperation, DxfMTextColumnBooleanSemantic,
    DxfMTextColumnCountSemantic, DxfMTextColumnDoubleSemantic, DxfMTextColumnHeightDisposition,
    DxfMTextColumnIssue, DxfMTextColumnSemantics, DxfMTextColumnSourceEntry, DxfMTextColumnType,
    DxfMTextColumnTypeSemantic, DxfMTextEmbeddedColumnRole, DxfMTextEmbeddedColumnValue,
    DxfMTextFlatColumnRole, DxfMTextFlatColumnValue, DxfMTextXDataColumnRole,
    DxfMTextXDataColumnValue, DxfRawGroup, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId, DxfTextSymbolValueData,
};

const NAMESPACE: &str = "entity.mtext.embedded_columns";

pub(super) fn project_entry<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    entry: DxfMTextColumnSourceEntry,
    values: &[V],
    height_disposition: DxfMTextColumnHeightDisposition,
    heights: &mut Vec<DxfMTextColumnDoubleSemantic>,
) -> Result<DxfMTextColumnSemantics, DxfError> {
    let height_start = compact_len(heights.len())?;
    for value in values
        .iter()
        .copied()
        .filter(|value| value.role() == Some(DxfMTextEmbeddedColumnRole::ColumnHeight))
    {
        heights.try_reserve(1).map_err(|_| out_of_memory())?;
        heights.push(project_nonnegative_double(source_id, value)?);
    }
    Ok(DxfMTextColumnSemantics {
        entry,
        column_type: project_column_type(source_id, values)?,
        column_count: project_count(source_id, values)?,
        column_width: project_unique_double(
            source_id,
            values,
            DxfMTextEmbeddedColumnRole::ColumnWidth,
            true,
        )?,
        column_gutter: project_unique_double(
            source_id,
            values,
            DxfMTextEmbeddedColumnRole::ColumnGutter,
            false,
        )?,
        auto_height: project_boolean(
            source_id,
            values,
            DxfMTextEmbeddedColumnRole::ColumnAutoHeight,
        )?,
        flow_reversed: project_boolean(
            source_id,
            values,
            DxfMTextEmbeddedColumnRole::ColumnFlowReversed,
        )?,
        shared_height: project_unique_double(
            source_id,
            values,
            DxfMTextEmbeddedColumnRole::SharedHeight,
            false,
        )?,
        height_disposition,
        height_start,
        height_end: compact_len(heights.len())?,
    })
}

fn project_column_type<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    values: &[V],
) -> Result<DxfMTextColumnTypeSemantic, DxfError> {
    let role = DxfMTextEmbeddedColumnRole::ColumnType;
    let field = field(source_id, role);
    match unique(values, role) {
        Unique::Absent => Ok(DxfSemanticValue::invalid(
            DxfMTextColumnIssue::MissingColumnType,
            field,
            None,
        )),
        Unique::Multiple(first, count) => invalid_multiple(field, first, count),
        Unique::One(value) => project_i16(field, value, |code| {
            Ok(match code {
                0 => DxfMTextColumnType::NoColumns,
                1 => DxfMTextColumnType::Static,
                2 => DxfMTextColumnType::Dynamic,
                _ => return Err(DxfMTextColumnIssue::UnsupportedColumnType { code }),
            })
        }),
    }
}

fn project_count<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    values: &[V],
) -> Result<DxfMTextColumnCountSemantic, DxfError> {
    project_unique_i16(
        source_id,
        values,
        DxfMTextEmbeddedColumnRole::ColumnCount,
        |count| u16::try_from(count).map_err(|_| DxfMTextColumnIssue::NegativeCount { count }),
    )
}

fn project_boolean<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    values: &[V],
    role: DxfMTextEmbeddedColumnRole,
) -> Result<DxfMTextColumnBooleanSemantic, DxfError> {
    project_unique_i16(source_id, values, role, |code| match code {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DxfMTextColumnIssue::BooleanOutOfDomain { role, code }),
    })
}

fn project_unique_i16<T: Copy, V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    values: &[V],
    role: DxfMTextEmbeddedColumnRole,
    classify: impl FnOnce(i16) -> Result<T, DxfMTextColumnIssue>,
) -> Result<DxfSemanticValue<T, DxfMTextColumnIssue>, DxfError> {
    let field = field(source_id, role);
    match unique(values, role) {
        Unique::Absent => Ok(DxfSemanticValue::absent(field)),
        Unique::Multiple(first, count) => invalid_multiple(field, first, count),
        Unique::One(value) => project_i16(field, value, classify),
    }
}

fn project_unique_double<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    values: &[V],
    role: DxfMTextEmbeddedColumnRole,
    positive: bool,
) -> Result<DxfMTextColumnDoubleSemantic, DxfError> {
    let field = field(source_id, role);
    match unique(values, role) {
        Unique::Absent => Ok(DxfSemanticValue::absent(field)),
        Unique::Multiple(first, count) => invalid_multiple(field, first, count),
        Unique::One(value) if positive => project_positive_double(source_id, value),
        Unique::One(value) => project_double(field, value, |number| {
            if number.to_f64() >= 0.0 {
                Ok(number)
            } else {
                Err(DxfMTextColumnIssue::NegativeValue {
                    role,
                    value: number,
                })
            }
        }),
    }
}

fn project_positive_double<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    value: V,
) -> Result<DxfMTextColumnDoubleSemantic, DxfError> {
    let role = value.role().ok_or_else(invalid_internal_data)?;
    project_double(field(source_id, role), value, |number| {
        if number.to_f64() > 0.0 {
            Ok(number)
        } else {
            Err(DxfMTextColumnIssue::NonPositiveValue {
                role,
                value: number,
            })
        }
    })
}

fn project_nonnegative_double<V: ColumnEvidenceValue>(
    source_id: DxfSourceId,
    value: V,
) -> Result<DxfMTextColumnDoubleSemantic, DxfError> {
    let role = value.role().ok_or_else(invalid_internal_data)?;
    project_double(field(source_id, role), value, |number| {
        if number.to_f64() >= 0.0 {
            Ok(number)
        } else {
            Err(DxfMTextColumnIssue::NegativeValue {
                role,
                value: number,
            })
        }
    })
}

fn project_i16<T: Copy, V: ColumnEvidenceValue>(
    field: DxfSemanticFieldProvenance,
    value: V,
    classify: impl FnOnce(i16) -> Result<T, DxfMTextColumnIssue>,
) -> Result<DxfSemanticValue<T, DxfMTextColumnIssue>, DxfError> {
    let raw = raw(value)?;
    match value.data() {
        DxfTextSymbolValueData::Int16(Ok(number)) => match classify(number) {
            Ok(number) => Ok(DxfSemanticValue::explicit(number, field, raw)),
            Err(issue) => Ok(DxfSemanticValue::invalid(issue, field, Some(raw))),
        },
        DxfTextSymbolValueData::Int16(Err(issue)) => Ok(DxfSemanticValue::invalid(
            DxfMTextColumnIssue::Numeric(issue),
            field,
            Some(raw),
        )),
        _ => Err(invalid_internal_data()),
    }
}

fn project_double<T: Copy, V: ColumnEvidenceValue>(
    field: DxfSemanticFieldProvenance,
    value: V,
    classify: impl FnOnce(DxfDouble) -> Result<T, DxfMTextColumnIssue>,
) -> Result<DxfSemanticValue<T, DxfMTextColumnIssue>, DxfError> {
    let raw = raw(value)?;
    match value.data() {
        DxfTextSymbolValueData::Double(Ok(number)) => match classify(number) {
            Ok(number) => Ok(DxfSemanticValue::explicit(number, field, raw)),
            Err(issue) => Ok(DxfSemanticValue::invalid(issue, field, Some(raw))),
        },
        DxfTextSymbolValueData::Double(Err(issue)) => Ok(DxfSemanticValue::invalid(
            DxfMTextColumnIssue::Numeric(issue),
            field,
            Some(raw),
        )),
        _ => Err(invalid_internal_data()),
    }
}

fn invalid_multiple<T, V: ColumnEvidenceValue>(
    field: DxfSemanticFieldProvenance,
    first: V,
    occurrence_count: u64,
) -> Result<DxfSemanticValue<T, DxfMTextColumnIssue>, DxfError> {
    Ok(DxfSemanticValue::invalid(
        DxfMTextColumnIssue::MultipleValues {
            role: first.role().ok_or_else(invalid_internal_data)?,
            occurrence_count,
        },
        field,
        Some(raw(first)?),
    ))
}

enum Unique<V> {
    Absent,
    One(V),
    Multiple(V, u64),
}

fn unique<V: ColumnEvidenceValue>(values: &[V], role: DxfMTextEmbeddedColumnRole) -> Unique<V> {
    let mut matching = values
        .iter()
        .copied()
        .filter(|value| value.role() == Some(role));
    let Some(first) = matching.next() else {
        return Unique::Absent;
    };
    let count = 1 + matching.count() as u64;
    if count == 1 {
        Unique::One(first)
    } else {
        Unique::Multiple(first, count)
    }
}

fn field(source_id: DxfSourceId, role: DxfMTextEmbeddedColumnRole) -> DxfSemanticFieldProvenance {
    let field_id = match role {
        DxfMTextEmbeddedColumnRole::Version => "version",
        DxfMTextEmbeddedColumnRole::SharedHeight => "shared_height",
        DxfMTextEmbeddedColumnRole::ColumnHeight => "column_height",
        DxfMTextEmbeddedColumnRole::ColumnType => "column_type",
        DxfMTextEmbeddedColumnRole::ColumnCount => "column_count",
        DxfMTextEmbeddedColumnRole::ColumnWidth => "column_width",
        DxfMTextEmbeddedColumnRole::ColumnGutter => "column_gutter",
        DxfMTextEmbeddedColumnRole::ColumnAutoHeight => "auto_height",
        DxfMTextEmbeddedColumnRole::ColumnFlowReversed => "flow_reversed",
    };
    DxfSemanticFieldProvenance::new(source_id, NAMESPACE, field_id)
}

fn raw(value: impl ColumnEvidenceValue) -> Result<DxfRawValueProvenance, DxfError> {
    let group = value.group();
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
}

pub(super) trait ColumnEvidenceValue: Copy {
    fn role(self) -> Option<DxfMTextEmbeddedColumnRole>;
    fn data(self) -> DxfTextSymbolValueData;
    fn group(self) -> DxfRawGroup;
}

impl ColumnEvidenceValue for DxfMTextEmbeddedColumnValue {
    fn role(self) -> Option<DxfMTextEmbeddedColumnRole> {
        Some(self.role())
    }

    fn data(self) -> DxfTextSymbolValueData {
        self.data()
    }

    fn group(self) -> DxfRawGroup {
        self.group()
    }
}

impl ColumnEvidenceValue for DxfMTextFlatColumnValue {
    fn role(self) -> Option<DxfMTextEmbeddedColumnRole> {
        use DxfMTextEmbeddedColumnRole as R;
        Some(match self.role() {
            DxfMTextFlatColumnRole::ColumnType => R::ColumnType,
            DxfMTextFlatColumnRole::ColumnCount => R::ColumnCount,
            DxfMTextFlatColumnRole::ColumnFlowReversed => R::ColumnFlowReversed,
            DxfMTextFlatColumnRole::ColumnAutoHeight => R::ColumnAutoHeight,
            DxfMTextFlatColumnRole::ColumnWidth => R::ColumnWidth,
            DxfMTextFlatColumnRole::ColumnGutter => R::ColumnGutter,
            DxfMTextFlatColumnRole::RotationOrColumnHeight => return None,
        })
    }

    fn data(self) -> DxfTextSymbolValueData {
        self.data()
    }

    fn group(self) -> DxfRawGroup {
        self.group()
    }
}

impl ColumnEvidenceValue for DxfMTextXDataColumnValue {
    fn role(self) -> Option<DxfMTextEmbeddedColumnRole> {
        use DxfMTextEmbeddedColumnRole as R;
        Some(match self.role() {
            DxfMTextXDataColumnRole::ColumnType => R::ColumnType,
            DxfMTextXDataColumnRole::ColumnAutoHeight => R::ColumnAutoHeight,
            DxfMTextXDataColumnRole::ColumnCount => R::ColumnCount,
            DxfMTextXDataColumnRole::ColumnFlowReversed => R::ColumnFlowReversed,
            DxfMTextXDataColumnRole::ColumnWidth => R::ColumnWidth,
            DxfMTextXDataColumnRole::ColumnGutter => R::ColumnGutter,
            DxfMTextXDataColumnRole::ColumnHeight => R::ColumnHeight,
            DxfMTextXDataColumnRole::DefinedHeight => R::SharedHeight,
            DxfMTextXDataColumnRole::ColumnHeightCount => return None,
        })
    }

    fn data(self) -> DxfTextSymbolValueData {
        self.data()
    }

    fn group(self) -> DxfRawGroup {
        self.value_group()
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}
