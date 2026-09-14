//! The lumen asset's contract: the app's own parse checks, morph, normals, resampling.

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
