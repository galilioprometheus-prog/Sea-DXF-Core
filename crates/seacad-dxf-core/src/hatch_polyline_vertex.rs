//! Conservative HATCH polyline-boundary vertex grouping.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchPolylineHeaderDirectory, DxfHatchPolylineHeaderIssue, DxfHatchPolylineHeaderState,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
};
use std::io;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexRole {
    X,
    Y,
    Bulge,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexMember {
    role: DxfHatchPolylineVertexRole,
    group: DxfRawGroup,
}
impl DxfHatchPolylineVertexMember {
    #[must_use]
    pub const fn role(self) -> DxfHatchPolylineVertexRole {
        self.role
    }
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexEntry {
    ordinal: u32,
    path_ordinal: u32,
    path_vertex_ordinal: u32,
    anchor: DxfRawGroup,
    member_start: u32,
    member_end: u32,
    y_count: u32,
    bulge_count: u32,
}
impl DxfHatchPolylineVertexEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }
    #[must_use]
    pub const fn path_ordinal(self) -> u64 {
        self.path_ordinal as u64
    }
    #[must_use]
    pub const fn path_vertex_ordinal(self) -> u64 {
        self.path_vertex_ordinal as u64
    }
    #[must_use]
    pub const fn anchor(self) -> DxfRawGroup {
        self.anchor
    }
    #[must_use]
    pub const fn x_state(self) -> DxfHatchPolylineVertexCardState {
        DxfHatchPolylineVertexCardState::Unique
    }
    #[must_use]
    pub const fn y_state(self) -> DxfHatchPolylineVertexCardState {
        state(self.y_count)
    }
    #[must_use]
    pub const fn bulge_state(self) -> DxfHatchPolylineVertexCardState {
        state(self.bulge_count)
    }
    #[must_use]
    pub const fn member_count(self) -> u64 {
        (self.member_end - self.member_start) as u64
    }
}
const fn state(n: u32) -> DxfHatchPolylineVertexCardState {
    match n {
        0 => DxfHatchPolylineVertexCardState::Absent,
        1 => DxfHatchPolylineVertexCardState::Unique,
        n => DxfHatchPolylineVertexCardState::Multiple {
            occurrence_count: n,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexCountRelation {
    Matched { count: u32 },
    Mismatched { declared: u32, observed: u32 },
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexGrouping {
    vertex_start: u32,
    vertex_end: u32,
    orphan_start: u32,
    orphan_end: u32,
    relation: DxfHatchPolylineVertexCountRelation,
}
impl DxfHatchPolylineVertexGrouping {
    #[must_use]
    pub const fn vertex_count(self) -> u64 {
        (self.vertex_end - self.vertex_start) as u64
    }
    #[must_use]
    pub const fn orphan_count(self) -> u64 {
        (self.orphan_end - self.orphan_start) as u64
    }
    #[must_use]
    pub const fn count_relation(self) -> DxfHatchPolylineVertexCountRelation {
        self.relation
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexGroupingState {
    NotPolyline,
    HeaderUnavailable(DxfHatchPolylineHeaderIssue),
    Grouped(DxfHatchPolylineVertexGrouping),
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexPathEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    state: DxfHatchPolylineVertexGroupingState,
}
impl DxfHatchPolylineVertexPathEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }
    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }
    #[must_use]
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }
    #[must_use]
    pub const fn state(self) -> DxfHatchPolylineVertexGroupingState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfHatchPolylineVertexDirectory {
    source_id: DxfSourceId,
    headers: DxfHatchPolylineHeaderDirectory,
    paths: Box<[DxfHatchPolylineVertexPathEntry]>,
    vertices: Box<[DxfHatchPolylineVertexEntry]>,
    members: Box<[DxfHatchPolylineVertexMember]>,
    orphans: Box<[DxfHatchPolylineVertexMember]>,
}
impl DxfHatchPolylineVertexDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        c: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure(c)?;
        let headers = document.hatch_polyline_header_directory(c)?;
        source(document.source_id(), headers.source_id())?;
        let mut paths = Vec::new();
        let mut vertices = Vec::new();
        let mut members = Vec::new();
        let mut orphans = Vec::new();
        paths
            .try_reserve(headers.entries().len())
            .map_err(|_| oom())?;
        for h in headers.entries().iter().copied() {
            ensure(c)?;
            let s = match h.state() {
                DxfHatchPolylineHeaderState::NotPolyline => {
                    DxfHatchPolylineVertexGroupingState::NotPolyline
                }
                DxfHatchPolylineHeaderState::Unavailable(i) => {
                    DxfHatchPolylineVertexGroupingState::HeaderUnavailable(i)
                }
                DxfHatchPolylineHeaderState::Explicit(header) => {
                    let fields = headers
                        .flag_directory()
                        .path_directory()
                        .payload_fields_for_path(h.ordinal())
                        .ok_or_else(invalid)?;
                    DxfHatchPolylineVertexGroupingState::Grouped(group(
                        fields,
                        h.ordinal(),
                        header.vertex_count().value(),
                        c,
                        &mut vertices,
                        &mut members,
                        &mut orphans,
                    )?)
                }
            };
            paths.push(DxfHatchPolylineVertexPathEntry {
                ordinal: compact(paths.len())?,
                subclass_ordinal: compact64(h.subclass_ordinal())?,
                raw_record_ordinal: compact64(h.raw_record_ordinal())?,
                state: s,
            });
        }
        ensure(c)?;
        Ok(Self {
            source_id: document.source_id(),
            headers,
            paths: paths.into_boxed_slice(),
            vertices: vertices.into_boxed_slice(),
            members: members.into_boxed_slice(),
            orphans: orphans.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }
    #[must_use]
    pub const fn header_directory(&self) -> &DxfHatchPolylineHeaderDirectory {
        &self.headers
    }
    #[must_use]
    pub fn paths(&self) -> &[DxfHatchPolylineVertexPathEntry] {
        &self.paths
    }
    #[must_use]
    pub fn vertices(&self) -> &[DxfHatchPolylineVertexEntry] {
        &self.vertices
    }
    #[must_use]
    pub fn path(&self, n: u64) -> Option<DxfHatchPolylineVertexPathEntry> {
        self.paths.get(usize::try_from(n).ok()?).copied()
    }
    #[must_use]
    pub fn vertices_for_path(&self, n: u64) -> Option<&[DxfHatchPolylineVertexEntry]> {
        let DxfHatchPolylineVertexGroupingState::Grouped(g) = self.path(n)?.state() else {
            return None;
        };
        self.vertices
            .get(usize::try_from(g.vertex_start).ok()?..usize::try_from(g.vertex_end).ok()?)
    }
    #[must_use]
    pub fn members_for_vertex(&self, n: u64) -> Option<&[DxfHatchPolylineVertexMember]> {
        let v = self.vertices.get(usize::try_from(n).ok()?)?;
        self.members
            .get(usize::try_from(v.member_start).ok()?..usize::try_from(v.member_end).ok()?)
    }
    #[must_use]
    pub fn orphans_for_path(&self, n: u64) -> Option<&[DxfHatchPolylineVertexMember]> {
        let DxfHatchPolylineVertexGroupingState::Grouped(g) = self.path(n)?.state() else {
            return None;
        };
        self.orphans
            .get(usize::try_from(g.orphan_start).ok()?..usize::try_from(g.orphan_end).ok()?)
    }
}
impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_vertex_directory(
        self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexDirectory, DxfError> {
        DxfHatchPolylineVertexDirectory::from_document(self, c)
    }
}
impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_vertex_directory(
        &self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_directory(c)
    }
}
impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_vertex_directory(
        &self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_directory(c)
    }
}
fn group(
    fields: &[crate::DxfFillMeshField],
    path: u64,
    declared: u32,
    c: &DxfCancellationToken,
    vertices: &mut Vec<DxfHatchPolylineVertexEntry>,
    members: &mut Vec<DxfHatchPolylineVertexMember>,
    orphans: &mut Vec<DxfHatchPolylineVertexMember>,
) -> Result<DxfHatchPolylineVertexGrouping, DxfError> {
    let vs = compact(vertices.len())?;
    let os = compact(orphans.len())?;
    let mut current: Option<(DxfRawGroup, u32, u32, u32)> = None;
    for f in fields {
        ensure(c)?;
        let code = f.group().group_code().value();
        if code == 10 {
            flush(path, current.take(), vertices, members)?;
            current = Some((f.group(), compact(members.len())?, 0, 0));
            members.push(DxfHatchPolylineVertexMember {
                role: DxfHatchPolylineVertexRole::X,
                group: f.group(),
            })
        } else if matches!(code, 20 | 42) {
            let role = if code == 20 {
                DxfHatchPolylineVertexRole::Y
            } else {
                DxfHatchPolylineVertexRole::Bulge
            };
            let m = DxfHatchPolylineVertexMember {
                role,
                group: f.group(),
            };
            if let Some((_, _, ref mut y, ref mut b)) = current {
                if code == 20 {
                    *y = y.checked_add(1).ok_or_else(invalid)?
                } else {
                    *b = b.checked_add(1).ok_or_else(invalid)?
                }
                members.push(m)
            } else {
                orphans.push(m)
            }
        }
    }
    flush(path, current, vertices, members)?;
    let observed = compact(vertices.len())?
        .checked_sub(vs)
        .ok_or_else(invalid)?;
    Ok(DxfHatchPolylineVertexGrouping {
        vertex_start: vs,
        vertex_end: compact(vertices.len())?,
        orphan_start: os,
        orphan_end: compact(orphans.len())?,
        relation: if declared == observed {
            DxfHatchPolylineVertexCountRelation::Matched { count: declared }
        } else {
            DxfHatchPolylineVertexCountRelation::Mismatched { declared, observed }
        },
    })
}
fn flush(
    path: u64,
    current: Option<(DxfRawGroup, u32, u32, u32)>,
    vertices: &mut Vec<DxfHatchPolylineVertexEntry>,
    members: &[DxfHatchPolylineVertexMember],
) -> Result<(), DxfError> {
    if let Some((anchor, start, y, b)) = current {
        let pv = vertices
            .iter()
            .rev()
            .take_while(|v| v.path_ordinal() == path)
            .count();
        vertices.push(DxfHatchPolylineVertexEntry {
            ordinal: compact(vertices.len())?,
            path_ordinal: compact64(path)?,
            path_vertex_ordinal: compact(pv)?,
            anchor,
            member_start: start,
            member_end: compact(members.len())?,
            y_count: y,
            bulge_count: b,
        })
    }
    Ok(())
}
fn compact(v: usize) -> Result<u32, DxfError> {
    u32::try_from(v).map_err(|_| invalid())
}
fn compact64(v: u64) -> Result<u32, DxfError> {
    u32::try_from(v).map_err(|_| invalid())
}
fn ensure(c: &DxfCancellationToken) -> Result<(), DxfError> {
    if c.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}
fn source(e: DxfSourceId, o: DxfSourceId) -> Result<(), DxfError> {
    if e == o {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch {
            expected: e,
            observed: o,
        })
    }
}
fn invalid() -> DxfError {
    ioe(io::ErrorKind::InvalidData)
}
fn oom() -> DxfError {
    ioe(io::ErrorKind::OutOfMemory)
}
fn ioe(k: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(k))
}
