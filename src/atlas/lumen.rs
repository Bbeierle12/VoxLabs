//! The lumen surface mesh (`tract_lumen_v2.bin`, magic `VTLUMN2`, schema
//! 2) and the mesh kernels, ported from the decompiled `TractLumenAsset`
//! and `TractMeshCpu`: the binary layout with its CRC-32, the constructor
//! checks (including the app's own ring-area consistency test), `morph`
//! (ring offsets scaled by the square root of the area ratio),
//! `resampleArea`, `computeVertexNormals` and `expandUncertainty`.
//!
//! Frame: the mesh is in the renderer's own millimetre frame as the file
//! stores it. The transform to the plan's "VTL canonical" `TractGeometry`
//! frame is not recorded anywhere in the APK, so it is reported as
//! unknown (`FRAME`), not invented.

use std::sync::OnceLock;

use crate::hash::{crc32, sha256_hex};

pub const BYTES: &[u8] = include_bytes!("../../assets/vocal_tract_lab/tract_lumen_v2.bin");
pub const SHA256: &str = "8ca7be68088afbe93f3327f52db1f34a790217e8af130cb47c39acecbd472006";
pub const FRAME: &str = "lumen-mm: the renderer's millimetre frame from tract_lumen_v2.bin; the transform to the \
     VTL canonical frame is not recorded in the source APK (unknown)";

const MAGIC: [u8; 8] = *b"VTLUMN2\0";
const HEADER_BYTES: usize = 104;
const SCHEMA_VERSION: u16 = 2;
/// The app's format limits: sections and angular samples in 8..=256,
/// angular even.
const COUNT_RANGE: std::ops::RangeInclusive<usize> = 8..=256;
/// The app's asset checks: the first/last section positions must be 0/1
/// within this, and each ring's polygon area must match the stored
/// reference area within this relative error.
const POSITION_EPS: f32 = 1.0e-5;
const RING_AREA_REL_EPS: f32 = 0.002;
/// The normal's length floor (the app's `max(1e-20, |n|²)`).
const NORMAL_EPS: f32 = 1.0e-20;

