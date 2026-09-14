# Vocal Acoustics Research Brief
## Physics of Vocal Acoustics for Choral Source Separation / Multi-F0 / Harmonic Fingerprinting

**Prepared for:** DSP engineer building SATB-section identification and individual-singer fingerprinting from single ambient microphone.
**Scope:** Physics-grounded facts, cited to primary literature. Uncertainty flagged explicitly.

---

## 1. Source–Filter Model of Voice Production

### The Model

- The classical source–filter model (Fant, 1960, *Acoustic Theory of Speech Production*, Mouton) treats voice production as two largely independent stages: a glottal source and a vocal-tract filter (resonator). The source generates a complex periodic waveform; the filter shapes its spectrum.
- The linear (decoupled) version has dominated speech technology for ~60 years. It is a useful first approximation but breaks down in certain singing conditions (see Nonlinear Coupling below).

### Glottal Source

- The glottal source is produced by vocal fold tissue collision and aerodynamic Bernoulli forces. The fundamental frequency F0 is set by vocal fold length, tension (controlled by cricothyroid and thyroarytenoid muscles), and subglottal pressure.
- The ideal glottal pulse (Rosenberg, Liljencrants–Fant models) has a spectrum that falls at approximately **−12 dB/octave** above the fundamental. This figure arises from a twice-integrated rectangular (triangular) pulse and corresponds to a second-order low-pass roll-off.  
  - Citation: Fant et al. (1985) LF model; confirmed by empirical measurement in Gobl & Ní Chasaide (2003), "Measures of the Glottal Source Spectrum," *JASA* [https://escholarship.org/content/qt9pn2b9qm/qt9pn2b9qm.pdf]
- The radiation characteristic of the lips adds a further **+6 dB/octave** slope, so the net radiated spectrum from an ideal source rolls off at approximately **−6 dB/octave** as heard by a microphone.
- Departure from ideal: the **open quotient (OQ)** — fraction of each glottal cycle that the folds are apart — directly controls spectral tilt. Higher OQ (breathy voice) raises H1 relative to H2 (large positive H1−H2). Lower OQ (pressed/creaky) does the opposite. H1−H2 is thus a proxy for breathiness and phonatory mode.  
  - Citation: Garellek (2019), "The Phonetics of Voice," *Oxford Handbook of Language Production* [https://idiom.ucsd.edu/~mgarellek/files/Garellek_Phonetics_of_Voice_Handbook_final.pdf]
- For a singing voice, H1−H2 typically ranges +2 to +8 dB for modal (classical) phonation, rising toward +10–15 dB for breathy onset, falling toward 0 or negative for loud pressed singing.

### Vocal-Tract Filter

- The vocal tract (approximately 17 cm for adult male, shorter for female) acts as a quarter-wave resonator with resonant peaks called **formants** (F1, F2, F3...). The tract is controlled by jaw, tongue body, tongue tip, lips, larynx height, and pharyngeal wall.
- **Physiology constrains** tract length (hence rough formant spacing) and the possible range of each articulatory gesture. **Singer control** operates through articulatory gestures that shift individual formants within that physiological range.
- What the singer cannot easily control: the physical length-to-area relationships set by skeletal anatomy. Bass voices literally have longer, wider tracts than soprano voices, producing systematically lower formants at comparable vowels.
- What the singer can and does control: larynx height (lowers F1, F2, widens pharynx), jaw opening (raises F1), lip rounding/protrusion (lowers all formants), tongue dorsum position (F1/F2 vowel space).

### Nonlinear Source–Filter Coupling

- The linear model fails when a harmonic falls near a formant, or when the epilarynx tube is strongly narrowed. Titze's two-level coupling theory (JASA 2008) identifies:
  - **Level 1:** Supraglottal acoustic pressure skews the glottal flow waveform without altering fold vibration pattern; produces harmonic distortion.
  - **Level 2:** Altered fold vibration when harmonics approach formant frequencies; most significant at high F0 (soprano high register, tenor passaggio).  
  - Citation: Titze (2008), "Nonlinear source–filter coupling in phonation: Theory," *JASA* 123(5):2733 [https://pmc.ncbi.nlm.nih.gov/articles/PMC2811547/]
- **Engineering implication:** At high soprano pitches (F0 > ~600 Hz), the assumption that source and filter are separable in the frequency domain breaks down substantially. Formant extraction from high-soprano recordings is unreliable using standard LPC.

---

## 2. Formants and the Singer's Formant

### Typical Formant Ranges (Adult Classical Singers)

All values are approximate ranges; vowel identity is the dominant variable within a voice type.

| Voice Type | F1 (Hz) | F2 (Hz) | F3 (Hz) | F4 (Hz) |
|---|---|---|---|---|
| Male (B/T) | 250–800 | 700–2400 | 1800–3000 | 2500–3500 |
| Female (S/A) | 300–1100 | 800–3200 | 2200–3500 | 3000–4200 |

- F1 and F2 encode vowel identity and are strongly vowel-dependent (200–800 Hz swing across /a/→/i/ for F1).
- F3–F5 encode voice quality, resonance strategy, and contribute to timbre.
- Sources: Sundberg (1987), *The Science of the Singing Voice*, Northern Illinois University Press; Titze (1994), *Principles of Voice Production*, Prentice Hall.

### The Singer's Formant Cluster

- In trained male singers (tenors, baritones, basses) and altos, F3, F4, and F5 cluster in the 2–4 kHz region, producing a single broad spectral prominence centered ~2400–3100 Hz. This is the **singer's formant** (Sundberg 1974).
- Mechanism: narrowing of the epilaryngeal tube to cross-sectional area ~0.2–0.36 cm² (versus neutral ~0.62 cm²), when the pharynx cross-section is at least 6× larger, creates a Helmholtz-like resonance that merges F3–F5.  
  - Citation: voicescience.org/lexicon/singers-formant/ (citing Sundberg and Titze primary work) [https://www.voicescience.org/lexicon/singers-formant/]
- **Center frequency by voice type** (from long-term spectrum of commercial recordings, Sundberg 2001 as cited in Journal of Voice):
  - Bass: ~2384 Hz (±164 Hz)
  - Baritone: ~2454 Hz (±206 Hz)
  - Tenor: ~2705 Hz (±221 Hz)
  - Soprano: ~3092 Hz (±284 Hz)
  - Citation: Journal of Voice, "Level and Center Frequency of the Singer's Formant" (PubMed 11411472) [https://pubmed.ncbi.nlm.nih.gov/11411472/]
- **Bandwidth** of the singer's formant cluster: approximately 250–500 Hz despite representing merged F3+F4+F5.
- **Level advantage:** When total SPL increases by 10 dB, the singer's formant energy increases by 16–19 dB — a superlinear amplification that provides orchestral projection without amplification.

### Soprano Exception

- At high pitches (F0 > ~600 Hz, roughly G5 and above), harmonic spacing exceeds the formant bandwidth. Sopranos **tune F1 to the first harmonic** (F0) to maximize output rather than maintaining a singer's formant cluster. The cluster bandwidth balloons to ≥2 kHz, eliminating the sharp spectral peak.
- Citation: Sundberg (1975), perceptual studies reviewed in Vos et al. (2015), "The Perception of Formant Tuning in Soprano Voices" [https://pure.royalholloway.ac.uk/ws/files/28187915/VosEtAl_FormTuningJVoicePURE.pdf]

### Between-Voice-Type vs. Within-Voice-Type Discrimination

- **Between voice types (B vs. T vs. A vs. S):** The singer's formant center frequency shifts ~300–700 Hz across the bass-to-soprano range, a span comparable to ±1–2 standard deviations of within-type spread. This makes voice-type discrimination by timbre **feasible but noisy** from spectral envelope alone.
- **Within voice type (individual discrimination):** Standard deviation of singer's formant center frequency is 164–284 Hz *within* a single voice type. Given a 300–700 Hz gap between adjacent voice types, there is meaningful overlap between the distribution tails of adjacent types. Distinguishing two basses by formant structure alone is not reliable.
- **The vowel confound:** Formant frequencies shift by hundreds of Hz with vowel change. Without knowing the vowel being sung, individual-level formant fingerprinting is essentially intractable from spectral envelope alone.
- The 2022 Scientific Reports study on "New objective timbre parameters for voice type classification" found that a Frequency of Half Energy (FHE) metric in the 2–4 kHz band correctly classified voice type with ~80–90% accuracy in controlled conditions, but accuracy dropped substantially in mixed/polyphonic contexts.  
  - Citation: (full text paywalled; results summarized via web search from voicescience.org referencing that study) [https://www.nature.com/articles/s41598-022-22821-w]

---

## 3. Vibrato: Rate, Extent, and AM/FM Coupling

### Typical Ranges

- **Rate:** 4.5–6.5 Hz for classical/opera singers; mean approximately 5.5 Hz. Population standard deviation ~0.6–0.7 Hz across individuals.  
  - Citation: Dromey et al. (2003); Sundberg review; voicescience.org/lexicon/vibrato-rate/ [https://www.voicescience.org/lexicon/vibrato-rate/]
- **Extent:** 25–100 cents peak-to-peak (±12.5 to ±50 cents). Classical norm approximately 50–120 cents peak-to-peak. Ensemble singers reduce to ~44 cents peak-to-peak mean (range ~20–80 cents).  
  - Citation: Blend in Singing Ensemble Performance study (Journal of Voice, 2016) [https://www.sciencedirect.com/science/article/abs/pii/S0892199716302156] — 44 cents mean in quartet vs. expectation up to 200 cents for soloists.
- Vibrato rate and extent show a statistically significant **negative correlation** (r = −0.62): faster vibratos tend to be narrower.
  - Citation: Vibrato Rate and Extent survey (Journal of Voice) [https://www.sciencedirect.com/science/article/abs/pii/S0892199715002064]

### AM/FM Coupling Mechanism

- Vibrato is primarily an **FM phenomenon**: the laryngeal neuromuscular system oscillates vocal fold tension at the vibrato rate, producing ±F0 modulation.
- **AM arises as a consequence of FM**, not as an independent modulation. As F0 sweeps, harmonics move into and out of formant peaks, causing amplitude modulation of each harmonic at the vibrato rate. The AM depth on a given harmonic depends on how close that harmonic sits to a formant.
- This means AM and FM are **coherent in phase** for harmonics near a formant, but the AM pattern across harmonics is non-uniform (harmonics on the skirts of formants show large AM; harmonics between formants show minimal AM).
- Citation: Peeters (2004) analysis; McAdams taxonomy [https://mtosmt.org/issues/mto.22.28.3/mto.22.28.3.mcadams.html]; voicescience.org/lexicon/vibrato/ [https://www.voicescience.org/lexicon/vibrato/]

### Common Fate: Do All Harmonics Share One Vibrato Phase/Rate?

- **Yes, in principle and largely in practice.** Because the FM source is a single neuromuscular oscillation, all harmonics of a single voice are frequency-modulated at the same rate and in the same phase. This is the acoustic basis of the perceptual "common fate" cue described by Bregman's Auditory Scene Analysis and extended by McAdams.
- Bregman's principle: components that change together (common fate) are grouped as one source. Synchronized FM across all harmonics is the strongest known grouping cue for sung tones.
  - Citation: McAdams (2022), "A Taxonomy of Orchestral Grouping Effects," Music Theory Online [https://mtosmt.org/issues/mto.22.28.3/mto.22.28.3.mcadams.html]
- **Practical caveat:** Phase coherence degrades slightly due to:
  1. Tremolo (lung pressure oscillation) can add a second, slightly different AM oscillation.
  2. At high harmonics (5th and above), measurement of phase coherence is noisy.
  3. In unison singing, the *beat* between two slightly detuned voices creates its own AM that mimics vibrato-AM coupling at the beating frequency.
- **Engineering implication:** A vibrato coherence detector across the harmonic series is a physically grounded grouping cue — if harmonics 1, 2, 3, 4... all share the same FM rate and phase, they belong to one voice. This is exploitable for F0 stream separation.

### Vibrato as an Individual Fingerprint

- Vibrato rate is determined by delays in the neural feedback loop governing vocal fold tension, making it **relatively resistant to deliberate control** and stable within an individual across time.
- Research on singer identification using vibrato features (Seneff 1988, Peeters 2004, Gerhard 2005): accuracy ~80–87% for solo singer identification from vibrato rate/extent features on 12-singer datasets.  
  - Citation: IEEE Xplore (2005), "Vibrato-Motivated Acoustic Features for Singer Identification," ICASSP [https://ieeexplore.ieee.org/document/1661330/]
- **However**: vibrato extent is modifiable by training and context (ensemble, dynamic level, vowel). Rate is more stable (~±0.3 Hz intra-session) but still shifts ±0.5–1 Hz across conditions (fatigue, pitch, loudness).
- **Critical caveat for ensemble use:** Most vibrato-fingerprinting studies used isolated solo singing. In choral blend, singers actively suppress vibrato extent (see Section 5), reducing the signal. Rate may still be present at low amplitude.

---

## 4. Individual-Voice Differentiators

### Feature Taxonomy and Robustness Ranking

Listed roughly from most to least robust under ensemble conditions:

**A. Fundamental Frequency (F0) / Pitch**
- The dominant discriminator between SATB sections. Soprano (C4–C6, ~262–1047 Hz), Alto (G3–E5, ~196–659 Hz), Tenor (C3–B4, ~131–494 Hz), Bass (E2–E4, ~82–330 Hz).
- Substantial overlap between alto and tenor (~196–330 Hz). In a chord, alto and tenor F0 can be coincident.
- Survives in mixture as long as F0 trajectories are not coincident (harmonic series overlap problem).

**B. Singer's Formant / Long-Term Spectral Envelope**
- 2–4 kHz band energy ratio (SPR metric) distinguishes trained vs. untrained singers and broadly separates voice types.
- Between-individual variation within a type is large enough to overlap adjacent types (see Section 2).
- Partially survives in mixture as an energy distribution feature, but the mix spectral envelope is the sum of all voices' contributions; deconvolving individual envelopes is ill-conditioned.

**C. Vibrato Rate**
- ~5.0–5.5 Hz mean ±0.6 Hz SD across population; individual rate stable to ~±0.3 Hz.
- Survives as a frequency-domain modulation component at F0 ± vibrato_rate sidebands.
- In mixture, each voice contributes sidebands at its own vibrato rate (if rates differ); a 2D (F0 × vibrato_rate) space potentially separates voices.
- Suppressed in ensemble blend, reducing amplitude but not necessarily rate.

**D. Spectral Tilt / H1−H2 / OQ**
- Encodes phonatory mode (breathy, modal, pressed). Individual singers have characteristic mean tilt.
- In mixture, spectral tilt is dominated by the spectral sum and is not separable without knowing individual F0 tracks.

**E. Jitter and Shimmer**
- Normal singing voice: jitter ~0.5–0.6%, shimmer ~0.2 dB.
- Jitter > ~1% or shimmer > 0.3 dB typically indicates vocal pathology or extreme register.
- In any mixture, cycle-to-cycle perturbation from multiple voices is completely incoherent and averages to near-zero in the mix signal. Jitter/shimmer are **not recoverable from a mixture**; they are pre-separation features only.

**F. Formant Frequencies and Bandwidths**
- Individual-level discrimination requires resolving formant peaks in the spectrum, which requires single-voice signals. In a dense blend, spectral peaks are contributions from multiple harmonic series; individual formant recovery is intractable without F0 separation first.

**G. Onset/Offset Transients**
- Voice onset time and spectral trajectory during onset contain individual articulatory signatures. However, in choral singing, singers are conducting simultaneous onsets with nearly identical timing (conductor-beat-synchronized). Offset transients are similarly masked.
- Potentially exploitable only in passages with staggered breathing.

**H. Breathiness / Noise Floor**
- Aspiration noise (high-frequency, 3–8 kHz, broadband) is an individual-level cue. In a mixture, noise floors add incoherently (SPL adds as square root of power), reducing the noise-to-harmonic ratio.
- Largely lost in blend.

### Conclusion on Differentiators

**For section-level SATB attribution:** F0 range is the primary and most robust cue; singer's formant energy region (2–4 kHz) provides secondary support. These survive in mixture.

**For individual-singer discrimination:** No single physical feature reliably survives in a dense monaural mixture at the precision needed to separate two singers in the same section. Vibrato rate is the least degraded; vibrato common-fate phase is the strongest structurally-grounded grouping cue.

---

## 5. Choral / Ensemble Acoustics

### How Blend Works

- Choral conductors seek "blend" in pitch, vowel, vibrato, timbre, and SPL. Achieving blend means deliberately reducing the individual cues that make voices perceptually distinct.
- Singers match their output SPL to neighbors; they modify vowel shape toward a shared target; they reduce vibrato extent and sometimes rate; they adopt uniform onset/offset timing.
- Citation: acoustic factors in choral dynamics, PMC5662467 [https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5662467/]; Blend in Singing Ensemble study (2016).

### Chorus Effect and Pitch Dispersion

- Within a section, individual singers on the same part are **not in perfect unison**. Measured F0 dispersion in professional choral sections:
  - Standard deviation of individual F0 values across a section: approximately **20–30 cents** (research consensus from multiple studies: 25–30 cents in 16-singer recordings; 20–50 cents observed range, mean ~20 cents).
  - Citation: Cuesta et al. (2019), "A Framework for Multi-f0 Modeling in SATB Choir Recordings," ISMIR [https://arxiv.org/abs/1904.05086]; web summary from unison analysis study [https://program.ismir2020.net/static/final_papers/259.pdf]
- This spread is the well-known "chorus effect": a slight detuning between voices creates beating patterns (beating frequency = F0 difference in Hz) and a perceived richness/width. It also produces **spectral smearing** of each harmonic into a band ~20–30 cents wide.
- Perceptual tolerance: listeners prefer near-zero dispersion (pure unison) but tolerate standard deviations up to ~14 cents before perceiving pitch scatter as tuning problems. Most trained choirs operate at the edge of this range.

### Effect on Aggregate Section Spectrum

- Each harmonic of a section is not a single spike but a **cluster of closely spaced spikes** from N singers, spread over ~20–50 cents. At 440 Hz, 20 cents = 5.1 Hz spread; 50 cents = 12.7 Hz spread. This is resolvable with FFT windows > ~200 ms.
- The aggregate harmonic has an amplitude envelope that oscillates at the beat frequencies between singers (typically 1–15 Hz, overlapping the vibrato rate band).
- Vibrato from individual singers (at 5–6 Hz) modulates F0 over ±25–50 cents — an extent larger than the within-section dispersion spread. This means the vibrato FM significantly exceeds the static pitch scatter, and vibrato traces from individual voices are largely masked by each other in the mixture.

### Vibrato Suppression in Ensemble

- Professional ensemble singers reduce vibrato extent substantially: mean peak-to-peak ~44 cents in a professional quartet vs. up to 200 cents for soloists (Blend study, Journal of Voice 2016).
- Conductors instruct singers to reduce vibrato to minimize beat-frequency conflicts between parts. A singer's rate persists; extent is reduced.
- **Implication:** In a well-blended choir, vibrato-based voice separation will be degraded compared to solo contexts. Rate sidebands may still be detectable at low amplitude.

### Perceptual Fusion

- Within a section, multiple voices fuse perceptually into one source because: (1) they share the same F0 or very close F0 values; (2) they share the same vibrato rate (approximately); (3) harmonics are in near-harmonic relationship; (4) they share the same onset time. All four Bregman fusion cues apply.
- Between sections (S vs. A, T vs. B), voices fuse less strongly because F0 differs, enabling auditory stream segregation.

---

## 6. Implications and Hard Limits for Single-Microphone Separation

### What Physically Survives in a Monaural Mixture

| Feature | Section-Level | Individual-Level | Notes |
|---|---|---|---|
| F0 / fundamental frequency | **Yes** — dominant cue | Partially — only if voices are not coincident | F0 overlap between A and T is the main failure mode |
| Singer's formant energy (2–4 kHz) | **Partially** — voice-type tendency | No — within-type overlap too large | Useful as a prior, not a reliable discriminator |
| Harmonic envelope shape | Partially | No | Sum of envelopes; deconvolution ill-conditioned |
| Vibrato rate sidebands | Partially — if rates differ | Potentially — with very long analysis windows | Suppressed in blend; beats from pitch scatter confound |
| Vibrato phase coherence (common fate) | **Yes** — structurally sound grouping cue | Yes — strongest individual cue | Works only if F0 tracks are first partially separated |
| Jitter/shimmer | No | No | Lost in mixture; incoherent addition |
| Formant frequencies | No (requires prior F0 sep.) | No | Cannot recover without individual F0 streams |
| Breathiness/noise | No | No | Incoherent noise floor in mix |

### (a) Multi-F0 Chord Transcription

- **Feasibility: Good for between-section separation (4 distinct F0 streams); poor for within-section.**
- When SATB sings a 4-note chord with well-separated pitches, a multi-F0 estimator faces a standard polyphonic transcription problem. Modern deep-learning approaches achieve 71.7% RPA (Raw Pitch Accuracy) on SATB data with single-voice per section.  
  - Citation: Cuesta et al. (2019/2022), Frontiers in Signal Processing [https://www.frontiersin.org/journals/signal-processing/articles/10.3389/frsip.2022.808594/full]
- Failure modes: (1) Alto-tenor range overlap (confuses the two inner voices at ~56–58% RPA); (2) octave errors due to strong second harmonics; (3) harmonic masking when one voice's harmonics align with another's fundamental.
- Alto/tenor confusion is a physics problem: their F0 ranges overlap by approximately a sixth (G3–E4, 196–330 Hz).

### (b) SATB Section Attribution

- **Feasibility: Moderate.** From a monaural recording, section attribution (which spectral content belongs to which SATB section) is tractable if the 4-voice chord has non-overlapping F0s. It degrades to near-chance when alto and tenor sing in the same pitch region.
- F0-based section attribution works at ~70–85% accuracy for well-composed excerpts where voice leading keeps sections in their characteristic ranges.
- When multiple singers per section produce the ~20–30 cent dispersion cluster, extracting a single clean F0 per section is replaced by modeling an F0 distribution — which the Cuesta 2019 framework explicitly adopts. This is achievable. Extracting *individual* singers within the distribution is not, from monaural audio.
- The deep-learning SATB separation results (SDR ~2.9 dB average, tenor SDR −7.09 dB) indicate significant residual interference even with full DNN training. The tenor case underperforms because it sits in the range most overlapped by adjacent voices.
  - Citation: Frontiers in Signal Processing 2022 (Open-Unmix experiments, Table 4–6) [https://www.frontiersin.org/journals/signal-processing/articles/10.3389/frsip.2022.808594/full]

### (c) Individual-Singer Separation (Within-Section)

- **Feasibility: Very low to infeasible from physics alone. Not achievable with current techniques from single ambient microphone.**
- Multiple singers on the same part produce nearly identical F0 (within ~20–30 cents), identical vowels, and actively similar timbres. Their harmonic series overlap almost completely. There is no known physics-based feature that separates two sopranos singing the same note in a blended ensemble from a monaural recording.
- Information-theoretic argument: the monaural mixture collapses N voices singing approximately the same F0 into a single time series. Without spatial diversity (stereo/array) or timing diversity (staggered entries), the N-to-1 channel collapse destroys independent voice information. No post-hoc processing can recover information that was never encoded in the signal.
- Vibrato offers a *partial* exception: if two sopranos singing the same note happen to have different vibrato rates (e.g., 5.0 Hz vs. 6.0 Hz), their respective contributions create sidebands at different positions relative to each harmonic, in principle separable with sufficient frequency resolution. In practice: (1) rates differ by only ~0.5–1 Hz between random singers; (2) ensemble blend suppresses vibrato extent, weakening the sidebands; (3) within-section pitch scatter creates additional amplitude modulation that confounds vibrato sideband identification.
- The 2022 Frontiers paper explicitly does not attempt individual-singer separation within a section, modeling sections as distributional F0 clouds instead.

### Hard Physical Limits Summary

1. **Monaural blind source separation of unison singers is information-theoretically limited:** a single channel collapses N simultaneous voices singing the same F0 into one time series with no spatial or independent-path diversity.
2. **Formant fingerprinting requires isolated voice signals:** in mixture, the composite spectral envelope is a superposition that cannot be uniquely decomposed without F0 separation first.
3. **Jitter/shimmer are not recoverable from polyphonic signals** regardless of algorithm.
4. **Vibrato rate is the only individual-level cue with realistic survival in blended mixture**, and only when: singers have sufficiently different rates (> ~0.5 Hz), vibrato extent is not heavily suppressed, and the target voices are not in identical pitch unison.
5. **Alto-tenor F0 overlap is a structural problem** that no monaural algorithm can eliminate without external pitch priors.

---

## Actionable Signals for the Engine

Concrete physics-grounded parameters and feature ranges for direct implementation:

### Multi-F0 Estimation (MVP: SATB chord transcription)

- **F0 search ranges per section:**
  - Soprano: 262–1047 Hz (C4–C6); primary operating range 330–880 Hz
  - Alto: 196–659 Hz (G3–E5); primary 220–523 Hz
  - Tenor: 131–494 Hz (C3–B4); primary 165–392 Hz
  - Bass: 82–330 Hz (E2–E4); primary 98–262 Hz
- **Overlap zone requiring disambiguation:** Alto/Tenor share 196–330 Hz. Use singer's formant energy ratio as a tie-breaker: energy in 2.4–2.8 kHz band favors tenor; energy in 2.8–3.2 kHz favors alto (within-gender formant inversion here is contested — treat as a weak prior, not a hard rule).
- **Harmonic series coherence:** For each F0 candidate, count how many harmonics are present and consistent with the expected −6 dB/oct slope. A score based on harmonic support (e.g., HPS or NMF-based salience) is more robust than fundamental detection alone.

### Singer's Formant Detection

- **Target band:** 2000–4000 Hz.
- **Singing Power Ratio (SPR):** measure peak amplitude in 2–4 kHz minus peak amplitude in 0–2 kHz (in dB). Trained singers show positive SPR (+3 to +10 dB); untrained/non-singer or soft choir blend shows near-zero or negative SPR.
- **Section disambiguation prior:** Bass center ~2384 Hz, Baritone ~2454 Hz, Tenor ~2705 Hz, Soprano ~3092 Hz. Standard deviations 164–284 Hz — treat as a soft Gaussian prior on voice-type given the observed spectral peak location.

### Vibrato Tracking

- **Rate band:** 4.5–6.5 Hz. Compute FM modulation depth at this rate on each tracked F0 partial. A strong modulation in this band confirms voiced singing vs. static tone.
- **Common fate grouping:** For each F0 hypothesis, measure phase of the ~5 Hz FM across harmonics 2, 3, 4, 5. If ≥3 harmonics share FM phase within ~20°, they belong to one voice — this is a physically principled grouping cue.
- **Sideband approach:** In spectrum, vibrato at rate R produces sidebands at F0 ± R, 2F0 ± R, etc. A Peaks in the spectrogram at integer-F0 ± 5.0±1.5 Hz are vibrato sidebands. If two F0-coincident voices have different vibrato rates R1 ≠ R2, their sidebands land at different positions.
- **Rate estimation precision:** Target ±0.1 Hz resolution, requiring analysis windows ≥ 1000 ms. Rate is stable within a phrase; extent varies.

### Within-Section Pitch Scatter Model

- Model each SATB section's F0 as a **Gaussian distribution** with mean = nominal pitch and SD ≈ 15–25 cents (use 20 cents as a starting point).
- Each harmonic of a section is therefore a spectral blob of width ~(harmonic_number × 20 cents). At harmonic 5 of A4 (440 Hz), the blob spans ~100 cents = ~25 Hz. This sets your frequency-resolution requirement: FFT bins must be narrower than this blur (~1–2 Hz bin width, requiring ≥0.5 s windows at 1 Hz resolution).
- Beatings within a section occur at differences between individual singer F0s, typically 1–10 Hz. These produce amplitude modulations that overlap the vibrato-rate band — a noise source for vibrato estimation.

### Spectral Slope Prior

- Glottal source: −12 dB/oct in acoustic power at the source; net radiated: −6 dB/oct.
- Use this as a harmonic amplitude prior: harmonic n should have amplitude approximately A₁/n relative to the fundamental (A₁). Deviations from this slope encode formant structure. A filter that whitens the expected −6 dB/oct slope will make formant peaks more visible.
- H1−H2 < 0 dB suggests pressed phonation or heavy chest mix; H1−H2 > 8 dB suggests breathy/falsetto. Both are meaningful for register/section classification (sopranos in high register often show near-zero or negative H1−H2 due to increased pressed phonation; choral tenors may show positive values).

### Infeasibility Guardrails

- **Do not attempt to recover jitter/shimmer from the mixture signal** — it is physically erased.
- **Do not use formant frequencies as individual fingerprints from the mixture** — requires prior voice separation.
- **Individual soprano/soprano or bass/bass separation** (two singers same section, same note) is not tractable from monaural ambient audio without non-acoustic side channels. Set as out-of-scope for MVP and north-star alike unless you can guarantee different vibrato rates (a condition you cannot control).
- **Alto-tenor chord ambiguity** at coincident pitches: have a fallback to return "ambiguous AT region" rather than forcing a section label; forced misclassification costs more than a null attribution.

---

## References (Cited Sources)

- Fant, G. (1960). *Acoustic Theory of Speech Production*. Mouton, The Hague.
- Sundberg, J. (1974). "Articulatory interpretation of the 'singing formant.'" *JASA* 55(4):838–844.
- Sundberg, J. (1987). *The Science of the Singing Voice*. Northern Illinois University Press.
- Gobl, C. & Ní Chasaide, A. (2003). "Measures of the Glottal Source Spectrum." eScholarship. [https://escholarship.org/content/qt9pn2b9qm/qt9pn2b9qm.pdf]
- Titze, I.R. (1994). *Principles of Voice Production*. Prentice Hall.
- Titze, I.R. (2008). "Nonlinear source–filter coupling in phonation: Theory." *JASA* 123(5):2733. [https://pmc.ncbi.nlm.nih.gov/articles/PMC2811547/]
- Garellek, M. (2019). "The Phonetics of Voice." *Oxford Handbook of Language Production*. [https://idiom.ucsd.edu/~mgarellek/files/Garellek_Phonetics_of_Voice_Handbook_final.pdf]
- Sundberg, J. (2001). "Level and Center Frequency of the Singer's Formant." *Journal of Voice*. PubMed 11411472. [https://pubmed.ncbi.nlm.nih.gov/11411472/]
- Vos, R. et al. (2015). "The Perception of Formant Tuning in Soprano Voices." *Journal of Voice*. [https://pure.royalholloway.ac.uk/ws/files/28187915/VosEtAl_FormTuningJVoicePURE.pdf]
- McAdams, S. (2022). "A Taxonomy of Orchestral Grouping Effects." *Music Theory Online* 28(3). [https://mtosmt.org/issues/mto.22.28.3/mto.22.28.3.mcadams.html]
- Peeters, G. (2004/2005). "Vibrato-Motivated Acoustic Features for Singer Identification." *ICASSP*. IEEE Xplore 1661330. [https://ieeexplore.ieee.org/document/1661330/]
- Cuesta, H. et al. (2019). "A Framework for Multi-F0 Modeling in SATB Choir Recordings." ISMIR. arXiv:1904.05086. [https://arxiv.org/abs/1904.05086]
- Cuesta, H. et al. (2022). "A Deep-Learning Based Framework for Source Separation, Analysis, and Synthesis of Choral Ensembles." *Frontiers in Signal Processing*. [https://www.frontiersin.org/journals/signal-processing/articles/10.3389/frsip.2022.808594/full]
- Acoustic factors in choral dynamics. PMC5662467. [https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5662467/]
- "Blend in Singing Ensemble Performance: Vibrato Production in a Vocal Quartet." *Journal of Voice* (2016). [https://www.sciencedirect.com/science/article/abs/pii/S0892199716302156]
- Standardization of acoustic measures for normal voice patterns. PMC9443584. [https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9443584/]
- voicescience.org lexicon entries (Sundberg/Titze synthesis): [https://www.voicescience.org/lexicon/singers-formant/], [https://www.voicescience.org/lexicon/vibrato/], [https://www.voicescience.org/lexicon/vibrato-rate/]
