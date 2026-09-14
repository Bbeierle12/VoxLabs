import { useMemo } from "react";
import { SPEC_SRC_CEIL_DB } from "../config/spectrogram-encoding";
import type { LiveRenderParams } from "./LiveSpectrogramView";
import { FileDrop } from "./FileDrop";
import type { InputMode, ViewMode } from "../App";
import { RehearsalControls, type RehearsalSettings } from "./RehearsalView";

interface LevelReading {
  peakDb: number;
  rmsDb: number;
}

interface Props {
  inputMode: InputMode;
  running: boolean;
  starting: boolean;
  decoding: boolean;
  loadedFileName: string | null;
  devices: MediaDeviceInfo[];
  selectedDeviceId: string | null;
  level: LevelReading | null;
  params: LiveRenderParams;
  error: string | null;
  onSelectInputMode: (mode: InputMode) => void;
  onStart: () => void;
  onStop: () => void;
  onSelectDevice: (id: string) => void;
  onRefreshDevices: () => void;
  onParamsChange: (partial: Partial<LiveRenderParams>) => void;
  onFile: (file: File) => void;
  frozen: boolean;
  onToggleFreeze: () => void;
  onSavePng: () => void;
  view: ViewMode;
  onSelectView: (view: ViewMode) => void;
  rehearsal: RehearsalSettings;
  onRehearsalChange: (partial: Partial<RehearsalSettings>) => void;
  onResetDrift: () => void;
}

const FFT_SIZES = [512, 1024, 2048, 4096] as const;
const FREQ_OPTIONS = [
  20, 50, 100, 200, 500, 1000, 2000, 4000, 8000, 12000, 16000,
] as const;

/**
 * Control surface for the live spectrogram. From the `md` breakpoint up it
 * is a left rail beside the canvas; on phone-width viewports it stacks
 * full-width below the canvas (see App). All state is owned by App.
 *
 * The input-mode toggle at the top selects between live mic capture
 * (primary) and decoded-file playback (secondary). The STFT / dB clamp /
 * frequency-range sections apply to whichever pipeline is active —
 * params are interpreted live in both modes (atomic StreamingStft swap
 * for framing changes; next-column for render changes).
 */