#[derive(Clone, Debug, PartialEq)]
pub struct Lumen {
    pub schema_version: u16,
    pub sections: usize,
    pub angular: usize,
    pub section_position: Vec<f32>,
    pub reference_area_cm2: Vec<f32>,
    /// `sections × 3`, mm.
    pub centerline_mm: Vec<f32>,
    /// `sections × angular × 3`, mm, relative to the section's centerline point.
    pub ring_offsets_mm: Vec<f32>,
    /// `sections × 6 × angular` vertex indices.
    pub triangle_indices: Vec<u16>,
    pub minimum_area_ratio: f32,
    pub maximum_area_ratio: f32,
    pub provenance_kind: u16,
    pub provenance: &'static str,
    pub source_sha256: [String; 2],
    pub model_id: &'static str,
    /// `sections × angular + 2` (two apex vertices).
    pub vertex_count: usize,
    /// Which section each vertex belongs to (apices → first/last section).
    pub section_for_vertex: Vec<usize>,
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.pos + n;
        if end > self.b.len() {
            return Err("lumen: read past the end".into());
        }
        let s = &self.b[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, String> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn i32(&mut self) -> Result<i32, String> {
        let s = self.take(4)?;
        Ok(i32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(self.i32()? as u32)
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn f32s(&mut self, n: usize) -> Result<Vec<f32>, String> {
        (0..n).map(|_| self.f32()).collect()
    }
    fn hex32(&mut self) -> Result<String, String> {
        Ok(self.take(32)?.iter().map(|b| format!("{b:02x}")).collect())
    }
}

impl Lumen {
    /// `TractLumenAsset.parse` then the constructor checks.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_BYTES {
            return Err("lumen asset is shorter than its header".into());
        }
        let mut r = Reader { b: bytes, pos: 0 };
        if r.take(8)? != MAGIC {
            return Err("invalid lumen asset magic".into());
        }
        let schema = r.u16()?;
        let header = r.u16()? as usize;
        let sections = r.u16()? as usize;
        let angular = r.u16()? as usize;
        let vertex_count = r.i32()?;
        let index_count = r.i32()?;
        let min_ratio = r.f32()?;
        let max_ratio = r.f32()?;
        let crc = r.u32()?;
        let provenance_kind = r.u16()?;
        let reserved = r.u16()?;
        let sha_a = r.hex32()?;
        let sha_b = r.hex32()?;
        if schema != SCHEMA_VERSION {
            return Err(format!("unsupported lumen schema {schema}"));
        }
        if header != HEADER_BYTES {
            return Err(format!("invalid lumen header size {header}"));
        }
        if reserved != 0 {
            return Err("nonzero reserved lumen-header field".into());
        }
        if !COUNT_RANGE.contains(&sections)
            || !COUNT_RANGE.contains(&angular)
            || !angular.is_multiple_of(2)
        {
            return Err(format!("lumen: invalid counts {sections} × {angular}"));
        }
        let ring_vertices = sections * angular;
        if vertex_count != (ring_vertices + 2) as i32 {
            return Err(format!(
                "lumen: vertex count {vertex_count} ≠ {}",
                ring_vertices + 2
            ));
        }
        if index_count != (sections * 6 * angular) as i32 {
            return Err(format!("lumen: index count {index_count}"));
        }
        let index_count = index_count as usize;
        let expected = header + sections * 4 * (angular * 3 + 5) + index_count * 2;
        if bytes.len() != expected {
            return Err(format!(
                "lumen asset payload length {} does not match its header ({expected})",
                bytes.len()
            ));
        }
        if crc32(&bytes[header..]) != crc {
            return Err("lumen asset CRC32 mismatch".into());
        }
        r.pos = header;
        let section_position = r.f32s(sections)?;
        let reference_area_cm2 = r.f32s(sections)?;
        let centerline_mm = r.f32s(sections * 3)?;
        let ring_offsets_mm = r.f32s(ring_vertices * 3)?;
        let mut triangle_indices = Vec::with_capacity(index_count);
        for _ in 0..index_count {
            triangle_indices.push(r.u16()?);
        }
        if r.pos != bytes.len() {
            return Err("unexpected trailing lumen bytes".into());
        }
        let (provenance, model_id) = match provenance_kind {
            1 => (
                "Equal 50/50 male_classical_a and female_classical_a engineering-template blend",
                "vt3d-sex-informed-a-blend-v2",
            ),
            2 => (
                "Frozen 0.6.3 metric-MRI engineering mean; five subjects; scientific gate closed",
                "vt3d-metric-mri-engineering-mean-v0.6.3",
            ),
            k => return Err(format!("unsupported lumen provenance {k}")),
        };
        let mut section_for_vertex = vec![0usize; ring_vertices + 2];
        for (i, s) in section_for_vertex.iter_mut().enumerate() {
            *s = if i >= ring_vertices {
                if i == ring_vertices { 0 } else { sections - 1 }
            } else {
                i / angular
            };
        }
        let l = Self {
            schema_version: schema,
            sections,
            angular,
            section_position,
            reference_area_cm2,
            centerline_mm,
            ring_offsets_mm,
            triangle_indices,
            minimum_area_ratio: min_ratio,
            maximum_area_ratio: max_ratio,
            provenance_kind,
            provenance,
            source_sha256: [sha_a, sha_b],
            model_id,
            vertex_count: ring_vertices + 2,
            section_for_vertex,
        };
        l.validate()?;
        Ok(l)
    }

    fn validate(&self) -> Result<(), String> {
        let finite = |v: &[f32]| v.iter().all(|x| x.is_finite());
        if !(self.minimum_area_ratio.is_finite() && self.maximum_area_ratio.is_finite())
            || self.minimum_area_ratio <= 0.0
            || self.maximum_area_ratio < self.minimum_area_ratio
        {
            return Err("lumen: invalid area ratio limits".into());
        }
        if !finite(&self.section_position)
            || !finite(&self.centerline_mm)
            || !finite(&self.ring_offsets_mm)
        {
            return Err("lumen: non-finite geometry".into());
        }
        if !self
            .reference_area_cm2
            .iter()
            .all(|&a| a.is_finite() && a > 0.0)
        {
            return Err("lumen: reference areas must be finite and positive".into());
        }
        if self
            .triangle_indices
            .iter()
            .any(|&i| i as usize >= self.vertex_count)
        {
            return Err("lumen: triangle index out of range".into());
        }
        if self.section_position[0].abs() >= POSITION_EPS
            || (self.section_position[self.sections - 1] - 1.0).abs() >= POSITION_EPS
        {
            return Err("lumen: section positions must span 0..1".into());
        }
        if !self.section_position.windows(2).all(|w| w[1] > w[0]) {
            return Err("lumen: section positions must increase".into());
        }
        for s in 0..self.sections {
            let ring = self.ring_area_cm2(s);
            let reference = self.reference_area_cm2[s];
            if ring <= 0.0 || (ring - reference).abs() / reference >= RING_AREA_REL_EPS {
                return Err(format!(
                    "lumen: reference area mismatch at section {s}: {ring} vs {reference}"
                ));
            }
        }
        Ok(())
    }

    /// Polygon area of one ring (shoelace on the offsets), cm².
    pub fn ring_area_cm2(&self, section: usize) -> f32 {
        let a = self.angular;
        let base = section * a * 3;
        let (mut x, mut y, mut z) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..a {
            let p = base + i * 3;
            let q = base + ((i + 1) % a) * 3;
            let o = &self.ring_offsets_mm;
            let (px, py, pz) = (o[p] as f64, o[p + 1] as f64, o[p + 2] as f64);
            let (qx, qy, qz) = (o[q] as f64, o[q + 1] as f64, o[q + 2] as f64);
            x += py * qz - pz * qy;
            y += pz * qx - qz * px;
            z += px * qy - py * qx;
        }
        (((x * x + y * y + z * z).sqrt() * 0.5) / 100.0) as f32
    }

    /// `TractLumenAsset.morph`: each ring's offsets scaled by
    /// sqrt(clamp(area / reference, min, max)) around its centerline point;
    /// the two apices at the centerline ends. `out` is `vertex_count × 3`.
    pub fn morph_into(&self, area_cm2: &[f32], out: &mut [f32]) -> Result<(), String> {
        if area_cm2.len() != self.sections {
            return Err(format!(
                "expected {} area samples, received {}",
                self.sections,
                area_cm2.len()
            ));
        }
        if area_cm2.iter().any(|a| !a.is_finite() || *a <= 0.0) {
            return Err("area samples must be finite and positive".into());
        }
        if out.len() != self.vertex_count * 3 {
            return Err(format!(
                "morph: {} slots for {} vertices",
                out.len(),
                self.vertex_count
            ));
        }
        for (s, &area) in area_cm2.iter().enumerate() {
            let scale = (area / self.reference_area_cm2[s])
                .clamp(self.minimum_area_ratio, self.maximum_area_ratio)
                .sqrt();
            let c = s * 3;
            for i in 0..self.angular {
                let v = (self.angular * s + i) * 3;
                for k in 0..3 {
                    out[v + k] = self.centerline_mm[c + k] + self.ring_offsets_mm[v + k] * scale;
                }
            }
        }
        let first = (self.vertex_count - 2) * 3;
        let last = (self.vertex_count - 1) * 3;
        let end = (self.sections - 1) * 3;
        out[first..first + 3].copy_from_slice(&self.centerline_mm[..3]);
        out[last..last + 3].copy_from_slice(&self.centerline_mm[end..end + 3]);
        Ok(())
    }

    /// `TractMeshCpu.expandUncertainty`: vertices pushed away from their
    /// section's centerline point by sqrt(1 + clamp(rel_std, 0, max)).
    pub fn expand_uncertainty_into(
        &self,
        vertices: &[f32],
        relative_area_std: f32,
        expand_max: f32,
        out: &mut [f32],
    ) -> Result<(), String> {
        if vertices.len() != self.vertex_count * 3 || out.len() != vertices.len() {
            return Err("expand_uncertainty: vertex buffer size".into());
        }
        let scale = (relative_area_std.clamp(0.0, expand_max) + 1.0).sqrt();
        for i in 0..self.vertex_count {
            let c = self.section_for_vertex[i].min(self.sections - 1) * 3;
            for k in 0..3 {
                let center = self.centerline_mm[c + k];
                out[i * 3 + k] = center + (vertices[i * 3 + k] - center) * scale;
            }
        }
        Ok(())
    }

    /// Arc length of the centerline, mm.
    pub fn centerline_length_mm(&self) -> f32 {
        let c = &self.centerline_mm;
        (1..self.sections)
            .map(|s| {
                let (a, b) = ((s - 1) * 3, s * 3);
                ((c[b] - c[a]).powi(2)
                    + (c[b + 1] - c[a + 1]).powi(2)
                    + (c[b + 2] - c[a + 2]).powi(2))
                .sqrt()
            })
            .sum()
    }
}

/// `TractMeshCpu.computeVertexNormals`: area-weighted face normals summed
/// per vertex, normalized. `out` is `vertices.len()`.
pub fn vertex_normals_into(
    vertices: &[f32],
    triangles: &[u16],
    out: &mut [f32],
) -> Result<(), String> {
    if !vertices.len().is_multiple_of(3)
        || !triangles.len().is_multiple_of(3)
        || out.len() != vertices.len()
    {
        return Err("vertex_normals: buffer sizes".into());
    }
    out.fill(0.0);
    for t in triangles.chunks_exact(3) {
        let (a, b, c) = (t[0] as usize * 3, t[1] as usize * 3, t[2] as usize * 3);
        if a + 2 >= vertices.len() || b + 2 >= vertices.len() || c + 2 >= vertices.len() {
            return Err("vertex_normals: index out of range".into());
        }
        let e1 = [
            vertices[b] - vertices[a],
            vertices[b + 1] - vertices[a + 1],
            vertices[b + 2] - vertices[a + 2],
        ];
        let e2 = [
            vertices[c] - vertices[a],
            vertices[c + 1] - vertices[a + 1],
            vertices[c + 2] - vertices[a + 2],
        ];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for v in [a, b, c] {
            for k in 0..3 {
                out[v + k] += n[k];
            }
        }
    }
    for n in out.chunks_exact_mut(3) {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2])
            .max(NORMAL_EPS)
            .sqrt();
        n[0] /= len;
        n[1] /= len;
        n[2] /= len;
    }
    Ok(())
}

