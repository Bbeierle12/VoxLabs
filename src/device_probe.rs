//! Device capability probe: what does this phone's capture stack actually
//! give us?
//!
//! The live-vs-loudspeaker research (see the "Live or Loudspeaker" brief)
//! made every spatial method conditional on facts that cannot be looked up,
//! only measured on the device in hand:
//!
//!   1. Is the `UNPROCESSED` audio source *really* supported (CDD 5.11), or
//!      does it silently fall back to processed capture? Sub-bass balance,
//!      low-band power shape, modulation depth and energy-lability cues all
//!      die under default processing.
//!   2. What microphones does the device report — how many, where?
//!   3. Do two *independent* channels reach the app, or one internally
//!      beamformed mix copied into a stereo frame? Nulling one TV source
//!      while preserving one live source needs two independent channels
//!      (M ≥ D+1); visible mic holes prove nothing about what the HAL hands
//!      applications.
//!
//! The probe runs a short second capture (~1 s, 48 kHz, two-channel) beside
//! the live engine, measures inter-channel correlation and difference
//! energy, and reports — in the UI, in plain sight — what the device
//! actually is. Everything Java-side goes through JNI against stable public
//! API constants; the analysis itself is pure Rust and unit-tested on every
//! target.
//!
//! Honesty boundary: a near-perfect correlation cannot distinguish "the HAL
//! duplicated one mic" from "one distant point source and no mic self-noise"
//! — which is why the UI tells the user to run the probe with sound in the
//! room (TV playing, or speaking) and why the verdict has an inconclusive
//! state instead of rounding to the nearest convenient answer.

use std::sync::Mutex;

/// Samples per channel the two-channel test tries to collect (1 s @ 48 kHz).
pub const PROBE_FRAMES: usize = 48_000;

/// Probe capture sample rate. 48 kHz is what the Pixel's HAL runs natively
/// and what the engine captures at; probing anything else would answer a
/// question nobody asked.
pub const PROBE_SAMPLE_RATE: u32 = 48_000;

/// Correlation lag search half-window, in samples (±2 ms at 48 kHz — wider
/// than any physically possible inter-mic delay on a handset, so a real
/// acoustic pair's best lag is inside the window while a DSP-delayed copy
/// still gets found).
pub const MAX_LAG: usize = 96;

/// Above this best-lag correlation the two channels are copies of one
/// signal: no pair of physically separated microphones with independent
/// self-noise reaches it on real room audio.
pub const DUPLICATE_CORR: f32 = 0.9999;

/// Below this the channels demonstrably differ — two independent
/// observations.
pub const INDEPENDENT_CORR: f32 = 0.999;

/// Both channels quieter than this (dBFS RMS) means the probe heard
/// nothing: contention, permission, or a dead route — no verdict possible.
pub const LOW_SIGNAL_DBFS: f32 = -60.0;

/// What the two-channel comparison concluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelVerdict {
    /// Channels differ measurably: two independent observations reach the
    /// app. The spatial lane (RTF calibration, nulling) is open.
    Independent,
    /// Channels are copies of one signal — one effective microphone,
    /// whatever the channel count claims.
    Duplicated,
    /// Nothing on either channel; verdict impossible.
    LowSignal,
    /// Between the thresholds. Re-run with steadier / louder room sound.
    Inconclusive,
}

/// Numbers behind the verdict — shown raw in the UI so the thresholds are
/// checkable, not oracular.
#[derive(Clone, Debug)]
pub struct TwoChStats {
    pub rms_l_db: f32,
    pub rms_r_db: f32,
    /// Normalized cross-correlation at lag zero.
    pub corr0: f32,
    /// Best |correlation| over ±[`MAX_LAG`] samples…
    pub corr_best: f32,
    /// …and the lag (ms, positive = right lags left) where it occurred.
    pub best_lag_ms: f32,
    /// Difference-to-mean power ratio, dB: 10·log10(E[(L−R)²] / mean power).
    /// Copies → very negative; independent channels → tens of dB higher.
    pub diff_db: f32,
    pub verdict: ChannelVerdict,
}

/// One microphone from the device's static inventory
/// (`AudioManager.getMicrophones()`).
#[derive(Clone, Debug)]
pub struct MicInfo {
    pub description: String,
    pub location: String,
    /// Device-frame position in metres, when the OEM reports one.
    pub position: Option<[f32; 3]>,
}

