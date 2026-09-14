//! The Phase 3 contract checks (the Vocal Tract Lab port), run as host
//! tests and from the phone's self-test through `contract::run_all`.

use super::contract::{SR, check, fmt, vowel};
use super::definition::PipelineDefinition;
use super::stage::Stage;
use super::types::{F0Track, FormantTrack, TractParams, Wire};
use crate::config::PipelineParams;
use crate::types::Formant;

/// Phase 3: both compiled-in data files load under their recorded digests
/// and say what the JSON says about themselves.
pub fn atlas_data_files_verify() -> Result<String, String> {
    let m = crate::atlas::reduced_model::shared()?;
    let l = crate::atlas::lumen::shared()?;
    check(
        "model id",
        m.model_id.as_str(),
        "vt3d-frozen-mri-pca-v0.7.0",
    )?;
    check("modes × sections", (m.n_modes(), m.n_sections()), (4, 32))?;
    check(
        "mesh",
        (l.sections, l.angular, l.vertex_count),
        (32, 24, 770),
    )?;
    check(
        "renderer digest in JSON",
        m.atlas.renderer_lumen_sha256.as_str(),
        crate::atlas::lumen::SHA256,
    )?;
    check(
        "scientific_release_ready",
        m.atlas.scientific_release_ready,
        false,
    )?;
    Ok(format!(
        "{} · {} subjects · release-ready {} · acceptances {}",
        m.model_id,
        m.atlas.subjects,
        m.atlas.scientific_release_ready,
        m.atlas.independent_expert_acceptances
    ))
}

/// Phase 3: the posterior stage on a strong frame equals the decompiled
/// formulas (clamped linear map, follow rate, confidence, uncertainty).
pub fn posterior_matches_reference() -> Result<String, String> {
    use super::stages::posterior::PosteriorInverseStage;
    use crate::config::PosteriorConfig;
    use crate::types::VoiceMetrics;
    let mut s =
        PosteriorInverseStage::init(&PipelineParams::DEFAULT, &fmt()).map_err(|e| e.to_string())?;
    let f0 = F0Track {
        hz: 120.0,
        confidence: 0.95,
        voiced: true,
        snr_db: Some(30.0),
        rejected: false,
    };
    let vm = VoiceMetrics {
        hnr_db: Some(25.0),
        ..Default::default()
    };
    let ft = FormantTrack {
        formants: [816.0f32, 2022.0, 3174.0].map(|frequency| Formant {
            frequency,
            bandwidth: 80.0,
        }),
        measured_f0: 120.0,
        confidence: 1.0,
        fresh: true,
        f4: None,
    };
    let mut out = TractParams::default();
    s.process((&f0, &ft, &vm), &mut out)
        .map_err(|e| e.to_string())?;
    let m = crate::atlas::reduced_model::shared()?;
    let cfg = PosteriorConfig::DEFAULT;
    let e = PosteriorInverseStage::evidence(&cfg, &f0, &vm, true);
    let alpha = (cfg.filter_alpha_gain * e + cfg.filter_alpha_offset)
        .clamp(cfg.filter_alpha_min, cfg.filter_alpha_max);
    for i in 0..4 {
        let raw: f32 = (0..3)
            .map(|j| {
                m.formant_to_mode[i][j] * (ft.formants[j].frequency - m.reference_formants_hz[j])
            })
            .sum::<f32>()
            .clamp(-2.0, 2.0);
        let want = (raw * alpha).clamp(-2.0, 2.0);
        if (out.modes[i] - want).abs() > 1e-5 {
            return Err(format!("mode {i}: got {}, expected {want}", out.modes[i]));
        }
    }
    let conf = e * cfg.confidence_follow;
    check("abstained", out.abstained, false)?;
    if (out.confidence.unwrap_or(-1.0) - conf).abs() > 1e-5 {
        return Err(format!("confidence {:?} vs {conf}", out.confidence));
    }
    let unc = (1.0 - conf) * cfg.area_std_gain + cfg.area_std_offset;
    if (out.uncertainty.unwrap_or(-1.0) - unc).abs() > 1e-5 {
        return Err(format!("uncertainty {:?} vs {unc}", out.uncertainty));
    }
    Ok(format!(
        "evidence {e:.3} · α {alpha:.3} · modes {:+.3} {:+.3} {:+.3} {:+.3} · conf {conf:.3}",
        out.modes[0], out.modes[1], out.modes[2], out.modes[3]
    ))
}

