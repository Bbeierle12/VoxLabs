import type { SaveDialogOptions } from '@tauri-apps/plugin-dialog';

/**
 * PNG export, in two flavours behind one call.
 *
 * Browser: hand the blob to the browser's own download flow through an
 * `<a download>` click. This is the web build's path and needs no plumbing.
 *
 * Tauri shell (desktop and Android): a WebView has no download handler, so the
 * anchor click is silently dropped — on Android nothing at all happens. Ask
 * the OS for a destination with the dialog plugin, then write the bytes with
 * the fs plugin. On Android the dialog returns a `content://` URI from the
 * system "Save as" sheet; the fs plugin accepts that as a path, and the dialog
 * plugin adds the chosen location to the fs scope at runtime, so the only
 * static permissions needed are `dialog:allow-save` and `fs:allow-write-file`
 * (src-tauri/capabilities/default.json). The plugin modules are imported
 * lazily so the web bundle never loads them.
 */

/** Prefix for exported file names: `coral-YYYYMMDD-HHmmss.png`. */
const PNG_FILE_PREFIX = 'coral';
/** Label the OS save sheet shows for the PNG type filter. */
const PNG_FILTER_NAME = 'PNG image';

export type SavePngResult = 'saved' | 'cancelled';

/** The seams `savePng` uses, injectable for tests. */
export interface SavePngDeps {
  isTauri: () => boolean;
  saveDialog: (options: SaveDialogOptions) => Promise<string | null>;
  writeFile: (path: string, data: Uint8Array) => Promise<void>;
  downloadViaAnchor: (blob: Blob, fileName: string) => void;
}

/** True inside a Tauri webview (desktop or mobile), false in a plain browser. */
export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** Browser download: blob URL on a synthetic anchor click. */
export function downloadViaAnchor(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}

const defaultDeps: SavePngDeps = {
  isTauri: isTauriRuntime,
  saveDialog: async (options) => (await import('@tauri-apps/plugin-dialog')).save(options),
  writeFile: async (path, data) => (await import('@tauri-apps/plugin-fs')).writeFile(path, data),
  downloadViaAnchor,
};

/** Local-time stamp for PNG filenames: YYYYMMDD-HHmmss. */
export function pngTimestamp(now: Date = new Date()): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return (
    `${now.getFullYear()}${p(now.getMonth() + 1)}${p(now.getDate())}` +
    `-${p(now.getHours())}${p(now.getMinutes())}${p(now.getSeconds())}`
  );
}

/** `coral-YYYYMMDD-HHmmss.png` for the current local time. */
export function pngFileName(now: Date = new Date()): string {
  return `${PNG_FILE_PREFIX}-${pngTimestamp(now)}.png`;
}

/** Encode a canvas as a PNG blob; rejects if the canvas cannot be encoded. */
export function canvasToPngBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob);
      else reject(new Error('canvasToPngBlob: canvas.toBlob produced no data'));
    }, 'image/png');
  });
}

/**
 * Save `blob` as `fileName`. Resolves `'saved'` once the browser has taken the
 * download or the Tauri file write completed, `'cancelled'` if the user
 * dismissed the OS save sheet. Rejects if the dialog or the write fails.
 */
export async function savePng(
  blob: Blob,
  fileName: string,
  deps: SavePngDeps = defaultDeps
): Promise<SavePngResult> {
  if (!deps.isTauri()) {
    deps.downloadViaAnchor(blob, fileName);
    return 'saved';
  }
  const path = await deps.saveDialog({
    defaultPath: fileName,
    filters: [{ name: PNG_FILTER_NAME, extensions: ['png'] }],
  });
  if (path === null) return 'cancelled';
  const bytes = new Uint8Array(await blob.arrayBuffer());
  await deps.writeFile(path, bytes);
  return 'saved';
}
