import { useCallback, useEffect, useState } from "react";
import type {
  HarmonyResult,
  HarmonyConfig,
  Voice,
  Target,
} from "../audio/harmony";
import { band, snapshotText, NOTE_NAMES, PROFILES } from "../audio/harmony";
import type { Section } from "../audio/section-labeler";

/**
 * Rehearsal view (ported from the Choral Harmony Analyzer): four FIXED
 * section cards, the harmony card (chord, consonance, drift, pair table,
 * balance) and the snapshot log. Reads one HarmonyResult per detection frame.
 *
 * Presentation rules come from the research memos: targets appear only once a
 * chord has held 300 ms; the anchor (lowest chord tone) never gets a "move"
 * instruction; ±10 c in tune / 20 c out; a section whose partial is wider
 * than 30 c is reported as "scattered" rather than sharp or flat.
 */

export interface RehearsalViewProps {
  result: HarmonyResult | null;
  cfg: HarmonyConfig;
  listening: boolean;
}

const SECTIONS: readonly {
  id: Section;
  name: string;
  range: string;
  color: string;
  hex: string;
}[] = [
  {
    id: "B",
    name: "Bass",
    range: "E2 – E4",
    color: "text-indigo-300",
    hex: "#818cf8",
  },
  {
    id: "T",
    name: "Tenor",
    range: "C3 – A4",
    color: "text-teal-300",
    hex: "#2dd4bf",
  },
  {
    id: "A",
    name: "Alto",
    range: "F3 – F5",
    color: "text-amber-300",
    hex: "#fbbf24",
  },
  {
    id: "S",
    name: "Soprano",
    range: "C4 – C6",
    color: "text-rose-300",
    hex: "#fb7185",
  },
];
const BAND_HEX = {
  in: "#34d399",
  marginal: "#fbbf24",
  out: "#fb7185",
} as const;
const clamp50 = (c: number) => 50 + Math.max(-50, Math.min(50, c));
const fmt = (c: number, d = 1) => `${c >= 0 ? "+" : ""}${c.toFixed(d)}c`;