/// Outcome of the two-channel capture attempt.
#[derive(Clone, Debug)]
pub enum CaptureOutcome {
    /// No recorder configuration could be opened; the string says what was
    /// tried and what each attempt returned.
    Failed(String),
    Ran(CaptureRun),
}

#[derive(Clone, Debug)]
pub struct CaptureRun {
    /// Which audio source fed the test recorder.
    pub source: &'static str,
    /// Which channel request succeeded ("index mask 0|1" or
    /// "position stereo").
    pub mask: &'static str,
    pub sample_rate: u32,
    /// Frames per channel actually collected.
    pub frames: usize,
    /// Active-microphone lines for the *test recorder's* session, with
    /// channel mapping — "mic_bottom → ch0 DIRECT" is the evidence line the
    /// whole probe exists for.
    pub active_mics: Vec<String>,
    pub stats: TwoChStats,
}

/// Everything the probe learned, for the Room tab and the log.
#[derive(Clone, Debug)]
pub struct ProbeReport {
    pub api_level: Option<i32>,
    /// `Some(true)` — the CDD-guaranteed raw path exists. `Some(false)` —
    /// it does not (capture silently degrades to processed). `None` — the
    /// query itself failed.
    pub unprocessed: Option<bool>,
    pub record_permission: Option<bool>,
    pub mics: Vec<MicInfo>,
    pub capture: CaptureOutcome,
    /// Anything worth keeping that fits no field above (failed attempts,
    /// API-level guards, contention warnings).
    pub notes: Vec<String>,
}

/// Probe lifecycle, snapshotted by the UI each frame.
#[derive(Clone, Debug)]
pub enum ProbeStatus {
    NotRun,
    Running,
    Done(ProbeReport),
}

static PROBE: Mutex<ProbeStatus> = Mutex::new(ProbeStatus::NotRun);