export function Sidebar({
  inputMode,
  running,
  starting,
  decoding,
  loadedFileName,
  devices,
  selectedDeviceId,
  level,
  params,
  error,
  onSelectInputMode,
  onStart,
  onStop,
  onSelectDevice,
  onRefreshDevices,
  onParamsChange,
  onFile,
  frozen,
  onToggleFreeze,
  onSavePng,
  view,
  onSelectView,
  rehearsal,
  onRehearsalChange,
  onResetDrift,
}: Props) {
  const hopOptions = useMemo(() => {
    const divisors = [2, 4, 8, 16];
    return divisors
      .map((d) => params.fftSize / d)
      .filter((h) => h >= 32 && Number.isInteger(h));
  }, [params.fftSize]);

  return (
    <aside className="order-2 w-full shrink-0 space-y-5 border-t border-neutral-800 bg-neutral-900/60 p-5 text-sm text-neutral-200 md:order-1 md:w-72 md:border-r md:border-t-0">
      <header>
        <h1 className="text-lg font-semibold tracking-tight text-neutral-100">
          Coral
        </h1>
        <p className="text-xs text-neutral-500">
          {view === "rehearsal"
            ? "Choral harmony analyzer"
            : inputMode === "mic"
              ? "Live mic spectrogram"
              : "File spectrogram"}
        </p>
      </header>

      <section className="space-y-2">
        <label className="text-xs uppercase tracking-wide text-neutral-400">
          View
        </label>
        <div className="inline-flex w-full rounded border border-neutral-700 overflow-hidden">
          <ModeTab
            active={view === "analysis"}
            onClick={() => onSelectView("analysis")}
            label="Spectrogram"
          />
          <ModeTab
            active={view === "rehearsal"}
            onClick={() => onSelectView("rehearsal")}
            label="Rehearsal"
            borderLeft
          />
        </div>
      </section>

      <section className="space-y-2">
        <label className="text-xs uppercase tracking-wide text-neutral-400">
          Input source
        </label>
        <div className="inline-flex w-full rounded border border-neutral-700 overflow-hidden">
          <ModeTab
            active={inputMode === "mic"}
            onClick={() => onSelectInputMode("mic")}
            label="Mic"
          />
          <ModeTab
            active={inputMode === "file"}
            onClick={() => onSelectInputMode("file")}
            label="File"
            borderLeft
          />
        </div>
      </section>

      {inputMode === "mic" ? (
        <MicControls
          running={running}
          starting={starting}
          devices={devices}
          selectedDeviceId={selectedDeviceId}
          level={level}
          onStart={onStart}
          onStop={onStop}
          onSelectDevice={onSelectDevice}
          onRefreshDevices={onRefreshDevices}
        />
      ) : (
        <FileControls
          running={running}
          decoding={decoding}
          loadedFileName={loadedFileName}
          onFile={onFile}
          onStop={onStop}
        />
      )}

      {error && (
        <p className="rounded border border-red-700 bg-red-950/40 p-2 text-xs text-red-200">
          {error}
        </p>
      )}

      {view === "rehearsal" && (
        <RehearsalControls
          settings={rehearsal}
          onChange={onRehearsalChange}
          onResetDrift={onResetDrift}
        />
      )}

      {view === "analysis" && (
        <>
          <section className="space-y-2">
            <h2 className="text-xs uppercase tracking-wide text-neutral-400">
              Capture
            </h2>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={onToggleFreeze}
                className={[
                  "flex-1 rounded px-3 py-1.5 text-xs font-medium transition-colors",
                  frozen
                    ? "bg-amber-600 text-white hover:bg-amber-500"
                    : "bg-neutral-800 text-neutral-200 hover:bg-neutral-700",
                ].join(" ")}
              >
                {frozen ? "Frozen ❄" : "Freeze"}
              </button>
              <button
                type="button"
                onClick={onSavePng}
                className="flex-1 rounded bg-neutral-800 px-3 py-1.5 text-xs font-medium text-neutral-200 hover:bg-neutral-700"
              >
                Save PNG
              </button>
            </div>
          </section>

          <section className="space-y-3">
            <h2 className="text-xs uppercase tracking-wide text-neutral-400">
              STFT
            </h2>
            <Field label="FFT size">
              <select
                value={params.fftSize}
                onChange={(e) => {
                  const fftSize = Number(e.target.value);
                  const hopSize = Math.min(params.hopSize, fftSize / 2);
                  onParamsChange({ fftSize, hopSize });
                }}
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
              >
                {FFT_SIZES.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Hop size">
              <select
                value={params.hopSize}
                onChange={(e) =>
                  onParamsChange({ hopSize: Number(e.target.value) })
                }
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
              >
                {hopOptions.map((h) => (
                  <option key={h} value={h}>
                    {h} ({Math.round((params.fftSize / h) * 100) / 100}×
                    overlap)
                  </option>
                ))}
              </select>
            </Field>
          </section>

          <section className="space-y-3">
            <h2 className="text-xs uppercase tracking-wide text-neutral-400">
              dB clamp
            </h2>
            <RangeField
              label="Min dB"
              value={params.minDb}
              min={-140}
              max={params.maxDb - 1}
              step={1}
              onChange={(minDb) => onParamsChange({ minDb })}
            />
            <RangeField
              label="Max dB"
              value={params.maxDb}
              min={params.minDb + 1}
              max={SPEC_SRC_CEIL_DB}
              step={1}
              onChange={(maxDb) => onParamsChange({ maxDb })}
            />
          </section>

          <section className="space-y-3">
            <h2 className="text-xs uppercase tracking-wide text-neutral-400">
              Frequency range
            </h2>
            <Field label="Min Hz">
              <select
                value={params.minFreqHz}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (v < params.maxFreqHz) onParamsChange({ minFreqHz: v });
                }}
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
              >
                {FREQ_OPTIONS.filter((f) => f < params.maxFreqHz).map((f) => (
                  <option key={f} value={f}>
                    {f}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Max Hz">
              <select
                value={params.maxFreqHz}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (v > params.minFreqHz) onParamsChange({ maxFreqHz: v });
                }}
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
              >
                {FREQ_OPTIONS.filter((f) => f > params.minFreqHz).map((f) => (
                  <option key={f} value={f}>
                    {f}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Axis labels">
              <div className="inline-flex rounded border border-neutral-700 overflow-hidden">
                <button
                  type="button"
                  onClick={() => onParamsChange({ axisMode: "notes" })}
                  className={[
                    "px-3 py-1 text-xs transition-colors",
                    params.axisMode === "notes"
                      ? "bg-emerald-600 text-white"
                      : "bg-neutral-950 text-neutral-300 hover:bg-neutral-800",
                  ].join(" ")}
                >
                  Notes
                </button>
                <button
                  type="button"
                  onClick={() => onParamsChange({ axisMode: "hz" })}
                  className={[
                    "px-3 py-1 text-xs transition-colors border-l border-neutral-700",
                    params.axisMode === "hz"
                      ? "bg-emerald-600 text-white"
                      : "bg-neutral-950 text-neutral-300 hover:bg-neutral-800",
                  ].join(" ")}
                >
                  Hz
                </button>
              </div>
            </Field>
          </section>
        </>
      )}
    </aside>
  );
}

function ModeTab({
  active,
  onClick,
  label,
  borderLeft = false,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  borderLeft?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "flex-1 px-3 py-1.5 text-xs transition-colors",
        borderLeft ? "border-l border-neutral-700" : "",
        active
          ? "bg-emerald-600 text-white"
          : "bg-neutral-950 text-neutral-300 hover:bg-neutral-800",
      ].join(" ")}
    >
      {label}
    </button>
  );
}

function MicControls({
  running,
  starting,
  devices,
  selectedDeviceId,
  level,
  onStart,
  onStop,
  onSelectDevice,
  onRefreshDevices,
}: {
  running: boolean;
  starting: boolean;
  devices: MediaDeviceInfo[];
  selectedDeviceId: string | null;
  level: LevelReading | null;
  onStart: () => void;
  onStop: () => void;
  onSelectDevice: (id: string) => void;
  onRefreshDevices: () => void;
}) {
  return (
    <>
      <section className="space-y-2">
        <button
          type="button"
          onClick={running ? onStop : onStart}
          disabled={starting}
          className={[
            "w-full rounded px-3 py-2 font-medium transition-colors",
            running
              ? "bg-red-600 hover:bg-red-500 text-white"
              : "bg-emerald-600 hover:bg-emerald-500 text-white",
            starting ? "opacity-60 cursor-wait" : "",
          ].join(" ")}
        >
          {starting ? "Starting…" : running ? "Stop mic" : "Start mic"}
        </button>
      </section>

      <section className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="text-xs uppercase tracking-wide text-neutral-400">
            Input device
          </label>
          <button
            type="button"
            onClick={onRefreshDevices}
            className="text-xs text-neutral-400 hover:text-neutral-200 underline"
          >
            refresh
          </button>
        </div>
        <select
          value={selectedDeviceId ?? ""}
          onChange={(e) => onSelectDevice(e.target.value)}
          className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs"
        >
          <option value="">System default</option>
          {devices.map((d) => (
            <option key={d.deviceId} value={d.deviceId}>
              {d.label || `(unnamed input · ${d.deviceId.slice(0, 8)})`}
            </option>
          ))}
        </select>
        {devices.every((d) => !d.label) && (
          <p className="text-[11px] text-neutral-500">
            Device labels appear after you grant mic permission once.
          </p>
        )}
      </section>

      <section className="space-y-1.5">
        <label className="text-xs uppercase tracking-wide text-neutral-400">
          Level
        </label>
        <LevelBar reading={level} />
      </section>
    </>
  );
}

function FileControls({
  running,
  decoding,
  loadedFileName,
  onFile,
  onStop,
}: {
  running: boolean;
  decoding: boolean;
  loadedFileName: string | null;
  onFile: (file: File) => void;
  onStop: () => void;
}) {
  return (
    <section className="space-y-2">
      <FileDrop onFile={onFile} disabled={decoding} />
      {loadedFileName && (
        <p
          className="truncate text-[11px] text-neutral-400"
          title={loadedFileName}
        >
          {decoding ? "Decoding " : running ? "Playing " : "Loaded "}
          <span className="font-mono text-neutral-200">{loadedFileName}</span>
        </p>
      )}
      {running && (
        <button
          type="button"
          onClick={onStop}
          className="w-full rounded bg-red-600 hover:bg-red-500 px-3 py-1.5 text-xs font-medium text-white"
        >
          Stop playback
        </button>
      )}
    </section>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <span className="block text-[11px] text-neutral-500">{label}</span>
      {children}
    </div>
  );
}

function RangeField({
  label,
  value,
  min,
  max,
  step,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-[11px] text-neutral-500">
        <span>{label}</span>
        <span className="font-mono text-neutral-300">{value} dB</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-emerald-500"
      />
    </div>
  );
}

function LevelBar({ reading }: { reading: LevelReading | null }) {
  // Map -60..0 dB onto a 0..1 fill ratio. Below -60 dB the bar is empty;
  // above 0 dB it's full and tinted red as a clipping warning.
  const clampDb = (db: number) => Math.max(-60, Math.min(6, db));
  const ratio = (db: number) => (clampDb(db) + 60) / 66;

  const peakDb = reading ? reading.peakDb : -Infinity;
  const rmsDb = reading ? reading.rmsDb : -Infinity;
  const peakRatio = isFinite(peakDb) ? ratio(peakDb) : 0;
  const rmsRatio = isFinite(rmsDb) ? ratio(rmsDb) : 0;
  const clipping = isFinite(peakDb) && peakDb >= 0;

  return (
    <div className="space-y-1">
      <div className="relative h-3 w-full overflow-hidden rounded bg-neutral-950 border border-neutral-800">
        <div
          className={[
            "absolute inset-y-0 left-0 transition-[width] duration-75",
            clipping ? "bg-red-500" : "bg-emerald-500/80",
          ].join(" ")}
          style={{ width: `${rmsRatio * 100}%` }}
        />
        <div
          className="absolute inset-y-0 w-px bg-neutral-100"
          style={{ left: `${peakRatio * 100}%` }}
        />
      </div>
      <div className="flex justify-between font-mono text-[10px] text-neutral-500">
        <span>peak {isFinite(peakDb) ? `${peakDb.toFixed(0)} dB` : "—"}</span>
        <span>rms {isFinite(rmsDb) ? `${rmsDb.toFixed(0)} dB` : "—"}</span>
      </div>
    </div>
  );
}
