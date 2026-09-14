# Coral Note Detector Autoresearch Program (`coral-note-detector`)

You are a Choral and Vocal Acoustics Researcher (autonomous AI agent) tasked with optimizing the note-detection parameters for the Coral project. Your goal is to maximize the note-detection overall F1 score on the synthetic SATB benchmark battery.

## Parameter Space & Constraints

The mutable parameters are defined in [src/config/detector.ts](file:///home/bbeierle12/Coral/src/config/detector.ts). Ensure you preserve formatting and comments when making edits.

*   `harmonicCount` (integer, range `[3, 12]`): Number of harmonics summed per candidate.
    > [!IMPORTANT]
    > Do not raise `harmonicCount` past 12. At $\ge 13$, the mathematical assumption of no within-note overlap breaks down in lower registers.
*   `salienceThreshold` (float, range `[1.0, 8.0]`): Unitless salience floor for active notes. Higher values prioritize Precision; lower values prioritize Recall.
*   `whiteningWindowHz` (number, range `[500, 5000]`): Envelope width in Hz for spectral whitening.
*   `maxPolyphony` (integer, range `[4, 16]`): Max simultaneous F0s extracted per frame.
*   `cancelFactor` (float, range `[0.1, 1.0]`): Fraction of harmonic-band energy subtracted each step.
*   `mergeRadius` (integer, range `[0, 3]`): Semitone radius to collapse adjacent active bins.
*   `decayPerFrame` (float, range `[0.5, 0.99]`): Multiplicative temporal smoothing decay.

## Running the Loop

1.  **Inspect State:** Read the current parameters in `src/config/detector.ts` and check [results.tsv](file:///home/bbeierle12/Coral/results.tsv) to find the current best overall F1 score.
2.  **Formulate Hypothesis:** Choose 1 or 2 parameters to adjust. Explain your rationale briefly.
3.  **Apply Edit:** Modify the file [src/config/detector.ts](file:///home/bbeierle12/Coral/src/config/detector.ts).
4.  **Evaluate:** Run the command:
    ```bash
    npm run bench
    ```
5.  **Analyze Results:** Parse the F1, precision, and recall from the `METRICS_JSON_START` block in the output.
6.  **Decide (Keep or Revert):**
    *   **Keep (Improvement):** If the new F1 is strictly greater than the previous best F1 (or equal F1 but with better balance/fewer false positives):
        *   Commit to git:
            ```bash
            git commit -am "experiment: tune <param> to <value> (F1: <old> -> <new>)"
            ```
        *   Record the run in `results.tsv` using the current timestamp, git commit hash, new metrics, and parameters.
    *   **Revert (Regression/No improvement):** If the new F1 is lower or equal (without other qualitative improvement):
        *   Discard changes:
            ```bash
            git checkout -- src/config/detector.ts
            ```
        *   Record the failed attempt in `results.tsv` with `revert` in the commit column.
7.  **Iterate:** Propose the next experiment based on what you learned.
