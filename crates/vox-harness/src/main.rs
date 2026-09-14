//! `voxlab` — the Fach Lab study harness.
//!
//! Decodes audio files and runs them through the SAME per-frame pipeline
//! the phone runs (`vox_core::frame::FrameAnalyzer`), plus the
//! voice-part measurements in `fach`, and writes three tables:
//!
//!   frames.csv — one row per 2048-sample analysis frame
//!   notes.csv  — one row per sustained note (≥ 1 s within ±1 semitone)
//!   files.csv  — one row per file: tessitura, FHE, cluster, LTAS
//!                third-octave levels, turnover / register events, pooled
//!                VTL, coverage, QC, joined with metadata.csv
//!
//! Usage:
//!   voxlab analyze <dataset_dir> [--out DIR] [--sr 48000] [--vtl-f0-max 300]
//!   voxlab file <audio>
//!   voxlab synth <out_dir>      write synthetic fixtures with known answers
//!
//! Everything a number here says about a file, the phone would say about
//! the same audio — that is the point of the harness.

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
mod experiment3;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
mod lab {
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use vox_core::audio_file::{self, resample};
    use vox_core::fach::{self, Ltas, RegisterDetector, TurnoverTracker};
    use vox_core::frame::{ANALYSIS_FRAME, FrameAnalyzer};
    use vox_core::math;
    use vox_core::tract;
    use vox_core::types::VocalProfile;

    pub const DEFAULT_SR: f32 = 48_000.0;
    /// Sustained-note segmentation: minimum length and pitch tolerance.
    pub const NOTE_MIN_SECS: f32 = 1.0;
    pub const NOTE_TOL_SEMITONES: f32 = 1.0;
    /// Identity-grade VTL follows the app's f0 gate
    /// (`math::FORMANT_F0_IDENTITY_MAX_HZ`, 200 Hz); the relaxed pool is a
    /// study option, since higher voices never sing below 200 Hz. This is
    /// the default ceiling for that pool (`--vtl-f0-max` overrides it).
    pub const RELAXED_F0_MAX: f32 = 300.0;

    // ─── Per-file analysis ─────────────────────────────────────────────

    #[derive(Clone, Debug)]
    pub struct FrameRow {
        pub idx: usize,
        pub t_s: f32,
        pub profile: VocalProfile,
        pub rms_db: f32,
        pub grade: &'static str,
        pub identity_ok: bool,
        pub vtl_cm: Option<f32>,
        pub a2_a1_db: Option<f32>,
        pub tilt: Option<f32>,
        pub dominant_h: Option<u8>,
        pub fhe_m: Option<f32>,
        pub fhe_f: Option<f32>,
        pub band_centroid_m: Option<f32>,
    }

    #[derive(Clone, Debug)]
    pub struct NoteRow {
        pub start_s: f32,
        pub end_s: f32,
        pub f0_hz: f32,
        pub frames: usize,
        pub f1: Option<f32>,
        pub f2: Option<f32>,
        pub f3: Option<f32>,
        pub vtl_cm: Option<f32>,
        pub fhe_m: Option<f32>,
        pub fhe_f: Option<f32>,
        pub hnr: Option<f32>,
        pub cpp: Option<f32>,
        pub h1h2: Option<f32>,
        pub jitter: Option<f32>,
        pub shimmer: Option<f32>,
        pub tilt: Option<f32>,
        pub dominant_h: Option<u8>,
        pub vibrato_rate: Option<f32>,
        pub vibrato_extent: Option<f32>,
    }

    #[derive(Clone, Debug)]
    pub struct FileReport {
        pub duration_s: f32,
        pub sr_native: u32,
        pub peak_dbfs: f32,
        pub clipped_samples: usize,
        pub floor_dbfs: Option<f32>,
        pub frames_total: usize,
        pub frames_voiced: usize,
        pub frames_identity: usize,
        pub tessitura: Option<fach::Tessitura>,
        pub fhe_m: Option<f32>,
        pub fhe_f: Option<f32>,
        pub cluster: Option<fach::ClusterStats>,
        pub third_octave: Option<[f32; 20]>,
        pub vtl_identity: Option<f32>,
        pub vtl_relaxed: Option<f32>,
        pub f3_identity: Option<f32>,
        pub turnovers: Vec<fach::TurnoverEvent>,
        pub register_events: Vec<fach::RegisterEvent>,
        pub vibrato_rate: Option<f32>,
        pub vibrato_extent: Option<f32>,
        pub h1_share: f32,
        pub h2_share: f32,
        pub h3_share: f32,
        pub frames: Vec<FrameRow>,
        pub notes: Vec<NoteRow>,
    }