function SectionCard({
  sec,
  voice,
  target,
  result,
  cfg,
}: {
  sec: (typeof SECTIONS)[number];
  voice: Voice | null;
  target: Target | null;
  result: HarmonyResult | null;
  cfg: HarmonyConfig;
}) {
  const active = voice != null;
  const scattered = voice?.scatterBand === "scattered";
  let dotColor = "#64748b";
  let line: React.ReactNode = "";
  let errTxt = "—";
  if (voice && target) {
    if (target.anchor) {
      dotColor = "#38bdf8";
      line = (
        <>
          <b className={sec.color}>Anchor</b> · {target.role} — the other
          sections tune to this note
        </>
      );
      errTxt = "anchor";
    } else if (scattered) {
      dotColor = BAND_HEX.out;
      line = (
        <>
          <b className={sec.color}>Section scattered</b> (±
          {voice.scatter?.toFixed(0)}c on partial {voice.scatterK}) — no single
          pitch to tune
        </>
      );
    } else if (!result?.settled) {
      line = "Settling… (chord held < 300 ms)";
      errTxt = "…";
    } else {
      const b = band(target.err, voice.heldMs, cfg);
      dotColor = BAND_HEX[b];
      const mag = Math.abs(target.err);
      line =
        b === "in" ? (
          <>
            <b className={sec.color}>In tune</b> · {target.role} (
            {target.ratioTxt}) target {target.tgt.toFixed(1)} Hz
          </>
        ) : (
          <>
            Sing{" "}
            <b className={sec.color}>
              {mag.toFixed(1)}c {target.err > 0 ? "lower" : "higher"}
            </b>{" "}
            for the {target.role} ({target.ratioTxt}) · target{" "}
            {target.tgt.toFixed(1)} Hz
            {b === "marginal" ? " · marginal" : ""}
          </>
        );
      errTxt = fmt(target.err);
    }
  } else if (voice) {
    line =
      result && result.voices.length > 1
        ? "No target (cluster / unmatched pitch class)"
        : "Single voice · ET reference only";
  }
  return (
    <div
      className="min-h-[170px] rounded-xl border bg-neutral-900/70 p-3 transition-shadow"
      style={{
        borderColor: active ? sec.hex : "#334155",
        boxShadow: active
          ? `0 0 0 1px ${sec.hex}, 0 0 24px -8px ${sec.hex}`
          : "none",
      }}
    >
      <div
        className={`flex justify-between text-[11px] font-bold uppercase tracking-wider ${sec.color}`}
      >
        <span>
          {sec.name}
          {voice?.label === "A/T?" ? (
            <span className="ml-1 font-normal normal-case tracking-normal text-neutral-400">
              (A or T?)
            </span>
          ) : null}
        </span>
        <span className="font-normal normal-case tracking-normal text-neutral-500">
          {sec.range}
        </span>
      </div>
      {!voice ? (
        <div className="mt-6 text-sm text-neutral-500">
          Resting · awaiting section input
        </div>
      ) : (
        <>
          <div className="mt-1 text-3xl font-bold leading-tight">
            {voice.name}
            <sub className="text-sm">{voice.oct}</sub>
          </div>
          <div className="text-xs text-neutral-400">
            {voice.f.toFixed(2)} Hz · ET {voice.etHz.toFixed(2)} Hz
          </div>
          <div className="mt-1 min-h-[34px] text-xs text-neutral-200">
            {line}
          </div>
        </>
      )}
      {/* Gauge: centre = 12-TET, amber = target, violet/grey ticks = pure / equal, dot = live pitch. */}
      <div className="relative mt-2 h-[22px] rounded-md border border-neutral-700 bg-neutral-950">
        {[0, 25, 50, 75, 100].map((p) => (
          <span
            key={p}
            className="absolute bottom-0 top-0 w-px"
            style={{
              left: `${p}%`,
              background: p === 50 ? "#94a3b8" : "#334155",
              width: p === 50 ? 2 : 1,
            }}
          />
        ))}
        {voice && target && (
          <>
            {cfg.profile !== "pure" && (
              <span
                className="absolute bottom-1.5 top-1.5 w-0.5 rounded"
                style={{
                  left: `${clamp50(target.alt.pure)}%`,
                  background: "#a78bfa",
                  opacity: 0.7,
                  transform: "translateX(-50%)",
                }}
              />
            )}
            {cfg.profile !== "equal" && (
              <span
                className="absolute bottom-1.5 top-1.5 w-0.5 rounded"
                style={{
                  left: `${clamp50(target.alt.equal)}%`,
                  background: "#94a3b8",
                  opacity: 0.7,
                  transform: "translateX(-50%)",
                }}
              />
            )}
            <span
              className="absolute bottom-0.5 top-0.5 w-[3px] rounded"
              style={{
                left: `${clamp50(target.jiCents)}%`,
                background: "#fbbf24",
                transform: "translateX(-50%)",
              }}
            />
          </>
        )}
        {voice && (
          <span
            className="absolute top-1 h-3.5 w-3.5 rounded-full shadow"
            style={{
              left: `${clamp50(voice.cents)}%`,
              background: dotColor,
              transform: "translateX(-50%)",
              transition: "left 90ms linear",
            }}
          />
        )}
        <span className="absolute -bottom-3.5 left-0 text-[9px] text-neutral-500">
          −50c
        </span>
        <span className="absolute -bottom-3.5 left-1/2 -translate-x-1/2 text-[9px] text-neutral-500">
          0 (12-TET)
        </span>
        <span className="absolute -bottom-3.5 right-0 text-[9px] text-neutral-500">
          +50c
        </span>
      </div>
      <div className="mt-4 flex flex-wrap justify-between gap-1 text-[11px] text-neutral-400">
        <span>
          ET dev{" "}
          <b className="text-neutral-100">{voice ? fmt(voice.cents) : "—"}</b>
        </span>
        <span>
          Scatter{" "}
          <b
            style={{
              color: scattered
                ? BAND_HEX.out
                : voice?.scatterBand === "loose"
                  ? BAND_HEX.marginal
                  : undefined,
            }}
            className="text-neutral-100"
          >
            {voice?.scatter != null ? `±${voice.scatter.toFixed(0)}c` : "—"}
          </b>
        </span>
        <span>
          Target err{" "}
          <b
            style={{
              color:
                voice &&
                target &&
                !target.anchor &&
                result?.settled &&
                !scattered
                  ? dotColor
                  : undefined,
            }}
            className="text-neutral-100"
          >
            {errTxt}
          </b>
        </span>
      </div>
    </div>
  );
}

function Tile({
  label,
  value,
  color,
  children,
}: {
  label: React.ReactNode;
  value: React.ReactNode;
  color?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="rounded-lg bg-neutral-800/70 px-2 py-1.5">
      <small className="block text-[10px] uppercase tracking-wider text-neutral-400">
        {label}
      </small>
      <b className="text-base" style={{ color }}>
        {value}
      </b>
      {children}
    </div>
  );
}