/// `TractMeshCpu.resampleArea`: piecewise-linear resampling of an area
/// function from `source_position` (increasing) to `target_position`,
/// clamped to the end values outside the source range and floored at
/// `min_area`.
pub fn resample_area_into(
    source_position: &[f32],
    source_area_cm2: &[f32],
    target_position: &[f32],
    min_area: f32,
    out: &mut [f32],
) -> Result<(), String> {
    if source_position.len() != source_area_cm2.len() || source_position.is_empty() {
        return Err("resample_area: source lengths".into());
    }
    if out.len() != target_position.len() {
        return Err("resample_area: target length".into());
    }
    if !source_position
        .iter()
        .chain(target_position)
        .all(|p| p.is_finite())
        || source_area_cm2.iter().any(|a| !a.is_finite() || *a <= 0.0)
    {
        return Err("resample_area: non-finite or non-positive input".into());
    }
    if !source_position.windows(2).all(|w| w[1] > w[0]) {
        return Err("resample_area: source positions must increase".into());
    }
    let n = source_position.len();
    for (o, &t) in out.iter_mut().zip(target_position) {
        let v = if t <= source_position[0] {
            source_area_cm2[0]
        } else if t >= source_position[n - 1] {
            source_area_cm2[n - 1]
        } else {
            let mut i = 1;
            while source_position[i] < t {
                i += 1;
            }
            let (p0, p1) = (source_position[i - 1], source_position[i]);
            let f = (t - p0) / (p1 - p0);
            source_area_cm2[i - 1] * (1.0 - f) + source_area_cm2[i] * f
        };
        *o = v.max(min_area);
    }
    Ok(())
}