    fn grade_name(g: math::FormantGrade) -> &'static str {
        match g {
            math::FormantGrade::Identity => "identity",
            math::FormantGrade::DisplayOnly => "display",
            math::FormantGrade::Reject => "reject",
        }
    }

    /// Run the full pipeline on mono samples already at `sr`.
    pub fn analyze_samples(
        samples: &[f32],
        sr: f32,
        sr_native: u32,
        vtl_f0_max: f32,
    ) -> FileReport {
        let mut analyzer = FrameAnalyzer::new(sr);
        let n_bins = ANALYSIS_FRAME / 2 + 1;
        let bin_hz = sr / ANALYSIS_FRAME as f32;
        let mut ltas = Ltas::new(n_bins, bin_hz);
        let mut turnover = TurnoverTracker::new();
        let mut register = RegisterDetector::new();
        let mut frames = Vec::new();
        let mut f0s = Vec::new();
        let mut fhe_m_all = Vec::new();
        let mut fhe_f_all = Vec::new();
        let mut vtl_id = Vec::new();
        let mut vtl_relaxed = Vec::new();
        let mut f3_id = Vec::new();
        let mut vib_rate = Vec::new();
        let mut vib_ext = Vec::new();
        let mut rms_all = Vec::new();
        let mut register_events = Vec::new();
        let mut dom = [0usize; 3];

        let peak = samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
        let clipped = samples.iter().filter(|&&s| s.abs() >= 0.999).count();

        for (idx, frame) in samples.chunks_exact(ANALYSIS_FRAME).enumerate() {
            let r = analyzer.analyze(frame);
            let p = r.profile;
            let rms_db = 20.0 * r.rms.max(1e-9).log10();
            rms_all.push(rms_db);
            let voiced = p.valid;

            let grade = math::formant_grade(&p.formants, p.formants_f0);
            let snr_ok = p
                .metrics
                .snr_db
                .is_none_or(|s| s >= math::IDENTITY_MIN_SNR_DB);
            let identity_ok = voiced && grade == math::FormantGrade::Identity && snr_ok;
            let vtl = if voiced && grade != math::FormantGrade::Reject {
                tract::vtl_from_formants(p.formants[1].frequency, p.formants[2].frequency)
            } else {
                None
            };
            if identity_ok {
                if let Some(l) = vtl {
                    vtl_id.push(l);
                }
                if p.formants[2].frequency > 0.0 {
                    f3_id.push(p.formants[2].frequency);
                }
            }
            if voiced
                && grade != math::FormantGrade::Reject
                && p.formants_f0 <= vtl_f0_max
                && snr_ok
                && let Some(l) = vtl
            {
                vtl_relaxed.push(l);
            }

            let (a2a1, tilt, dominant_h) = if voiced {
                (
                    math::a2_a1_db(&p.partial_amplitudes),
                    math::spectral_tilt_db_per_octave(&p.partial_amplitudes),
                    fach::dominant_harmonic(&p.partial_amplitudes),
                )
            } else {
                (None, None, None)
            };
            if let Some(d) = dominant_h {
                dom[(d - 1) as usize] += 1;
            }

            let (fhe_m, fhe_f, band_c) = if voiced {
                match fach::power_spectrum(frame, sr) {
                    Some(ps) => {
                        ltas.push(&ps);
                        let m = fach::band_half_energy(
                            &ps,
                            fach::FHE_BAND_MALE.0,
                            fach::FHE_BAND_MALE.1,
                        );
                        let f = fach::band_half_energy(
                            &ps,
                            fach::FHE_BAND_FEMALE.0,
                            fach::FHE_BAND_FEMALE.1,
                        );
                        (
                            m.map(|b| b.fhe_hz),
                            f.map(|b| b.fhe_hz),
                            m.map(|b| b.centroid_hz),
                        )
                    }
                    None => (None, None, None),
                }
            } else {
                (None, None, None)
            };
            if let Some(v) = fhe_m {
                fhe_m_all.push(v);
            }
            if let Some(v) = fhe_f {
                fhe_f_all.push(v);
            }
            if voiced {
                f0s.push(p.f0);
                if let Some(v) = p.metrics.vibrato {
                    vib_rate.push(v.rate_hz);
                    vib_ext.push(v.extent_cents);
                }
            }

            turnover.push(voiced.then_some(p.f0), a2a1);
            if let Some(ev) = register.push(
                voiced.then_some(p.f0),
                p.metrics.h1_h2_db,
                p.metrics.cpp_db,
                p.metrics.jitter_pct,
            ) {
                register_events.push(ev);
            }

            frames.push(FrameRow {
                idx,
                t_s: idx as f32 * ANALYSIS_FRAME as f32 / sr,
                profile: p,
                rms_db,
                grade: grade_name(grade),
                identity_ok,
                vtl_cm: vtl,
                a2_a1_db: a2a1,
                tilt,
                dominant_h,
                fhe_m,
                fhe_f,
                band_centroid_m: band_c,
            });
        }

        let notes = segment_notes(&frames, sr);
        let mean_power = ltas.mean_power();
        // Smooth over ±(median f0) so a sustained vowel's harmonic lines
        // merge into the envelope before the cluster is measured.
        let tess = fach::tessitura(&f0s);
        let smooth_hz = tess.map_or(150.0, |t| t.p50.clamp(80.0, 400.0));
        let cluster = mean_power
            .as_deref()
            .and_then(|mp| fach::cluster_stats(&fach::smooth_power(mp, bin_hz, smooth_hz), bin_hz));
        let mut floor_sorted = rms_all.clone();
        floor_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let dom_total = (dom[0] + dom[1] + dom[2]).max(1) as f32;

        FileReport {
            duration_s: samples.len() as f32 / sr,
            sr_native,
            peak_dbfs: 20.0 * peak.max(1e-9).log10(),
            clipped_samples: clipped,
            floor_dbfs: fach::percentile(&floor_sorted, 5.0),
            frames_total: frames.len(),
            frames_voiced: frames.iter().filter(|f| f.profile.valid).count(),
            frames_identity: frames.iter().filter(|f| f.identity_ok).count(),
            tessitura: tess,
            fhe_m: fach::median(&fhe_m_all),
            fhe_f: fach::median(&fhe_f_all),
            cluster,
            third_octave: ltas.third_octave_db(),
            vtl_identity: fach::median(&vtl_id),
            vtl_relaxed: fach::median(&vtl_relaxed),
            f3_identity: fach::median(&f3_id),
            turnovers: turnover.events().to_vec(),
            register_events,
            vibrato_rate: fach::median(&vib_rate),
            vibrato_extent: fach::median(&vib_ext),
            h1_share: dom[0] as f32 / dom_total,
            h2_share: dom[1] as f32 / dom_total,
            h3_share: dom[2] as f32 / dom_total,
            frames,
            notes,
        }
    }

    /// Sustained notes: runs of voiced frames staying within
    /// ±NOTE_TOL_SEMITONES of the run's running median for ≥ NOTE_MIN_SECS.
    fn segment_notes(frames: &[FrameRow], sr: f32) -> Vec<NoteRow> {
        let frame_s = ANALYSIS_FRAME as f32 / sr;
        let min_frames = (NOTE_MIN_SECS / frame_s).ceil() as usize;
        let mut notes = Vec::new();
        let mut run: Vec<&FrameRow> = Vec::new();
        let flush = |run: &mut Vec<&FrameRow>, notes: &mut Vec<NoteRow>| {
            if run.len() >= min_frames {
                notes.push(summarize_note(run, sr));
            }
            run.clear();
        };
        for f in frames {
            if !f.profile.valid {
                flush(&mut run, &mut notes);
                continue;
            }
            if let Some(first) = run.first() {
                let st_run = fach::hz_to_semitone(first.profile.f0);
                let st_now = fach::hz_to_semitone(f.profile.f0);
                // Compare against the run median so vibrato doesn't split notes.
                let med = fach::median(&run.iter().map(|r| r.profile.f0).collect::<Vec<_>>())
                    .map(fach::hz_to_semitone)
                    .unwrap_or(st_run);
                if (st_now - med).abs() > NOTE_TOL_SEMITONES {
                    flush(&mut run, &mut notes);
                }
            }
            run.push(f);
        }
        flush(&mut run, &mut notes);
        notes
    }

    fn summarize_note(run: &[&FrameRow], sr: f32) -> NoteRow {
        let frame_s = ANALYSIS_FRAME as f32 / sr;
        let med = |f: &dyn Fn(&FrameRow) -> Option<f32>| {
            fach::median(&run.iter().filter_map(|r| f(r)).collect::<Vec<_>>())
        };
        let id = |i: usize| -> Option<f32> {
            fach::median(
                &run.iter()
                    .filter(|r| r.identity_ok && r.profile.formants[i].frequency > 0.0)
                    .map(|r| r.profile.formants[i].frequency)
                    .collect::<Vec<_>>(),
            )
        };
        let mut dom = [0usize; 3];
        for r in run {
            if let Some(d) = r.dominant_h {
                dom[(d - 1) as usize] += 1;
            }
        }
        let dominant_h = dom
            .iter()
            .enumerate()
            .max_by_key(|&(_, &c)| c)
            .filter(|&(_, &c)| c > 0)
            .map(|(i, _)| i as u8 + 1);
        NoteRow {
            start_s: run[0].t_s,
            end_s: run[run.len() - 1].t_s + frame_s,
            f0_hz: fach::median(&run.iter().map(|r| r.profile.f0).collect::<Vec<_>>())
                .unwrap_or(0.0),
            frames: run.len(),
            f1: id(0),
            f2: id(1),
            f3: id(2),
            vtl_cm: med(&|r| if r.identity_ok { r.vtl_cm } else { None }),
            fhe_m: med(&|r| r.fhe_m),
            fhe_f: med(&|r| r.fhe_f),
            hnr: med(&|r| r.profile.metrics.hnr_db),
            cpp: med(&|r| r.profile.metrics.cpp_db),
            h1h2: med(&|r| r.profile.metrics.h1_h2_db),
            jitter: med(&|r| r.profile.metrics.jitter_pct),
            shimmer: med(&|r| r.profile.metrics.shimmer_db),
            tilt: med(&|r| r.tilt),
            dominant_h,
            vibrato_rate: med(&|r| r.profile.metrics.vibrato.map(|v| v.rate_hz)),
            vibrato_extent: med(&|r| r.profile.metrics.vibrato.map(|v| v.extent_cents)),
        }
    }

    // ─── Metadata ───────────────────────────────────────────────────────

    /// Minimal CSV reader (quoted fields, no embedded newlines).
    pub fn read_csv(path: &Path) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let text = fs::read_to_string(path)?;
        let mut rows = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let mut fields = Vec::new();
            let mut cur = String::new();
            let mut in_q = false;
            let mut chars = line.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '"' if in_q && chars.peek() == Some(&'"') => {
                        cur.push('"');
                        chars.next();
                    }
                    '"' => in_q = !in_q,
                    ',' if !in_q => {
                        fields.push(cur.trim().to_string());
                        cur.clear();
                    }
                    _ => cur.push(c),
                }
            }
            fields.push(cur.trim().to_string());
            rows.push(fields);
        }
        let header = rows.first().cloned().unwrap_or_default();
        Ok((header, rows.into_iter().skip(1).collect()))
    }

    pub fn csv_quote(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }

    fn opt(v: Option<f32>) -> String {
        v.map(|x| format!("{x:.3}")).unwrap_or_default()
    }

    // ─── Dataset walk + writers ─────────────────────────────────────────

    pub fn audio_files(root: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(rd) = fs::read_dir(dir) else { return };
            let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(&p, out);
                } else if audio_file::is_audio_file(&p) {
                    out.push(p);
                }
            }
        }
        walk(root, &mut out);
        out
    }

    pub fn analyze_dataset(
        root: &Path,
        out_dir: &Path,
        sr: f32,
        vtl_f0_max: f32,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(out_dir)?;
        // Metadata, keyed by normalized relative path.
        let (meta_header, meta_rows) = read_csv(&root.join("metadata.csv")).unwrap_or_default();
        let file_col = meta_header.iter().position(|h| h == "file");
        let meta: HashMap<String, Vec<String>> = meta_rows
            .into_iter()
            .filter_map(|r| {
                file_col
                    .and_then(|c| r.get(c).cloned())
                    .map(|k| (k.replace('\\', "/"), r))
            })
            .collect();
        let extra_cols: Vec<&String> = meta_header.iter().filter(|h| *h != "file").collect();

        let mut frames_w = File::create(out_dir.join("frames.csv"))?;
        let mut notes_w = File::create(out_dir.join("notes.csv"))?;
        let mut files_w = File::create(out_dir.join("files.csv"))?;
        writeln!(
            frames_w,
            "file,idx,t_s,voiced,f0_hz,note,rms_db,snr_db,noisy,hnr_db,h1h2_db,cpp_db,jitter_pct,shimmer_db,centroid_hz,f1,f2,f3,formants_f0,grade,identity_ok,vtl_cm,a2a1_db,tilt_db_oct,dominant_h,fhe_m,fhe_f,band_centroid_m,{}",
            (1..=16)
                .map(|k| format!("h{k}"))
                .collect::<Vec<_>>()
                .join(",")
        )?;
        writeln!(
            notes_w,
            "file,start_s,end_s,f0_hz,note,frames,f1,f2,f3,vtl_cm,fhe_m,fhe_f,hnr_db,cpp_db,h1h2_db,jitter_pct,shimmer_db,tilt_db_oct,dominant_h,vibrato_rate_hz,vibrato_extent_cents"
        )?;
        let to_cols: Vec<String> = fach::THIRD_OCTAVE_CENTERS
            .iter()
            .map(|c| format!("ltas_{}", *c as u32))
            .collect();
        writeln!(
            files_w,
            "file,duration_s,sr_native,peak_dbfs,clipped,floor_dbfs,frames,voiced_frac,identity_frac,tess_p10,tess_p25,tess_p50,tess_p75,tess_p90,tess_lo,tess_hi,tess_p50_note,fhe_m,fhe_f,cluster_peak_hz,cluster_prom_db,cluster_width_hz,vtl_identity_cm,vtl_relaxed_cm,f3_identity,vibrato_rate_hz,vibrato_extent_cents,h1_share,h2_share,h3_share,turnovers,register_events,{},{}",
            to_cols.join(","),
            extra_cols
                .iter()
                .map(|c| csv_quote(c))
                .collect::<Vec<_>>()
                .join(",")
        )?;

        let files = audio_files(root);
        eprintln!("{} audio files under {}", files.len(), root.display());
        for (n, path) in files.iter().enumerate() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            eprint!("[{}/{}] {rel} … ", n + 1, files.len());
            let dec = match audio_file::decode(path) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("decode failed: {e}");
                    continue;
                }
            };
            let sr_native = dec.sample_rate;
            let samples = resample(&dec.samples, sr_native as f32, sr);
            let rep = analyze_samples(&samples, sr, sr_native, vtl_f0_max);
            eprintln!(
                "{:.1} s, voiced {:.0}%, identity {:.0}%, FHE(m) {}",
                rep.duration_s,
                100.0 * rep.frames_voiced as f32 / rep.frames_total.max(1) as f32,
                100.0 * rep.frames_identity as f32 / rep.frames_voiced.max(1) as f32,
                opt(rep.fhe_m)
            );
            write_report(
                &rel,
                &rep,
                &mut frames_w,
                &mut notes_w,
                &mut files_w,
                &meta,
                &meta_header,
                &extra_cols,
            )?;
        }
        eprintln!("wrote {}", out_dir.display());
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn write_report(
        rel: &str,
        rep: &FileReport,
        frames_w: &mut File,
        notes_w: &mut File,
        files_w: &mut File,
        meta: &HashMap<String, Vec<String>>,
        meta_header: &[String],
        extra_cols: &[&String],
    ) -> anyhow::Result<()> {
        let q = csv_quote(rel);
        for f in &rep.frames {
            let p = &f.profile;
            let m = &p.metrics;
            let amps: Vec<String> = p
                .partial_amplitudes
                .iter()
                .take(16)
                .map(|a| format!("{a:.5}"))
                .collect();
            writeln!(
                frames_w,
                "{q},{},{:.3},{},{:.2},{},{:.1},{},{},{},{},{},{},{},{},{:.0},{:.0},{:.0},{:.1},{},{},{},{},{},{},{},{},{},{}",
                f.idx,
                f.t_s,
                u8::from(p.valid),
                p.f0,
                if p.valid {
                    fach::note_name(p.f0)
                } else {
                    String::new()
                },
                f.rms_db,
                opt(m.snr_db),
                u8::from(m.voiced_but_noisy),
                opt(m.hnr_db),
                opt(m.h1_h2_db),
                opt(m.cpp_db),
                opt(m.jitter_pct),
                opt(m.shimmer_db),
                opt(m.centroid_hz),
                p.formants[0].frequency,
                p.formants[1].frequency,
                p.formants[2].frequency,
                p.formants_f0,
                f.grade,
                u8::from(f.identity_ok),
                opt(f.vtl_cm),
                opt(f.a2_a1_db),
                opt(f.tilt),
                f.dominant_h.map(|d| d.to_string()).unwrap_or_default(),
                opt(f.fhe_m),
                opt(f.fhe_f),
                opt(f.band_centroid_m),
                amps.join(",")
            )?;
        }
        for n in &rep.notes {
            writeln!(
                notes_w,
                "{q},{:.3},{:.3},{:.2},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                n.start_s,
                n.end_s,
                n.f0_hz,
                fach::note_name(n.f0_hz),
                n.frames,
                opt(n.f1),
                opt(n.f2),
                opt(n.f3),
                opt(n.vtl_cm),
                opt(n.fhe_m),
                opt(n.fhe_f),
                opt(n.hnr),
                opt(n.cpp),
                opt(n.h1h2),
                opt(n.jitter),
                opt(n.shimmer),
                opt(n.tilt),
                n.dominant_h.map(|d| d.to_string()).unwrap_or_default(),
                opt(n.vibrato_rate),
                opt(n.vibrato_extent)
            )?;
        }
        let t = rep.tessitura;
        let tess = |f: &dyn Fn(&fach::Tessitura) -> f32| {
            t.map(|t| format!("{:.2}", f(&t))).unwrap_or_default()
        };
        let turn = rep
            .turnovers
            .iter()
            .map(|e| {
                format!(
                    "{:.1}{}{}",
                    e.f0_hz,
                    if e.rising { "↑" } else { "↓" },
                    if e.to_h1 { "H1" } else { "H2" }
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let regs = rep
            .register_events
            .iter()
            .map(|e| format!("{:.0}>{:.0}", e.f0_from, e.f0_to))
            .collect::<Vec<_>>()
            .join(" ");
        let ltas = rep
            .third_octave
            .map(|b| {
                b.iter()
                    .map(|v| format!("{v:.2}"))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| vec![String::new(); 20].join(","));
        let extra: Vec<String> = extra_cols
            .iter()
            .map(|col| {
                let idx = meta_header.iter().position(|h| h == *col);
                meta.get(rel)
                    .and_then(|row| idx.and_then(|i| row.get(i)))
                    .map(|v| csv_quote(v))
                    .unwrap_or_default()
            })
            .collect();
        writeln!(
            files_w,
            "{q},{:.2},{},{:.1},{},{},{},{:.3},{:.3},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{:.3},{},{},{},{}",
            rep.duration_s,
            rep.sr_native,
            rep.peak_dbfs,
            rep.clipped_samples,
            opt(rep.floor_dbfs),
            rep.frames_total,
            rep.frames_voiced as f32 / rep.frames_total.max(1) as f32,
            rep.frames_identity as f32 / rep.frames_voiced.max(1) as f32,
            tess(&|t| t.p10),
            tess(&|t| t.p25),
            tess(&|t| t.p50),
            tess(&|t| t.p75),
            tess(&|t| t.p90),
            tess(&|t| t.lo),
            tess(&|t| t.hi),
            t.map(|t| fach::note_name(t.p50)).unwrap_or_default(),
            opt(rep.fhe_m),
            opt(rep.fhe_f),
            opt(rep.cluster.map(|c| c.peak_hz)),
            opt(rep.cluster.map(|c| c.prominence_db)),
            opt(rep.cluster.map(|c| c.width_hz)),
            opt(rep.vtl_identity),
            opt(rep.vtl_relaxed),
            opt(rep.f3_identity),
            opt(rep.vibrato_rate),
            opt(rep.vibrato_extent),
            rep.h1_share,
            rep.h2_share,
            rep.h3_share,
            csv_quote(&turn),
            csv_quote(&regs),
            ltas,
            extra.join(",")
        )?;
        Ok(())
    }

    // ─── Synthetic fixtures with known answers ──────────────────────────

    /// Lorentzian resonance gain (linear frequency), peak 1 at `fc`.
    fn resonance(f: f32, fc: f32, bw: f32) -> f32 {
        let x = (f - fc) / (bw / 2.0);
        1.0 / (1.0 + x * x)
    }

    /// A 4 s vowel at `f0` with formants 700/1200/2600 Hz and a singer's-
    /// formant cluster centred at `cluster_hz`.
    pub fn synth_vowel(f0: f32, cluster_hz: f32, sr: f32, secs: f32) -> Vec<f32> {
        let n = (sr * secs) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / sr;
                let mut s = 0.0;
                let mut k = 1;
                while f0 * (k as f32) < sr * 0.45 {
                    let f = f0 * k as f32;
                    let env = 0.6 * resonance(f, 700.0, 120.0)
                        + 0.35 * resonance(f, 1200.0, 160.0)
                        + 0.25 * resonance(f, cluster_hz, 350.0)
                        + 0.02;
                    let tilt = (f / f0).powf(-0.6);
                    s += env * tilt * (2.0 * std::f32::consts::PI * f * t).sin();
                    k += 1;
                }
                s * 0.15
            })
            .collect()
    }

    /// A glide f0 `from`→`to` over `secs` with a single fixed resonance at
    /// `f1`: A2/A1 crosses 0 dB where H1 and H2 are equidistant from F1,
    /// i.e. at f0 = 2·F1/3 for a symmetric linear-frequency resonance.
    pub fn synth_glide(from: f32, to: f32, f1: f32, sr: f32, secs: f32) -> Vec<f32> {
        let n = (sr * secs) as usize;
        let mut phase = [0.0f32; 8];
        (0..n)
            .map(|i| {
                let u = i as f32 / n as f32;
                let f0 = from * (to / from).powf(u);
                let mut s = 0.0;
                for (k, ph) in phase.iter_mut().enumerate() {
                    let f = f0 * (k + 1) as f32;
                    *ph += 2.0 * std::f32::consts::PI * f / sr;
                    let env = resonance(f, f1, 300.0) + 0.01;
                    s += env * ph.sin();
                }
                s * 0.15
            })
            .collect()
    }

    pub fn write_wav(path: &Path, samples: &[f32], sr: u32) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sr,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec)?;
        for &s in samples {
            w.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16)?;
        }
        w.finalize()?;
        Ok(())
    }

    pub fn synth_fixtures(out: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(out.join("bass").join("synth_b"))?;
        fs::create_dir_all(out.join("tenor").join("synth_t"))?;
        let sr = 48_000u32;
        write_wav(
            &out.join("bass/synth_b/vowel_a_110.wav"),
            &synth_vowel(110.0, 2400.0, sr as f32, 4.0),
            sr,
        )?;
        write_wav(
            &out.join("tenor/synth_t/vowel_a_180.wav"),
            &synth_vowel(180.0, 2800.0, sr as f32, 4.0),
            sr,
        )?;
        write_wav(
            &out.join("tenor/synth_t/glide_a.wav"),
            &synth_glide(200.0, 600.0, 600.0, sr as f32, 8.0),
            sr,
        )?;
        let mut m = File::create(out.join("metadata.csv"))?;
        writeln!(
            m,
            "file,singer_id,part,weight,sex,material,vowel,nominal_pitch,key,source,notes"
        )?;
        writeln!(
            m,
            "bass/synth_b/vowel_a_110.wav,synth_b,bass,,m,vowel,a,A2,,synth,cluster 2400"
        )?;
        writeln!(
            m,
            "tenor/synth_t/vowel_a_180.wav,synth_t,tenor,,m,vowel,a,F#3,,synth,cluster 2800"
        )?;
        writeln!(
            m,
            "tenor/synth_t/glide_a.wav,synth_t,tenor,,m,glide,a,,,synth,F1 600 → turnover 400"
        )?;
        Ok(())
    }

    // ─── CLI ────────────────────────────────────────────────────────────

    // ─── `voxlab run` / `voxlab validate`: any mode file, headless ─────

    /// `voxlab run`: every audio file under `root` (or one file) through
    /// `mode`, results beside each other in `out`.
    pub fn run_mode(mode_name: &str, root: &Path, out: &Path, sr: f32) -> anyhow::Result<()> {
        let mode = vox_core::pipeline::PipelineDefinition::by_name_or_path(mode_name)
            .map_err(|e| anyhow::anyhow!("{mode_name}: {e}"))?;
        let files: Vec<PathBuf> = if root.is_file() {
            vec![root.to_path_buf()]
        } else {
            let mut v: Vec<PathBuf> = fs::read_dir(root)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| audio_file::is_audio_file(p))
                .collect();
            v.sort();
            v
        };
        if files.is_empty() {
            anyhow::bail!("no audio files under {}", root.display());
        }
        for f in &files {
            let (hops, _) = vox_validation::record_run(&mode, f, out, sr)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{} · {} · {hops} hops", f.display(), mode.name);
        }
        println!("{} file(s) → {}", files.len(), out.display());
        Ok(())
    }

    /// `voxlab validate`: the provenance round-trip for every
    /// `*.provenance.json` under `dir` (or the one file named).
    pub fn validate(target: &Path) -> anyhow::Result<()> {
        let records: Vec<PathBuf> = if target.is_file() {
            vec![target.to_path_buf()]
        } else {
            let mut v: Vec<PathBuf> = fs::read_dir(target)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.ends_with(".provenance.json"))
                })
                .collect();
            v.sort();
            v
        };
        if records.is_empty() {
            anyhow::bail!("no provenance records under {}", target.display());
        }
        let mut failed = 0;
        for r in &records {
            let rep = vox_validation::validate_and_write(r).map_err(|e| anyhow::anyhow!("{e}"))?;
            let verdict = if rep.pass { "PASS" } else { "FAIL" };
            println!(
                "{verdict}  {} · {} hops · input digest {} · params digest {}",
                r.display(),
                rep.compare.hops_compared,
                if rep.input_digest_matched {
                    "ok"
                } else {
                    "MISMATCH"
                },
                if rep.params_digest_matched {
                    "ok"
                } else {
                    "MISMATCH"
                }
            );
            for (t, d) in &rep.compare.per_type {
                println!(
                    "      {t}: {} compared, {} outside band, worst {:.3}× ({})",
                    d.compared, d.outside_band, d.worst_ratio, d.worst
                );
            }
            failed += usize::from(!rep.pass);
        }
        if failed > 0 {
            anyhow::bail!("{failed} of {} record(s) failed", records.len());
        }
        Ok(())
    }

    fn flag(args: &[String], name: &str) -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    }

    pub fn main() -> anyhow::Result<()> {
        let args: Vec<String> = std::env::args().collect();
        let usage = "usage: voxlab analyze <dataset_dir> [--out DIR] [--sr HZ] [--vtl-f0-max HZ]\n       voxlab file <audio>\n       voxlab synth <out_dir>\n       voxlab run <mode|mode.toml> <audio|dir> [--out DIR] [--sr HZ]\n       voxlab validate <results_dir|x.provenance.json>\n       voxlab gen-inverse-fixtures <out_dir> [--n PAIRS] [--seed S]";
        match args.get(1).map(String::as_str) {
            Some("run") => {
                let mode = args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?;
                let root = PathBuf::from(args.get(3).ok_or_else(|| anyhow::anyhow!(usage))?);
                let out = flag(&args, "--out").map(PathBuf::from).unwrap_or_else(|| {
                    if root.is_dir() {
                        root.join("results")
                    } else {
                        root.parent().unwrap_or(Path::new(".")).join("results")
                    }
                });
                let sr: f32 = flag(&args, "--sr")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_SR);
                run_mode(mode, &root, &out, sr)
            }
            Some("validate") => {
                let target = PathBuf::from(args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?);
                validate(&target)
            }
            Some("gen-inverse-fixtures") => {
                let out = PathBuf::from(args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?);
                let v = vox_core::config::ValidationConfig::DEFAULT;
                let n: usize = flag(&args, "--n")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(v.inverse_fixture_pairs);
                let seed: u32 = flag(&args, "--seed")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(v.inverse_fixture_seed);
                let report = super::experiment3::run(&out, n, seed)?;
                super::experiment3::print(&report);
                println!("→ {}", out.join("inverse_eval.json").display());
                Ok(())
            }
            Some("analyze") => {
                let root = PathBuf::from(args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?);
                let out = flag(&args, "--out")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.join("results"));
                let sr: f32 = flag(&args, "--sr")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_SR);
                let vtl_max: f32 = flag(&args, "--vtl-f0-max")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(RELAXED_F0_MAX);
                analyze_dataset(&root, &out, sr, vtl_max)
            }
            Some("file") => {
                let path = PathBuf::from(args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?);
                let dec = audio_file::decode(&path)?;
                let sr_native = dec.sample_rate;
                let samples = resample(&dec.samples, sr_native as f32, DEFAULT_SR);
                let rep = analyze_samples(&samples, DEFAULT_SR, sr_native, RELAXED_F0_MAX);
                println!("{}", path.display());
                println!(
                    "  {:.1} s @ {} Hz native · peak {:.1} dBFS · clipped {} · floor {}",
                    rep.duration_s,
                    rep.sr_native,
                    rep.peak_dbfs,
                    rep.clipped_samples,
                    opt(rep.floor_dbfs)
                );
                println!(
                    "  frames {} · voiced {:.0}% · identity-grade {:.0}% of voiced",
                    rep.frames_total,
                    100.0 * rep.frames_voiced as f32 / rep.frames_total.max(1) as f32,
                    100.0 * rep.frames_identity as f32 / rep.frames_voiced.max(1) as f32
                );
                if let Some(t) = rep.tessitura {
                    println!(
                        "  tessitura P10/50/90 {:.0}/{:.0}/{:.0} Hz ({}/{}/{}) · range {:.0}–{:.0} Hz",
                        t.p10,
                        t.p50,
                        t.p90,
                        fach::note_name(t.p10),
                        fach::note_name(t.p50),
                        fach::note_name(t.p90),
                        t.lo,
                        t.hi
                    );
                }
                println!(
                    "  FHE male-band {} · female-band {} Hz",
                    opt(rep.fhe_m),
                    opt(rep.fhe_f)
                );
                if let Some(c) = rep.cluster {
                    println!(
                        "  cluster peak {:.0} Hz · prominence {:.1} dB · width {:.0} Hz",
                        c.peak_hz, c.prominence_db, c.width_hz
                    );
                }
                println!(
                    "  VTL identity {} · relaxed {} cm · F3 {} Hz",
                    opt(rep.vtl_identity),
                    opt(rep.vtl_relaxed),
                    opt(rep.f3_identity)
                );
                println!(
                    "  dominant harmonic shares H1 {:.2} H2 {:.2} H3 {:.2} · vibrato {} Hz ±{} cents",
                    rep.h1_share,
                    rep.h2_share,
                    rep.h3_share,
                    opt(rep.vibrato_rate),
                    opt(rep.vibrato_extent)
                );
                for e in &rep.turnovers {
                    println!(
                        "  turnover at {:.0} Hz ({}) {} pitch, {} takes over",
                        e.f0_hz,
                        fach::note_name(e.f0_hz),
                        if e.rising { "rising" } else { "falling" },
                        if e.to_h1 { "H1" } else { "H2" }
                    );
                }
                for e in &rep.register_events {
                    println!(
                        "  register event {:.0} → {:.0} Hz ({:+.1} st)",
                        e.f0_from, e.f0_to, e.jump_semitones
                    );
                }
                println!("  notes: {}", rep.notes.len());
                for n in &rep.notes {
                    println!(
                        "    {:.1}–{:.1} s  {} ({:.1} Hz)  F1 {} F2 {} F3 {}  VTL {}  FHE {}",
                        n.start_s,
                        n.end_s,
                        fach::note_name(n.f0_hz),
                        n.f0_hz,
                        opt(n.f1),
                        opt(n.f2),
                        opt(n.f3),
                        opt(n.vtl_cm),
                        opt(n.fhe_m)
                    );
                }
                Ok(())
            }
            Some("synth") => {
                let out = PathBuf::from(args.get(2).ok_or_else(|| anyhow::anyhow!(usage))?);
                synth_fixtures(&out)?;
                eprintln!("wrote synthetic fixtures under {}", out.display());
                Ok(())
            }
            _ => Err(anyhow::anyhow!(usage)),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn synthetic_vowel_measures_its_cluster_and_pitch() {
            let sr = DEFAULT_SR;
            let s = synth_vowel(180.0, 2800.0, sr, 3.0);
            let rep = analyze_samples(&s, sr, sr as u32, RELAXED_F0_MAX);
            assert!(
                rep.frames_voiced > rep.frames_total / 2,
                "voiced {}/{}",
                rep.frames_voiced,
                rep.frames_total
            );
            let t = rep.tessitura.expect("tessitura");
            assert!((t.p50 - 180.0).abs() < 3.0, "p50 {}", t.p50);
            let fhe = rep.fhe_m.expect("fhe");
            assert!((fhe - 2800.0).abs() < 120.0, "fhe {fhe}");
            let c = rep.cluster.expect("cluster");
            assert!((c.peak_hz - 2800.0).abs() < 100.0, "peak {}", c.peak_hz);
            assert!(!rep.notes.is_empty(), "a 3 s vowel is one note");
        }

        #[test]
        fn cluster_position_moves_with_the_synthetic_voice_type() {
            let sr = DEFAULT_SR;
            let bass = analyze_samples(
                &synth_vowel(110.0, 2400.0, sr, 3.0),
                sr,
                sr as u32,
                RELAXED_F0_MAX,
            );
            let tenor = analyze_samples(
                &synth_vowel(180.0, 2800.0, sr, 3.0),
                sr,
                sr as u32,
                RELAXED_F0_MAX,
            );
            let (b, t) = (bass.fhe_m.unwrap(), tenor.fhe_m.unwrap());
            assert!(t - b > 250.0, "bass {b} vs tenor {t}");
        }

        #[test]
        fn synthetic_glide_turns_over_at_two_thirds_f1() {
            let sr = DEFAULT_SR;
            let s = synth_glide(200.0, 600.0, 600.0, sr, 8.0);
            let rep = analyze_samples(&s, sr, sr as u32, RELAXED_F0_MAX);
            let rising: Vec<_> = rep.turnovers.iter().filter(|e| e.rising).collect();
            assert!(
                !rising.is_empty(),
                "no rising turnover: {:?}",
                rep.turnovers
            );
            let f = rising[0].f0_hz;
            assert!(
                (f - 400.0).abs() < 25.0,
                "turnover at {f} Hz, expected ~400"
            );
            assert!(
                rising[0].to_h1,
                "ascending through 2F1/3: H1 should take over"
            );
        }

        #[test]
        fn resampled_tone_analyzes_at_its_pitch() {
            let sr_in = 44_100.0;
            let n = 44_100;
            let x: Vec<f32> = (0..n)
                .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr_in).sin())
                .collect();
            let y = resample(&x, sr_in, 48_000.0);
            let rep = analyze_samples(&y, 48_000.0, 44_100, RELAXED_F0_MAX);
            let t = rep.tessitura.expect("tessitura");
            assert!((t.p50 - 440.0).abs() < 3.0, "p50 {}", t.p50);
        }

        #[test]
        fn wav_roundtrip_through_the_decoder() {
            let dir = std::env::temp_dir().join(format!("voxlab-test-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let p = dir.join("tone.wav");
            let s = synth_vowel(150.0, 2600.0, 48_000.0, 1.0);
            write_wav(&p, &s, 48_000).unwrap();
            let dec = audio_file::decode(&p).unwrap();
            assert_eq!(dec.sample_rate, 48_000);
            assert!((dec.samples.len() as isize - s.len() as isize).abs() < 4);
            let err: f32 = dec
                .samples
                .iter()
                .zip(&s)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max);
            assert!(err < 1e-3, "max abs error {err}");
            let _ = fs::remove_dir_all(&dir);
        }

        #[test]
        fn csv_reader_handles_quotes() {
            let dir = std::env::temp_dir().join(format!("voxlab-csv-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let p = dir.join("m.csv");
            fs::write(
                &p,
                "file,part,notes\na/b.wav,bass,\"hello, \"\"world\"\"\"\n",
            )
            .unwrap();
            let (h, rows) = read_csv(&p).unwrap();
            assert_eq!(h, vec!["file", "part", "notes"]);
            assert_eq!(rows[0][2], "hello, \"world\"");
            let _ = fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() -> anyhow::Result<()> {
    lab::main()
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {}