/// Phase 3: the lumen morphed at its reference areas is the stored mesh.
pub fn lumen_mesh_morphs() -> Result<String, String> {
    let l = crate::atlas::lumen::shared()?;
    let mut v = vec![0.0f32; l.vertex_count * 3];
    l.morph_into(&l.reference_area_cm2, &mut v)?;
    let mut worst = 0.0f32;
    for s in 0..l.sections {
        for i in 0..l.angular {
            let idx = (s * l.angular + i) * 3;
            for k in 0..3 {
                let want = l.centerline_mm[s * 3 + k] + l.ring_offsets_mm[idx + k];
                worst = worst.max((v[idx + k] - want).abs());
            }
        }
    }
    if worst > 0.0 {
        return Err(format!("morph at reference differs by {worst} mm"));
    }
    let mut n = vec![0.0f32; v.len()];
    crate::atlas::lumen::vertex_normals_into(&v, &l.triangle_indices, &mut n)?;
    let bad = n
        .chunks_exact(3)
        .filter(|c| ((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt() - 1.0).abs() > 1e-3)
        .count();
    check("non-unit normals", bad, 0)?;
    Ok(format!(
        "{} vertices · {} triangles · centerline {:.1} mm",
        l.vertex_count,
        l.triangle_indices.len() / 3,
        l.centerline_length_mm()
    ))
}

/// Phase 3: the atlas mode builds and, on a synthetic vowel, stops
/// abstaining, shapes a live area function and meshes it finitely.
pub fn atlas_mode_runs_a_vowel() -> Result<String, String> {
    use super::offline::run_offline;
    crate::room::reset();
    let def = PipelineDefinition::by_name_or_path("atlas").map_err(|e| e.to_string())?;
    let signal = vowel(120.0, 3.86, 1.35, SR as usize);
    let idx = |name: &str| {
        def.stages
            .iter()
            .position(|s| s.name == name)
            .map(|i| i + 1)
    };
    let (i_inv, i_tract, i_mesh) = (
        idx("inverse").unwrap(),
        idx("tract").unwrap(),
        idx("mesh").unwrap(),
    );
    let mut evidence_hops = 0u64;
    let mut live_meshes = 0u64;
    let mut last = None;
    let mut finite = true;
    let (_, hops) = run_offline(&def, SR, &signal, |h| {
        if let (
            Some(Wire::TractParams(p)),
            Some(Wire::AreaFunction(a)),
            Some(Wire::TractGeometry(g)),
        ) = (
            h.wires.get(i_inv),
            h.wires.get(i_tract),
            h.wires.get(i_mesh),
        ) {
            if !p.abstained {
                evidence_hops += 1;
            }
            if g.live && a.live {
                live_meshes += 1;
            }
            finite &= g.vertices_mm.iter().all(|v| v.is_finite())
                && a.areas_cm2[..32].iter().all(|v| v.is_finite() && *v > 0.0);
            last = Some(*p);
        }
    })
    .map_err(|e| e.to_string())?;
    check("finite geometry", finite, true)?;
    if evidence_hops == 0 {
        return Err(format!(
            "the posterior abstained on all {hops} hops: {last:?}"
        ));
    }
    check("live meshes = evidence hops", live_meshes, evidence_hops)?;
    Ok(format!(
        "{hops} hops · {evidence_hops} with evidence · last {:?}",
        last.map(|p| (p.modes, p.confidence, p.reason))
    ))
}
