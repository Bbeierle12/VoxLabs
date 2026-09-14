import { useCallback, useEffect, useRef, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import {
  LiveSpectrogramView,
  type LiveRenderParams,
  type LiveSpectrogramHandle,
} from "./components/LiveSpectrogramView";
import { MicPipeline, type MicFrame } from "./audio/mic-pipeline";
import { FilePlayback } from "./audio/file-playback";
import type { DspHarmonyMsg } from "./audio/dsp-protocol";
import type { HarmonyResult } from "./audio/harmony";
import {
  RehearsalView,
  DEFAULT_REHEARSAL_SETTINGS,
  type RehearsalSettings,
} from "./components/RehearsalView";
import { decodeAudioFile } from "./audio/decoder";
import { log } from "./utils/log";

interface LevelReading {
  peakDb: number;
  rmsDb: number;
}

export type InputMode = "mic" | "file";
/** Main-pane view: the spectrogram (Coral's original) or the rehearsal cards. */
export type ViewMode = "analysis" | "rehearsal";

/** Translate sidebar rehearsal settings into the worker's setHarmony message. */
function harmonyMessage(s: RehearsalSettings) {
  return {
    a4: s.a4,
    profile: s.profile,
    vibratoHeavy: s.vibratoHeavy,
    sections: s.sections,
    familyMarginDb: s.soloMode ? 8 : 0,
  };
}

const DEFAULT_PARAMS: LiveRenderParams = {
  fftSize: 2048,
  hopSize: 512,
  minDb: -100,
  maxDb: 0,
  minFreqHz: 50,
  maxFreqHz: 8000,
  axisMode: "notes",
};

export function App() {
  const pipelineRef = useRef<MicPipeline | null>(null);
  const filePlaybackRef = useRef<FilePlayback | null>(null);
  const liveViewRef = useRef<LiveSpectrogramHandle>(null);

  const [inputMode, setInputMode] = useState<InputMode>("mic");
  const [running, setRunning] = useState(false);
  const [starting, setStarting] = useState(false);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [selectedDeviceId, setSelectedDeviceId] = useState<string | null>(null);
  const [level, setLevel] = useState<LevelReading | null>(null);
  const [params, setParams] = useState<LiveRenderParams>(DEFAULT_PARAMS);
  const [error, setError] = useState<string | null>(null);
  const [loadedFileName, setLoadedFileName] = useState<string | null>(null);
  const [decoding, setDecoding] = useState(false);
  const [frozen, setFrozen] = useState(false);
  const [view, setView] = useState<ViewMode>("analysis");
  const [harmony, setHarmony] = useState<HarmonyResult | null>(null);
  const [rehearsal, setRehearsal] = useState<RehearsalSettings>(
    DEFAULT_REHEARSAL_SETTINGS,
  );
  // The pipelines are created once (effect below) but rehearsal settings
  // change over time; a ref lets start handlers push the current settings
  // without re-creating the pipelines.
  const rehearsalRef = useRef(rehearsal);
  rehearsalRef.current = rehearsal;

  // Frame and level callbacks are stable function identities that read
  // refs internally; the pipeline doesn't re-bind on every render.
  const handleFrame = useCallback((frame: MicFrame) => {
    liveViewRef.current?.appendFrame(frame);
  }, []);
  const handleLevel = useCallback((peakDb: number, rmsDb: number) => {
    setLevel({ peakDb, rmsDb });
  }, []);
  const handleError = useCallback((err: Error) => {
    log.error("pipeline:error", err);
    setError(err.message);
  }, []);
  // Non-fatal analysis caveats (e.g. the browser kept AGC on despite our
  // constraints) reuse the error banner — the pipeline keeps running, but
  // the user must see the caveat.
  const handleWarning = useCallback((message: string) => {
    setError(message);
  }, []);
  const handleFileComplete = useCallback(() => {
    setRunning(false);
  }, []);
  // One harmony result per detection frame (~12/s at the default hop).
  // Rendering every one is cheap: the rehearsal view is a few hundred DOM
  // nodes, and React batches within the message handler.
  const handleHarmony = useCallback((msg: DspHarmonyMsg) => {
    setHarmony(msg.result);
  }, []);

  // One mic pipeline and one file-playback instance live across the App's
  // lifetime. Only one is active at a time — the input-mode toggle stops
  // whichever is running before activating the other.
  useEffect(() => {
    const mic = new MicPipeline({
      onFrame: handleFrame,
      onLevel: handleLevel,
      onError: handleError,
      onWarning: handleWarning,
      onHarmony: handleHarmony,
    });
    const file = new FilePlayback({
      onFrame: handleFrame,
      onComplete: handleFileComplete,
      onError: handleError,
      onHarmony: handleHarmony,
    });
    pipelineRef.current = mic;
    filePlaybackRef.current = file;
    return () => {
      mic.stop();
      file.stop();
      pipelineRef.current = null;
      filePlaybackRef.current = null;
    };
  }, [
    handleFrame,
    handleLevel,
    handleError,
    handleWarning,
    handleFileComplete,
    handleHarmony,
  ]);

  const refreshDevices = useCallback(async () => {
    try {
      const list = await MicPipeline.listAudioInputs();
      setDevices(list);
    } catch (err) {
      log.warn("devices:list:failed", { error: (err as Error).message });
    }
  }, []);

  useEffect(() => {
    void refreshDevices();
    const handler = () => void refreshDevices();
    navigator.mediaDevices?.addEventListener("devicechange", handler);
    return () => {
      navigator.mediaDevices?.removeEventListener("devicechange", handler);
    };
  }, [refreshDevices]);

  const handleStart = useCallback(async () => {
    const p = pipelineRef.current;
    if (!p || p.isRunning) return;
    setStarting(true);
    setError(null);
    try {
      await p.start({
        deviceId: selectedDeviceId ?? undefined,
        fftSize: params.fftSize,
        hopSize: params.hopSize,
        minFreqHz: params.minFreqHz,
        maxFreqHz: params.maxFreqHz,
      });
      p.setHarmony(harmonyMessage(rehearsalRef.current));
      liveViewRef.current?.clear();
      setHarmony(null);
      setRunning(true);
      // Re-enumerate now that permission has been granted — labels will
      // be populated this time.
      void refreshDevices();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setStarting(false);
    }
  }, [
    params.fftSize,
    params.hopSize,
    params.minFreqHz,
    params.maxFreqHz,
    refreshDevices,
    selectedDeviceId,
  ]);

  const handleStop = useCallback(() => {
    pipelineRef.current?.stop();
    filePlaybackRef.current?.stop();
    setRunning(false);
    setLevel(null);
    setFrozen(false);
  }, []);

  const handleToggleFreeze = useCallback(() => setFrozen((f) => !f), []);
  // Save PNG is async in the Tauri shell (OS save sheet, then a file write).
  // A failure there would otherwise be invisible — the Android symptom this
  // path exists to fix — so it lands in the error banner like a pipeline error.
  const handleSavePng = useCallback(async () => {
    try {
      await liveViewRef.current?.exportPng();
    } catch (err) {
      log.error("export:png:failed", err as Error);
      setError(`Save PNG failed — ${(err as Error).message}`);
    }
  }, []);

  const handleSelectDevice = useCallback(
    async (id: string) => {
      const next = id === "" ? null : id;
      setSelectedDeviceId(next);
      // If running, restart on the new device.
      const p = pipelineRef.current;
      if (p?.isRunning) {
        p.stop();
        setRunning(false);
        setLevel(null);
        setStarting(true);
        setError(null);
        try {
          await p.start({
            deviceId: next ?? undefined,
            fftSize: params.fftSize,
            hopSize: params.hopSize,
            minFreqHz: params.minFreqHz,
            maxFreqHz: params.maxFreqHz,
          });
          p.setHarmony(harmonyMessage(rehearsalRef.current));
          liveViewRef.current?.clear();
          setHarmony(null);
          setRunning(true);
        } catch (err) {
          setError((err as Error).message);
        } finally {
          setStarting(false);
        }
      }
    },
    [params.fftSize, params.hopSize, params.minFreqHz, params.maxFreqHz],
  );

  // State-updater functions must stay pure — StrictMode invokes them
  // twice and concurrent rendering may invoke-and-discard them. Pipeline
  // calls therefore happen out here, against the committed `params`, and
  // setParams receives a plain value.
  const handleParamsChange = useCallback(
    (partial: Partial<LiveRenderParams>) => {
      const next = { ...params, ...partial };
      const framingChanged =
        next.fftSize !== params.fftSize || next.hopSize !== params.hopSize;
      const freqRangeChanged =
        next.minFreqHz !== params.minFreqHz ||
        next.maxFreqHz !== params.maxFreqHz;
      const mic = pipelineRef.current;
      const file = filePlaybackRef.current;
      if (framingChanged) {
        if (mic?.isRunning) {
          mic.setFraming(next.fftSize, next.hopSize);
          liveViewRef.current?.clear();
        } else if (file?.isRunning) {
          file.setFraming(next.fftSize, next.hopSize);
          liveViewRef.current?.clear();
        }
      }
      // Frequency range only affects the worker's note detector (the
      // spectrum bytes are raw bins); no clear needed.
      if (freqRangeChanged) {
        if (mic?.isRunning) mic.setFreqRange(next.minFreqHz, next.maxFreqHz);
        else if (file?.isRunning)
          file.setFreqRange(next.minFreqHz, next.maxFreqHz);
      }
      setParams(next);
    },
    [params],
  );

  const handleSelectInputMode = useCallback(
    (mode: InputMode) => {
      if (mode === inputMode) return;
      // Stop whichever pipeline is active so the new mode starts clean.
      pipelineRef.current?.stop();
      filePlaybackRef.current?.stop();
      setRunning(false);
      setLevel(null);
      setError(null);
      if (mode === "mic") setLoadedFileName(null);
      setInputMode(mode);
    },
    [inputMode],
  );

  const handleFile = useCallback(
    async (file: File) => {
      const fp = filePlaybackRef.current;
      if (!fp) return;
      // Stop both pipelines before decoding so a slow decode can't race
      // with a still-running mic capture.
      pipelineRef.current?.stop();
      fp.stop();
      setRunning(false);
      setLevel(null);
      setError(null);
      setDecoding(true);
      setLoadedFileName(file.name);
      try {
        const buffer = await decodeAudioFile(file);
        liveViewRef.current?.clear();
        fp.start({
          buffer,
          fftSize: params.fftSize,
          hopSize: params.hopSize,
          minFreqHz: params.minFreqHz,
          maxFreqHz: params.maxFreqHz,
        });
        fp.setHarmony(harmonyMessage(rehearsalRef.current));
        setHarmony(null);
        setRunning(true);
      } catch (err) {
        setError((err as Error).message);
        setLoadedFileName(null);
      } finally {
        setDecoding(false);
      }
    },
    [params.fftSize, params.hopSize, params.minFreqHz, params.maxFreqHz],
  );

  const handleRehearsalChange = useCallback(
    (partial: Partial<RehearsalSettings>) => {
      const next = { ...rehearsal, ...partial };
      setRehearsal(next);
      const msg = harmonyMessage(next);
      const mic = pipelineRef.current;
      const file = filePlaybackRef.current;
      if (mic?.isRunning) mic.setHarmony(msg);
      else if (file?.isRunning) file.setHarmony(msg);
    },
    [rehearsal],
  );
  const handleResetDrift = useCallback(() => {
    const mic = pipelineRef.current;
    const file = filePlaybackRef.current;
    if (mic?.isRunning) mic.setHarmony({ resetDrift: true });
    else if (file?.isRunning) file.setHarmony({ resetDrift: true });
  }, []);

  const footerText = (() => {
    if (inputMode === "mic") {
      return running
        ? "Live capture in progress. Adjust params on the left; older columns keep their original coloring until they scroll off."
        : "Click Start mic to begin live capture.";
    }
    if (decoding) return `Decoding ${loadedFileName ?? "file"}…`;
    if (running)
      return `Playing ${loadedFileName ?? "file"} at real-time pace.`;
    if (loadedFileName)
      return `Finished ${loadedFileName}. Drop another file to analyze again.`;
    return "Drop an audio file (WAV, MP3, FLAC, OGG) to analyze.";
  })();

  return (
    // Phone-width viewports (below the `md` breakpoint) stack the canvas on
    // top of the controls; from `md` up the sidebar is the left rail. The
    // Android build is the reason this matters — a 288 px rail beside a
    // 1400:760 canvas leaves no usable plot on a portrait phone.
    <div className="flex min-h-screen flex-col bg-neutral-950 text-neutral-100 md:flex-row">
      <Sidebar
        inputMode={inputMode}
        running={running}
        starting={starting}
        decoding={decoding}
        loadedFileName={loadedFileName}
        devices={devices}
        selectedDeviceId={selectedDeviceId}
        level={level}
        params={params}
        error={error}
        onSelectInputMode={handleSelectInputMode}
        onStart={handleStart}
        onStop={handleStop}
        onSelectDevice={handleSelectDevice}
        onRefreshDevices={refreshDevices}
        onParamsChange={handleParamsChange}
        onFile={handleFile}
        frozen={frozen}
        onToggleFreeze={handleToggleFreeze}
        onSavePng={handleSavePng}
        view={view}
        onSelectView={setView}
        rehearsal={rehearsal}
        onRehearsalChange={handleRehearsalChange}
        onResetDrift={handleResetDrift}
      />
      <main className="order-1 min-w-0 flex-1 p-3 md:order-2 md:p-6">
        {/* The spectrogram stays mounted (hidden) in rehearsal view so its
            ring buffer keeps filling and a switch back shows history. */}
        <div hidden={view !== "analysis"}>
          <LiveSpectrogramView
            ref={liveViewRef}
            params={params}
            frozen={frozen}
          />
        </div>
        {view === "rehearsal" && (
          <RehearsalView result={harmony} cfg={rehearsal} listening={running} />
        )}
        <p className="mt-3 text-xs text-neutral-500">{footerText}</p>
      </main>
    </div>
  );
}