static SHARED: OnceLock<Result<Lumen, String>> = OnceLock::new();

/// The compiled-in mesh, parsed once, digest checked first.
pub fn shared() -> Result<&'static Lumen, String> {
    SHARED
        .get_or_init(|| {
            let digest = sha256_hex(BYTES);
            if digest != SHA256 {
                return Err(format!(
                    "tract_lumen_v2.bin digest {digest} is not the recorded {SHA256}"
                ));
            }
            Lumen::parse(BYTES)
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lumen() -> &'static Lumen {
        shared().unwrap()
    }

    /// The parse itself is the app's contract: header, CRC-32, payload
    /// length, ring areas within 0.2 % of the stored references.
    #[test]
    fn the_shipped_mesh_parses_as_32_by_24_with_the_frozen_mean_provenance() {
        let l = lumen();
        assert_eq!((l.sections, l.angular, l.vertex_count), (32, 24, 770));
        assert_eq!(l.triangle_indices.len(), 32 * 6 * 24);
        assert_eq!(l.provenance_kind, 2);
        assert_eq!(l.model_id, "vt3d-metric-mri-engineering-mean-v0.6.3");
        assert_eq!(l.section_for_vertex[768], 0);
        assert_eq!(l.section_for_vertex[769], 31);
        assert_eq!(l.section_for_vertex[24 * 5 + 3], 5);
        let len = l.centerline_length_mm();
        assert!(len > 100.0 && len < 250.0, "centerline {len} mm");
    }

    /// `morph` at the reference areas is centerline + offsets exactly, the
    /// apices are the centerline ends; a doubled area scales offsets by √2.
    #[test]
    fn morphing_at_the_reference_areas_reproduces_the_stored_mesh() {
        let l = lumen();
        let mut v = vec![0.0f32; l.vertex_count * 3];
        l.morph_into(&l.reference_area_cm2, &mut v).unwrap();
        for s in 0..l.sections {
            for i in 0..l.angular {
                let idx = (s * l.angular + i) * 3;
                for k in 0..3 {
                    assert_eq!(
                        v[idx + k],
                        l.centerline_mm[s * 3 + k] + l.ring_offsets_mm[idx + k]
                    );
                }
            }
        }
        assert_eq!(&v[768 * 3..768 * 3 + 3], &l.centerline_mm[..3]);
        assert_eq!(&v[769 * 3..], &l.centerline_mm[31 * 3..]);
        let doubled: Vec<f32> = l.reference_area_cm2.iter().map(|a| a * 2.0).collect();
        let mut w = vec![0.0f32; l.vertex_count * 3];
        l.morph_into(&doubled, &mut w).unwrap();
        let idx = (3 * l.angular + 7) * 3;
        let want = l.centerline_mm[9] + l.ring_offsets_mm[idx] * 2f32.sqrt();
        assert!((w[idx] - want).abs() < 1e-4);
        assert!(l.morph_into(&doubled[..10], &mut w).is_err());
        let mut n = vec![0.0f32; v.len()];
        vertex_normals_into(&v, &l.triangle_indices, &mut n).unwrap();
        for nn in n.chunks_exact(3) {
            let len = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-3, "normal length {len}");
        }
        let mut u = vec![0.0f32; v.len()];
        l.expand_uncertainty_into(&v, 0.5, 1.5, &mut u).unwrap();
        let s = 1.5f32.sqrt();
        assert!((u[idx] - (l.centerline_mm[9] + (v[idx] - l.centerline_mm[9]) * s)).abs() < 1e-4);
    }

    #[test]
    fn resampling_interpolates_clamps_and_floors() {
        let mut out = [0.0f32; 5];
        resample_area_into(
            &[0.0, 0.5, 1.0],
            &[1.0, 3.0, 2.0],
            &[-1.0, 0.25, 0.5, 0.75, 2.0],
            0.001,
            &mut out,
        )
        .unwrap();
        assert_eq!(out, [1.0, 2.0, 3.0, 2.5, 2.0]);
        let mut one = [0.0f32; 1];
        resample_area_into(&[0.0, 1.0], &[0.0005, 0.0005], &[0.5], 0.001, &mut one).unwrap();
        assert_eq!(one, [0.001]);
        assert!(resample_area_into(&[0.0, 0.0], &[1.0, 1.0], &[0.5], 0.001, &mut one).is_err());
    }

    #[test]
    fn a_flipped_byte_fails_the_crc() {
        let mut bad = BYTES.to_vec();
        bad[HEADER_BYTES + 40] ^= 0x01;
        assert!(Lumen::parse(&bad).unwrap_err().contains("CRC32"));
        assert!(Lumen::parse(&BYTES[..HEADER_BYTES - 1]).is_err());
    }
}