const LOG_KEY = "coral.rehearsal.log.v1";
interface Snap {
  t: number;
  text: string;
}
function loadLog(): Snap[] {
  try {
    return JSON.parse(localStorage.getItem(LOG_KEY) || "[]") as Snap[];
  } catch {
    return [];
  }
}

export function RehearsalView({ result, cfg, listening }: RehearsalViewProps) {
  const [log, setLog] = useState<Snap[]>(() => loadLog());
  const [status, setStatus] = useState<string | null>(null);
  useEffect(() => {
    try {
      localStorage.setItem(LOG_KEY, JSON.stringify(log));
    } catch {
      /* private mode: keep in memory */
    }
  }, [log]);
  useEffect(() => {
    if (!status) return;
    const id = setTimeout(() => setStatus(null), 2500);
    return () => clearTimeout(id);
  }, [status]);

  const copy = useCallback(async (t: string) => {
    try {
      await navigator.clipboard.writeText(t);
      setStatus("Copied");
    } catch {
      setStatus("Copy failed — select the text and copy manually");
    }
  }, []);
  const snapshot = useCallback(() => {
    if (!result) return;
    const text = snapshotText(result, Date.now(), cfg);
    if (!text) {
      setStatus("Nothing to snapshot — no voices detected");
      return;
    }
    setLog((l) => [...l, { t: Date.now(), text }]);
    setStatus("Snapshot saved");
  }, [result, cfg]);

  const voices = result?.voices ?? [];
  const byIdx = (s: Section) => voices.findIndex((v) => v.section === s);
  const counts = result?.counts ?? { B: 0, T: 0, A: 0, S: 0 };
  const tot = counts.B + counts.T + counts.A + counts.S || 1;
  const drift = result?.drift ?? null;
  const id = result?.id ?? null;

  return (
    <div className="space-y-3">
      <section className="grid grid-cols-2 gap-2.5 md:grid-cols-4">
        {SECTIONS.map((sec) => {
          const i = byIdx(sec.id);
          return (
            <SectionCard
              key={sec.id}
              sec={sec}
              voice={i >= 0 ? (voices[i] as Voice) : null}
              target={i >= 0 ? (result?.tg[i] ?? null) : null}
              result={result}
              cfg={cfg}
            />
          );
        })}
      </section>
      <p className="text-[11px] text-neutral-500">
        Cards are fixed per section. Gauge: centre = 12-TET, amber = target for
        the identified chord (anchored on the lowest chord tone), violet / grey
        ticks = where pure / equal would sit, dot = live pitch (±50c). Green
        ≤10c, amber ≤20c, red beyond. "A or T?" marks the alto/tenor overlap
        register where pitch alone cannot say which section is singing.
      </p>

      <section className="rounded-xl border border-neutral-800 bg-neutral-900/70 p-3">
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="text-2xl font-bold">
            {!voices.length
              ? "—"
              : id?.cluster
                ? "Cluster"
                : `${NOTE_NAMES[id!.root]} ${id!.chord.name}`}
          </span>
          <span className="text-neutral-400">
            {!voices.length
              ? listening
                ? "No voices detected"
                : "Listening…"
              : id?.cluster
                ? "non-tertian / polyphonic texture"
                : `${voices.length} voice${voices.length > 1 ? "s" : ""}${id?.missing ? ` · ${id.missing} chord tone missing` : ""}`}
          </span>
          {id && id.chord.ratio !== "—" && (
            <span className="font-mono text-xs text-sky-300">
              pure {id.chord.ratio} · target{" "}
              {PROFILES[cfg.profile].name.toLowerCase()}
            </span>
          )}
        </div>
        <div className="mt-2 grid grid-cols-2 gap-1.5 md:grid-cols-3">
          <Tile
            label="Consonance index"
            value={result?.cons == null ? "—" : `${result.cons} %`}
          >
            <div className="mt-1 h-2 overflow-hidden rounded bg-neutral-950">
              <i
                className="block h-full"
                style={{
                  width: `${result?.cons ?? 0}%`,
                  background: "linear-gradient(90deg,#fb7185,#fbbf24,#34d399)",
                }}
              />
            </div>
          </Tile>
          <Tile
            label="Drift since start"
            value={drift == null ? "—" : fmt(drift)}
            color={
              drift == null
                ? undefined
                : Math.abs(drift) < 10
                  ? BAND_HEX.in
                  : Math.abs(drift) < 25
                    ? BAND_HEX.marginal
                    : BAND_HEX.out
            }
          />
          <Tile
            label="Chord held"
            value={
              result && voices.length
                ? `${(result.chordHeldMs / 1000).toFixed(1)} s`
                : "—"
            }
          />
          <Tile label="Active voices" value={voices.length} />
          <Tile
            label="Mean |ET dev|"
            value={
              voices.length
                ? `${(voices.reduce((t, v) => t + Math.abs(v.cents), 0) / voices.length).toFixed(1)}c`
                : "—"
            }
          />
          <Tile
            label="Mean |target err|"
            value={(() => {
              const je = (result?.tg ?? []).filter(
                (t): t is Target => !!t && !t.anchor,
              );
              return je.length
                ? `${(je.reduce((t, j) => t + Math.abs(j.err), 0) / je.length).toFixed(1)}c`
                : "—";
            })()}
          />
        </div>
        {result && result.pairs.length > 0 && (
          <div className="mt-2 overflow-x-auto">
            <table className="w-full text-xs">
              <thead>
                <tr className="text-left text-[10px] uppercase tracking-wider text-neutral-400">
                  <th className="py-1 pr-2">Pair</th>
                  <th className="py-1 pr-2">Interval</th>
                  <th className="py-1 pr-2 text-right">Ratio</th>
                  <th className="py-1 pr-2 text-right">Cents</th>
                  <th className="py-1 pr-2 text-right">Err</th>
                  <th className="py-1 text-right">Beat Hz</th>
                </tr>
              </thead>
              <tbody className="font-mono">
                {result.pairs.map((p) => {
                  const b = band(p.err, null, cfg);
                  return (
                    <tr
                      key={`${p.a.section}-${p.b.section}`}
                      className="border-t border-neutral-800"
                    >
                      <td className="py-1 pr-2 font-sans">
                        {p.a.name}
                        {p.a.oct}–{p.b.name}
                        {p.b.oct}
                      </td>
                      <td className="py-1 pr-2 font-sans">{p.name}</td>
                      <td className="py-1 pr-2 text-right">{p.ratio}</td>
                      <td className="py-1 pr-2 text-right">
                        {p.cents.toFixed(1)}
                      </td>
                      <td
                        className="py-1 pr-2 text-right"
                        style={{ color: BAND_HEX[b] }}
                      >
                        {fmt(p.err)}
                      </td>
                      <td className="py-1 text-right">{p.beat.toFixed(2)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
        <div className="mt-2 text-[10px] uppercase tracking-wider text-neutral-400">
          Choir register balance
        </div>
        <div className="mt-1 flex h-2.5 overflow-hidden rounded-md border border-neutral-700 bg-neutral-950">
          {SECTIONS.map((s) => (
            <i
              key={s.id}
              className="block h-full"
              style={{
                width: `${(100 * counts[s.id]) / tot}%`,
                background: s.hex,
              }}
            />
          ))}
        </div>
        <div className="mt-1 flex justify-between text-[10px]">
          {SECTIONS.map((s) => (
            <span key={s.id} className={s.color}>
              {s.id} <b>{counts[s.id]}</b>
            </span>
          ))}
        </div>
      </section>

      <section className="rounded-xl border border-neutral-800 bg-neutral-900/70 p-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-xs uppercase tracking-wider text-neutral-400">
            Rehearsal snapshot log
          </h2>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={snapshot}
              className="rounded bg-sky-600 px-3 py-1 text-xs font-medium text-white hover:bg-sky-500"
            >
              Snapshot
            </button>
            <button
              type="button"
              onClick={() => copy(log.map((s) => s.text).join("\n\n"))}
              className="rounded bg-neutral-800 px-3 py-1 text-xs hover:bg-neutral-700"
              disabled={!log.length}
            >
              Copy all
            </button>
            <button
              type="button"
              onClick={() => setLog([])}
              className="rounded bg-neutral-800 px-3 py-1 text-xs hover:bg-neutral-700"
              disabled={!log.length}
            >
              Clear
            </button>
          </div>
        </div>
        {status && (
          <p className="mt-1 text-[11px] text-neutral-400">{status}</p>
        )}
        {!log.length ? (
          <p className="mt-2 text-[11px] text-neutral-500">
            No snapshots yet. Tap Snapshot while the choir holds a chord to
            freeze the numbers for review.
          </p>
        ) : (
          <div className="mt-2 space-y-2">
            {log
              .slice()
              .reverse()
              .map((s, i) => (
                <div
                  key={s.t}
                  className="rounded-lg bg-neutral-800/70 p-2 text-xs"
                >
                  <div className="flex justify-between text-[11px] text-neutral-400">
                    <span>Take {log.length - i}</span>
                    <span className="flex gap-1">
                      <button
                        type="button"
                        className="rounded bg-neutral-700 px-2 text-[11px]"
                        onClick={() => copy(s.text)}
                      >
                        Copy
                      </button>
                      <button
                        type="button"
                        className="rounded bg-neutral-700 px-2 text-[11px]"
                        onClick={() =>
                          setLog((l) => l.filter((x) => x.t !== s.t))
                        }
                      >
                        ✕
                      </button>
                    </span>
                  </div>
                  <pre className="mt-1 whitespace-pre-wrap font-mono text-[11px] text-neutral-200">
                    {s.text}
                  </pre>
                </div>
              ))}
          </div>
        )}
      </section>
    </div>
  );
}

/** Sidebar controls for the rehearsal view. */
export interface RehearsalSettings extends HarmonyConfig {
  /** NoteDetector familyMarginDb: 0 = choir (default), 8 = solo / sectional. */
  soloMode: boolean;
}
export const DEFAULT_REHEARSAL_SETTINGS: RehearsalSettings = {
  a4: 440,
  profile: "ensemble",
  vibratoHeavy: false,
  sections: { B: true, T: true, A: true, S: true },
  soloMode: false,
};

export function RehearsalControls({
  settings,
  onChange,
  onResetDrift,
}: {
  settings: RehearsalSettings;
  onChange: (partial: Partial<RehearsalSettings>) => void;
  onResetDrift: () => void;
}) {
  const on = Object.values(settings.sections).filter(Boolean).length;
  return (
    <section className="space-y-3">
      <h2 className="text-xs uppercase tracking-wide text-neutral-400">
        Rehearsal
      </h2>
      <div>
        <div className="mb-1 text-[11px] text-neutral-400">
          Sections singing
        </div>
        <div className="flex gap-1">
          {SECTIONS.map((s) => {
            const active = settings.sections[s.id];
            return (
              <button
                key={s.id}
                type="button"
                onClick={() => {
                  if (active && on === 1) return;
                  onChange({
                    sections: { ...settings.sections, [s.id]: !active },
                  });
                }}
                className="flex-1 rounded border py-1 text-xs font-bold"
                style={{
                  background: active ? s.hex : "transparent",
                  color: active ? "#0f172a" : "#64748b",
                  borderColor: active ? s.hex : "#334155",
                }}
              >
                {s.id}
              </button>
            );
          })}
        </div>
      </div>
      <label className="block">
        <span className="text-[11px] text-neutral-400">Target tuning</span>
        <select
          value={settings.profile}
          onChange={(e) =>
            onChange({ profile: e.target.value as HarmonyConfig["profile"] })
          }
          className="mt-1 w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
        >
          <option value="pure">Pure (just) · M3 386</option>
          <option value="ensemble">Ensemble · M3 394</option>
          <option value="equal">Equal (12-TET) · M3 400</option>
        </select>
      </label>
      <label className="block">
        <span className="text-[11px] text-neutral-400">Reference A4</span>
        <select
          value={settings.a4}
          onChange={(e) => onChange({ a4: Number(e.target.value) })}
          className="mt-1 w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs"
        >
          {[440, 442, 443, 432, 415].map((a) => (
            <option key={a} value={a}>
              {a} Hz
            </option>
          ))}
        </select>
      </label>
      <label className="flex items-center gap-2 text-xs text-neutral-300">
        <input
          type="checkbox"
          checked={settings.vibratoHeavy}
          onChange={(e) => onChange({ vibratoHeavy: e.target.checked })}
        />
        Vibrato-heavy ensemble (widen flat side)
      </label>
      <label className="flex items-center gap-2 text-xs text-neutral-300">
        <input
          type="checkbox"
          checked={settings.soloMode}
          onChange={(e) => onChange({ soloMode: e.target.checked })}
        />
        Solo / sectional mode (one singer per part)
      </label>
      <p className="text-[10px] leading-snug text-neutral-500">
        Solo mode groups a voice's own harmonics into one card (a chest voice's
        H2/H3 can otherwise register as tenor/alto). Off, octave doublings
        between sections are kept — see DETECTOR_CONFIG.familyMarginDb for the
        measured trade-off.
      </p>
      <button
        type="button"
        onClick={onResetDrift}
        className="w-full rounded bg-neutral-800 px-3 py-1.5 text-xs font-medium text-neutral-200 hover:bg-neutral-700"
      >
        Reset drift reference
      </button>
    </section>
  );
}