/// Current probe state (clone; cheap — reports are small).
pub fn status() -> ProbeStatus {
    match PROBE.lock() {
        Ok(s) => s.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

fn set_status(s: ProbeStatus) {
    match PROBE.lock() {
        Ok(mut slot) => *slot = s,
        Err(poisoned) => *poisoned.into_inner() = s,
    }
}

/// Kick off the probe on a background thread. No-op while one is running.
/// Android-only: the questions it answers are questions about the Android
/// capture stack, and the desktop UI never offers it.
#[cfg(target_os = "android")]
pub fn start() {
    {
        let slot = PROBE.lock().unwrap_or_else(|p| p.into_inner());
        if matches!(*slot, ProbeStatus::Running) {
            return;
        }
    }
    set_status(ProbeStatus::Running);
    std::thread::spawn(|| {
        let report = android::run_probe();
        for line in report_log_lines(&report) {
            log::info!("device probe: {line}");
        }
        set_status(ProbeStatus::Done(report));
    });
}

/// The whole report as loggable lines (logcat is the retrievable record on
/// a phone; the UI card is the visible one).
pub fn report_log_lines(r: &ProbeReport) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(api) = r.api_level {
        out.push(format!("api level {api}"));
    }
    out.push(match r.unprocessed {
        Some(true) => "UNPROCESSED: supported".into(),
        Some(false) => "UNPROCESSED: NOT supported (capture is processed)".into(),
        None => "UNPROCESSED: query failed".into(),
    });
    if let Some(p) = r.record_permission {
        out.push(format!("RECORD_AUDIO granted: {p}"));
    }
    out.push(format!("microphones reported: {}", r.mics.len()));
    for m in &r.mics {
        let pos = match m.position {
            Some([x, y, z]) => format!(
                " · pos ({:+.0}, {:+.0}, {:+.0}) mm",
                x * 1e3,
                y * 1e3,
                z * 1e3
            ),
            None => String::new(),
        };
        out.push(format!(
            "  mic \"{}\" — {}{}",
            m.description, m.location, pos
        ));
    }
    match &r.capture {
        CaptureOutcome::Failed(why) => out.push(format!("2-ch capture failed: {why}")),
        CaptureOutcome::Ran(run) => {
            out.push(format!(
                "2-ch capture: {} · {} · {} Hz · {} frames",
                run.source, run.mask, run.sample_rate, run.frames
            ));
            for a in &run.active_mics {
                out.push(format!("  active: {a}"));
            }
            let s = &run.stats;
            out.push(format!(
                "  corr {:.5} @ {:+.2} ms (lag0 {:.5}) · diff {:.0} dB · L {:.0} / R {:.0} dBFS · {:?}",
                s.corr_best, s.best_lag_ms, s.corr0, s.diff_db, s.rms_l_db, s.rms_r_db, s.verdict
            ));
        }
    }
    for n in &r.notes {
        out.push(format!("note: {n}"));
    }
    out
}

/// Compare the two channels of an interleaved capture. Pure math — this is
/// the part with a definite right answer, so it is the part with tests.
pub fn analyze_pair(left: &[f32], right: &[f32], sample_rate: f32) -> TwoChStats {
    let n = left.len().min(right.len());
    let (l, r) = (&left[..n], &right[..n]);

    let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len().max(1) as f32;
    let (ml, mr) = (mean(l), mean(r));

    let mut e_l = 0.0f64;
    let mut e_r = 0.0f64;
    let mut e_d = 0.0f64;
    for i in 0..n {
        let (a, b) = ((l[i] - ml) as f64, (r[i] - mr) as f64);
        e_l += a * a;
        e_r += b * b;
        let d = a - b;
        e_d += d * d;
    }
    let rms_db = |e: f64| 20.0 * ((e / n.max(1) as f64).sqrt().max(1e-9)).log10() as f32;
    let (rms_l_db, rms_r_db) = (rms_db(e_l), rms_db(e_r));

    let mean_pow = (0.5 * (e_l + e_r) / n.max(1) as f64).max(1e-18);
    let diff_pow = (e_d / n.max(1) as f64).max(0.0);
    let diff_db = (10.0 * (diff_pow.max(1e-8 * mean_pow) / mean_pow).log10() as f32).max(-80.0);

    // Normalized cross-correlation over ±MAX_LAG, normalized per overlap
    // window (NOT by the full-signal energies): a biased estimator loses
    // |lag|/n of its peak, which at the duplicate threshold of 0.9999 would
    // let a DSP-delayed copy of one mic slip under as "independent". With
    // per-overlap normalization an exact delayed copy scores exactly 1 at
    // its lag. O(n·lags) on 48 k samples is tens of milliseconds on a
    // background thread, and the probe runs once.
    let corr_at = |lag: isize| -> f32 {
        let mut acc = 0.0f64;
        let mut ea = 0.0f64;
        let mut eb = 0.0f64;
        for (i, &li) in l.iter().enumerate() {
            let j = i as isize + lag;
            if j < 0 || j >= n as isize {
                continue;
            }
            let a = (li - ml) as f64;
            let b = (r[j as usize] - mr) as f64;
            acc += a * b;
            ea += a * a;
            eb += b * b;
        }
        (acc / (ea * eb).sqrt().max(1e-18)) as f32
    };
    let corr0 = corr_at(0);
    // Cap the lag search so the smallest overlap window is still ≥ 3n/4 —
    // tiny overlaps make the normalized estimate noisy.
    let max_lag = MAX_LAG.min(n / 4) as isize;
    let mut corr_best = corr0;
    let mut best_lag: isize = 0;
    let mut lag = -max_lag;
    while lag <= max_lag {
        let c = corr_at(lag);
        if c.abs() > corr_best.abs() {
            corr_best = c;
            best_lag = lag;
        }
        lag += 1;
    }
    let best_lag_ms = best_lag as f32 * 1e3 / sample_rate.max(1.0);

    let verdict = if rms_l_db < LOW_SIGNAL_DBFS && rms_r_db < LOW_SIGNAL_DBFS {
        ChannelVerdict::LowSignal
    } else if corr_best.abs() >= DUPLICATE_CORR {
        ChannelVerdict::Duplicated
    } else if corr_best.abs() <= INDEPENDENT_CORR {
        ChannelVerdict::Independent
    } else {
        ChannelVerdict::Inconclusive
    };

    TwoChStats {
        rms_l_db,
        rms_r_db,
        corr0,
        corr_best,
        best_lag_ms,
        diff_db,
        verdict,
    }
}

// ─── Android side: JNI against the public audio API ─────────────────────────
//
// Everything below talks to Java through the `jni` crate (already in the
// tree under cpal). Constants are hardcoded from the frozen public API —
// MediaRecorder.AudioSource.UNPROCESSED = 9, VOICE_RECOGNITION = 6;
// AudioFormat.ENCODING_PCM_16BIT = 2, CHANNEL_IN_STEREO = 0xC;
// AudioRecord.STATE_INITIALIZED = 1, RECORDSTATE_RECORDING = 3;
// MicrophoneInfo.CHANNEL_MAPPING_DIRECT = 1, _PROCESSED = 2 — these values
// are API-stable by contract and reading them reflectively would only add
// failure modes.

#[cfg(target_os = "android")]
mod android {
    use super::*;
    use jni::objects::{JObject, JString, JValue};
    use jni::{Env, jni_sig, jni_str};

    const SOURCE_VOICE_RECOGNITION: i32 = 6;
    const SOURCE_UNPROCESSED: i32 = 9;
    const ENCODING_PCM_16BIT: i32 = 2;
    const CHANNEL_IN_STEREO: i32 = 0xC;
    const STATE_INITIALIZED: i32 = 1;
    const RECORDSTATE_RECORDING: i32 = 3;

    /// Error carrier that satisfies `E: From<jni::errors::Error>` for the
    /// attach callback while staying printable.
    struct JErr(String);
    impl From<jni::errors::Error> for JErr {
        fn from(e: jni::errors::Error) -> Self {
            JErr(format!("{e}"))
        }
    }

    pub(super) fn run_probe() -> ProbeReport {
        let ctx = ndk_context::android_context();
        // android-activity initialized this context; a null VM would mean the
        // process has no Java side at all, which cannot happen in an APK.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
        let activity_ptr = ctx.context();
        let result: Result<ProbeReport, JErr> = vm.attach_current_thread(|env| {
            let activity = unsafe { JObject::from_raw(env, activity_ptr as jni::sys::jobject) };
            Ok(probe_in_env(env, &activity))
        });
        match result {
            Ok(report) => report,
            Err(JErr(e)) => ProbeReport {
                api_level: None,
                unprocessed: None,
                record_permission: None,
                mics: Vec::new(),
                capture: CaptureOutcome::Failed(format!("JNI attach failed: {e}")),
                notes: Vec::new(),
            },
        }
    }

    /// If the last JNI call left a pending Java exception, clear it (so
    /// later calls can proceed) and fold its presence into the error text.
    fn clear_exception(env: &mut Env) {
        if env.exception_check() {
            env.exception_clear();
        }
    }

    fn get_string(env: &mut Env, obj: JObject) -> Option<String> {
        if obj.is_null() {
            return None;
        }
        // The object is known to be a java.lang.String (every call site got
        // it from an API returning String), so the unchecked cast is sound.
        let js = unsafe { JString::from_raw(env, obj.into_raw()) };
        match js.try_to_string(env) {
            Ok(s) => Some(s),
            Err(_) => {
                clear_exception(env);
                None
            }
        }
    }

    fn probe_in_env(env: &mut Env, activity: &JObject) -> ProbeReport {
        let mut notes = Vec::new();

        let api_level = env
            .get_static_field(
                jni_str!("android/os/Build$VERSION"),
                jni_str!("SDK_INT"),
                jni_sig!("I"),
            )
            .and_then(|v| v.i())
            .map_err(|_| clear_exception(env))
            .ok();

        // RECORD_AUDIO grant state — the probe's failures read differently
        // when the permission is the reason.
        let record_permission = (|| -> Result<bool, JErr> {
            let name = env.new_string("android.permission.RECORD_AUDIO")?;
            let r = env
                .call_method(
                    activity,
                    jni_str!("checkSelfPermission"),
                    jni_sig!("(Ljava/lang/String;)I"),
                    &[JValue::Object(&name)],
                )?
                .i()?;
            Ok(r == 0)
        })()
        .map_err(|_| clear_exception(env))
        .ok();

        // AudioManager.
        let audio_manager = (|| -> Result<JObject, JErr> {
            let name = env.new_string("audio")?;
            let am = env
                .call_method(
                    activity,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&name)],
                )?
                .l()?;
            Ok(am)
        })();
        let audio_manager = match audio_manager {
            Ok(am) if !am.is_null() => am,
            _ => {
                clear_exception(env);
                return ProbeReport {
                    api_level,
                    unprocessed: None,
                    record_permission,
                    mics: Vec::new(),
                    capture: CaptureOutcome::Failed("AudioManager unavailable".into()),
                    notes,
                };
            }
        };

        // PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED — "true" or absent.
        let unprocessed = (|| -> Result<Option<bool>, JErr> {
            let key = env.new_string("android.media.property.SUPPORT_AUDIO_SOURCE_UNPROCESSED")?;
            let v = env
                .call_method(
                    &audio_manager,
                    jni_str!("getProperty"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/String;"),
                    &[JValue::Object(&key)],
                )?
                .l()?;
            let s = get_string(env, v);
            Ok(Some(s.as_deref() == Some("true")))
        })()
        .unwrap_or_else(|_| {
            clear_exception(env);
            None
        });

        // Static microphone inventory (API 28+).
        let mut mics = Vec::new();
        if api_level.unwrap_or(0) >= 28 {
            if let Err(JErr(e)) = read_microphones(env, &audio_manager, &mut mics) {
                clear_exception(env);
                notes.push(format!("getMicrophones failed: {e}"));
            }
        } else {
            notes.push("API < 28: MicrophoneInfo unavailable".into());
        }

        // Two-channel capture attempts, strongest-claim first. Order:
        // UNPROCESSED (only if the property said it exists — CDD says an
        // unsupported request silently degrades, which would poison the
        // result) then VOICE_RECOGNITION; index mask before position mask.
        let mut attempts: Vec<(&'static str, i32, &'static str, bool)> = Vec::new();
        if unprocessed == Some(true) {
            attempts.push(("UNPROCESSED", SOURCE_UNPROCESSED, "index mask 0|1", true));
            attempts.push(("UNPROCESSED", SOURCE_UNPROCESSED, "position stereo", false));
        }
        attempts.push((
            "VOICE_RECOGNITION",
            SOURCE_VOICE_RECOGNITION,
            "index mask 0|1",
            true,
        ));
        attempts.push((
            "VOICE_RECOGNITION",
            SOURCE_VOICE_RECOGNITION,
            "position stereo",
            false,
        ));

        let mut capture = None;
        for (source_name, source, mask_name, index_mask) in attempts {
            match try_capture(env, source, index_mask, api_level.unwrap_or(0)) {
                Ok((interleaved, active_mics)) => {
                    let frames = interleaved.len() / 2;
                    let mut left = Vec::with_capacity(frames);
                    let mut right = Vec::with_capacity(frames);
                    for pair in interleaved.chunks_exact(2) {
                        left.push(pair[0] as f32 / 32768.0);
                        right.push(pair[1] as f32 / 32768.0);
                    }
                    let stats = analyze_pair(&left, &right, PROBE_SAMPLE_RATE as f32);
                    capture = Some(CaptureOutcome::Ran(CaptureRun {
                        source: source_name,
                        mask: mask_name,
                        sample_rate: PROBE_SAMPLE_RATE,
                        frames,
                        active_mics,
                        stats,
                    }));
                    break;
                }
                Err(JErr(e)) => {
                    clear_exception(env);
                    notes.push(format!("{source_name} + {mask_name}: {e}"));
                }
            }
        }
        let capture = capture.unwrap_or_else(|| {
            CaptureOutcome::Failed(if record_permission == Some(false) {
                "every recorder configuration failed — RECORD_AUDIO is not granted".into()
            } else {
                "every recorder configuration failed (details in notes)".into()
            })
        });

        ProbeReport {
            api_level,
            unprocessed,
            record_permission,
            mics,
            capture,
            notes,
        }
    }

    fn location_name(loc: i32) -> &'static str {
        match loc {
            1 => "main body",
            2 => "main body (movable)",
            3 => "peripheral",
            _ => "unknown location",
        }
    }

    fn read_microphones(
        env: &mut Env,
        audio_manager: &JObject,
        out: &mut Vec<MicInfo>,
    ) -> Result<(), JErr> {
        let list = env
            .call_method(
                audio_manager,
                jni_str!("getMicrophones"),
                jni_sig!("()Ljava/util/List;"),
                &[],
            )?
            .l()?;
        if list.is_null() {
            return Ok(());
        }
        let size = env
            .call_method(&list, jni_str!("size"), jni_sig!("()I"), &[])?
            .i()?;
        for k in 0..size.min(16) {
            let mi = env
                .call_method(
                    &list,
                    jni_str!("get"),
                    jni_sig!("(I)Ljava/lang/Object;"),
                    &[JValue::Int(k)],
                )?
                .l()?;
            let desc_obj = env
                .call_method(
                    &mi,
                    jni_str!("getDescription"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let description =
                get_string(env, desc_obj).unwrap_or_else(|| format!("microphone {k}"));
            let location = env
                .call_method(&mi, jni_str!("getLocation"), jni_sig!("()I"), &[])?
                .i()?;
            let pos_obj = env
                .call_method(
                    &mi,
                    jni_str!("getPosition"),
                    jni_sig!("()Landroid/media/MicrophoneInfo$Coordinate3F;"),
                    &[],
                )?
                .l()?;
            let position = if pos_obj.is_null() {
                None
            } else {
                let x = env.get_field(&pos_obj, jni_str!("x"), jni_sig!("F"))?.f()?;
                let y = env.get_field(&pos_obj, jni_str!("y"), jni_sig!("F"))?.f()?;
                let z = env.get_field(&pos_obj, jni_str!("z"), jni_sig!("F"))?.f()?;
                // POSITION_UNKNOWN is a Coordinate3F of -Float.MAX_VALUE.
                (x > -1.0e30).then_some([x, y, z])
            };
            out.push(MicInfo {
                description,
                location: location_name(location).to_string(),
                position,
            });
        }
        Ok(())
    }

    /// Build a two-channel AudioRecord, capture ~[`PROBE_FRAMES`] frames,
    /// return interleaved i16 samples plus active-mic mapping lines.
    fn try_capture(
        env: &mut Env,
        source: i32,
        index_mask: bool,
        api_level: i32,
    ) -> Result<(Vec<i16>, Vec<String>), JErr> {
        // AudioFormat.
        let fmt_builder = env.new_object(
            jni_str!("android/media/AudioFormat$Builder"),
            jni_sig!("()V"),
            &[],
        )?;
        env.call_method(
            &fmt_builder,
            jni_str!("setSampleRate"),
            jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
            &[JValue::Int(PROBE_SAMPLE_RATE as i32)],
        )?;
        env.call_method(
            &fmt_builder,
            jni_str!("setEncoding"),
            jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
            &[JValue::Int(ENCODING_PCM_16BIT)],
        )?;
        if index_mask {
            env.call_method(
                &fmt_builder,
                jni_str!("setChannelIndexMask"),
                jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
                &[JValue::Int(0b11)],
            )?;
        } else {
            env.call_method(
                &fmt_builder,
                jni_str!("setChannelMask"),
                jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
                &[JValue::Int(CHANNEL_IN_STEREO)],
            )?;
        }
        let format = env
            .call_method(
                &fmt_builder,
                jni_str!("build"),
                jni_sig!("()Landroid/media/AudioFormat;"),
                &[],
            )?
            .l()?;

        // AudioRecord — `build()` throws when the configuration is
        // unsupported, which surfaces here as Err and is exactly the
        // signal we're probing for.
        let rec_builder = env.new_object(
            jni_str!("android/media/AudioRecord$Builder"),
            jni_sig!("()V"),
            &[],
        )?;
        env.call_method(
            &rec_builder,
            jni_str!("setAudioSource"),
            jni_sig!("(I)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Int(source)],
        )?;
        env.call_method(
            &rec_builder,
            jni_str!("setAudioFormat"),
            jni_sig!("(Landroid/media/AudioFormat;)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Object(&format)],
        )?;
        // 2 s of 16-bit stereo — comfortable headroom over the 1 s target.
        env.call_method(
            &rec_builder,
            jni_str!("setBufferSizeInBytes"),
            jni_sig!("(I)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Int((PROBE_SAMPLE_RATE * 2 * 2 * 2) as i32)],
        )?;
        let rec = env
            .call_method(
                &rec_builder,
                jni_str!("build"),
                jni_sig!("()Landroid/media/AudioRecord;"),
                &[],
            )?
            .l()?;

        let state = env
            .call_method(&rec, jni_str!("getState"), jni_sig!("()I"), &[])?
            .i()?;
        if state != STATE_INITIALIZED {
            let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
            return Err(JErr(format!("recorder state {state} (not initialized)")));
        }

        env.call_method(&rec, jni_str!("startRecording"), jni_sig!("()V"), &[])?;
        let rec_state = env
            .call_method(&rec, jni_str!("getRecordingState"), jni_sig!("()I"), &[])?
            .i()?;
        if rec_state != RECORDSTATE_RECORDING {
            let _ = env.call_method(&rec, jni_str!("stop"), jni_sig!("()V"), &[]);
            let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
            return Err(JErr(format!("recording state {rec_state} (not recording)")));
        }

        // Active microphones for THIS session — where DIRECT/PROCESSED
        // channel mapping lives.
        let mut active_mics = Vec::new();
        if api_level >= 28
            && let Err(JErr(e)) = read_active_mics(env, &rec, &mut active_mics)
        {
            clear_exception(env);
            active_mics.push(format!("getActiveMicrophones failed: {e}"));
        }

        // Read loop: blocking read() of interleaved shorts.
        const CHUNK: usize = 9600; // 0.1 s of stereo
        let want = PROBE_FRAMES * 2;
        let chunk_arr = env.new_short_array(CHUNK)?;
        let mut samples: Vec<i16> = Vec::with_capacity(want);
        let mut buf = vec![0i16; CHUNK];
        let mut zero_reads = 0;
        while samples.len() < want {
            let n = env
                .call_method(
                    &rec,
                    jni_str!("read"),
                    jni_sig!("([SII)I"),
                    &[
                        JValue::Object(&chunk_arr),
                        JValue::Int(0),
                        JValue::Int(CHUNK as i32),
                    ],
                )?
                .i()?;
            if n <= 0 {
                zero_reads += 1;
                if zero_reads > 5 {
                    break;
                }
                continue;
            }
            chunk_arr.get_region(env, 0, &mut buf[..n as usize])?;
            samples.extend_from_slice(&buf[..n as usize]);
        }

        let _ = env.call_method(&rec, jni_str!("stop"), jni_sig!("()V"), &[]);
        let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);

        if samples.len() < want / 4 {
            return Err(JErr(format!(
                "only {} of {} samples arrived (read errors)",
                samples.len(),
                want
            )));
        }
        Ok((samples, active_mics))
    }

    fn read_active_mics(env: &mut Env, rec: &JObject, out: &mut Vec<String>) -> Result<(), JErr> {
        let list = env
            .call_method(
                rec,
                jni_str!("getActiveMicrophones"),
                jni_sig!("()Ljava/util/List;"),
                &[],
            )?
            .l()?;
        if list.is_null() {
            return Ok(());
        }
        let size = env
            .call_method(&list, jni_str!("size"), jni_sig!("()I"), &[])?
            .i()?;
        for k in 0..size.min(8) {
            let mi = env
                .call_method(
                    &list,
                    jni_str!("get"),
                    jni_sig!("(I)Ljava/lang/Object;"),
                    &[JValue::Int(k)],
                )?
                .l()?;
            let desc_obj = env
                .call_method(
                    &mi,
                    jni_str!("getDescription"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let desc = get_string(env, desc_obj).unwrap_or_else(|| format!("mic {k}"));
            let mut line = desc;
            let mapping = env
                .call_method(
                    &mi,
                    jni_str!("getChannelMapping"),
                    jni_sig!("()Ljava/util/List;"),
                    &[],
                )?
                .l()?;
            if !mapping.is_null() {
                let msize = env
                    .call_method(&mapping, jni_str!("size"), jni_sig!("()I"), &[])?
                    .i()?;
                for j in 0..msize.min(8) {
                    let pair = env
                        .call_method(
                            &mapping,
                            jni_str!("get"),
                            jni_sig!("(I)Ljava/lang/Object;"),
                            &[JValue::Int(j)],
                        )?
                        .l()?;
                    let first = env
                        .get_field(&pair, jni_str!("first"), jni_sig!("Ljava/lang/Object;"))?
                        .l()?;
                    let second = env
                        .get_field(&pair, jni_str!("second"), jni_sig!("Ljava/lang/Object;"))?
                        .l()?;
                    let ch = env
                        .call_method(&first, jni_str!("intValue"), jni_sig!("()I"), &[])?
                        .i()?;
                    let kind = env
                        .call_method(&second, jni_str!("intValue"), jni_sig!("()I"), &[])?
                        .i()?;
                    let kind = match kind {
                        1 => "DIRECT",
                        2 => "PROCESSED",
                        _ => "?",
                    };
                    line.push_str(&format!(" → ch{ch} {kind}"));
                }
            }
            out.push(line);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic LCG so tests need no rand dependency.
    struct Lcg(u64);
    impl Lcg {
        fn next_f32(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            // Top 24 bits → [-1, 1).
            ((self.0 >> 40) as f32 / (1u32 << 23) as f32) - 1.0
        }
    }

    fn noise(seed: u64, n: usize, amp: f32) -> Vec<f32> {
        let mut g = Lcg(seed);
        (0..n).map(|_| amp * g.next_f32()).collect()
    }

    #[test]
    fn duplicated_channels_are_flagged() {
        let l = noise(1, 8000, 0.1);
        let s = analyze_pair(&l, &l, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::Duplicated);
        assert!(s.corr_best > 0.9999);
        assert!(s.diff_db <= -60.0, "diff {}", s.diff_db);
    }

    #[test]
    fn scaled_copy_is_still_duplicated() {
        // One mic duplicated with a gain difference is still one mic.
        let l = noise(2, 8000, 0.1);
        let r: Vec<f32> = l.iter().map(|v| v * 0.5).collect();
        let s = analyze_pair(&l, &r, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::Duplicated);
    }

    #[test]
    fn delayed_copy_is_found_within_lag_window() {
        // DSP delay compensation ≤ 2 ms must not disguise a copy. This test
        // is why the correlation is normalized per overlap window: a
        // full-energy (biased) estimator tops out at 1 − |lag|/n ≈ 0.995
        // here, under the duplicate threshold — the copy would have passed
        // as independent.
        let base = noise(3, 9000, 0.1);
        let l = base[..8000].to_vec();
        let r = base[40..8040].to_vec(); // r is l advanced by 40 samples
        let s = analyze_pair(&l, &r, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::Duplicated);
        // l[i] matches r[i + lag] at lag = −40 → −0.833 ms.
        let expected_ms = -40.0 * 1e3 / 48_000.0;
        assert!(
            (s.best_lag_ms - expected_ms).abs() < 0.05,
            "best lag {} ms, expected {expected_ms} ms",
            s.best_lag_ms
        );
    }

    #[test]
    fn independent_noise_is_independent() {
        let l = noise(4, 8000, 0.1);
        let r = noise(5, 8000, 0.1);
        let s = analyze_pair(&l, &r, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::Independent);
        assert!(s.corr_best.abs() < 0.1);
    }

    #[test]
    fn shared_source_with_self_noise_is_independent() {
        // Two real mics hearing one source still differ by their own noise
        // floors: shared signal + per-channel noise 20 dB down.
        let shared = noise(6, 8000, 0.1);
        let nl = noise(7, 8000, 0.01);
        let nr = noise(8, 8000, 0.01);
        let l: Vec<f32> = shared.iter().zip(&nl).map(|(s, n)| s + n).collect();
        let r: Vec<f32> = shared.iter().zip(&nr).map(|(s, n)| s + n).collect();
        let s = analyze_pair(&l, &r, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::Independent);
        assert!(
            s.corr_best > 0.9,
            "still strongly correlated: {}",
            s.corr_best
        );
    }

    #[test]
    fn silence_gives_no_verdict() {
        let l = noise(9, 8000, 1e-5);
        let r = noise(10, 8000, 1e-5);
        let s = analyze_pair(&l, &r, 48_000.0);
        assert_eq!(s.verdict, ChannelVerdict::LowSignal);
    }

    #[test]
    fn report_log_lines_cover_the_report() {
        let r = ProbeReport {
            api_level: Some(35),
            unprocessed: Some(true),
            record_permission: Some(true),
            mics: vec![MicInfo {
                description: "mic_bottom".into(),
                location: "main body".into(),
                position: Some([0.01, -0.06, 0.002]),
            }],
            capture: CaptureOutcome::Failed("test".into()),
            notes: vec!["note".into()],
        };
        let lines = report_log_lines(&r);
        assert!(lines.iter().any(|l| l.contains("UNPROCESSED: supported")));
        assert!(lines.iter().any(|l| l.contains("mic_bottom")));
        assert!(lines.iter().any(|l| l.contains("capture failed")));
    }
}
