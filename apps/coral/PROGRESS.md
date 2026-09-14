# Progress Tracking - Karpathy Autoresearch Loop

This file tracks the phases of building and executing the Karpathy autoresearch loop for the Coral note detector.

## Phases

- [x] **Phase 1: Environment Verification**
  - Verify that the local test suite runs and passes.
  - Test command: `npm test`
  - Status: Completed (102 tests passed, 1 skipped).

- [x] **Phase 2: Implement the Immutable Judge**
  - Create the evaluation script `scripts/run-bench.ts` to run the benchmark battery from `src/audio/choir-bench.ts` and print JSON metrics.
  - Add `npm run bench` command in `package.json` to execute the benchmark via `vite-node`.
  - Verify the benchmark output format.

- [x] **Phase 3: Create the Research Brief and Memory**
  - Create `program.md` in the project root to outline parameter descriptions, ranges, objectives, constraints, and instructions for agent execution.
  - Create `results.tsv` in the project root containing the current baseline scores (F1, precision, recall) extracted from the current config in `src/config/detector.ts`.

- [x] **Phase 4: Run Autoresearch Loop Iterations**
  - Run initial tuning cycles (as the agent) by tweaking parameters in `src/config/detector.ts`.
  - Evaluate each candidate, record the output in `results.tsv`, and commit or revert.
  - Document final optimized parameters and F1 improvements.

## Findings & Optimization Summary

Three experimental loops were run to optimize the note detector parameters:
1. **Lowering `salienceThreshold` to `2.8`**: Regressed F1 score to `0.756` (overall false positives increased from 10 to 14, without any gain in true positives). Status: Reverted.
2. **Raising `salienceThreshold` to `3.2`**: Successfully raised overall F1 score to **`0.793`** (precision improved from 0.828 to 0.857 by pruning 2 false positives, while retaining all 48 true positives). Status: Committed (hash `29cae41`).
3. **Raising `salienceThreshold` to `3.4`**: Regressed F1 score to `0.783` (lost one true positive note, drop in recall). Status: Reverted.

The optimal `salienceThreshold` is verified to be `3.2`, raising the overall detector F1 from `0.780` to `0.793`.

- [x] **Phase 5: High Priority DSP Bug Fixes**
  - Implement sub-octave check in `estimateF0` to prevent octave-flipping when H2 >= H1 (P1). (Completed)
  - Add test case in `src/audio/qifft.test.ts` to verify the sub-octave check. (Completed)
  - Swap lag rounding in `detectVibrato` to clamp the vibrato detector to 3–8 Hz (P2). (Completed)
  - Add guard to check `binFreqs` against `whiteningWindowBins` in `NoteDetector` (P2). (Completed)
  - Verify all unit tests and benchmarks pass. (Completed, all 104 tests pass, benchmark F1=0.793)

- [x] **Phase 6: Loop Metadata & Dashboard Integration**
  - Create `autoresearch.json` to store loop configuration and naming metadata. (Completed)
  - Update `program.md` to incorporate the official loop name. (Completed)

- [x] **Phase 7: P3 DSP Cleanups**
  - Delete dead `subOctaveRatio` property from detector config and note-detector. (Completed)
  - Implement length validation check in `NoteDetector.analyze()` to throw on short frames. (Completed)
  - Add test case verifying analyze throws on short frames. (Completed)
  - Verify all unit tests and benchmarks pass. (Completed, all 105 tests pass)




