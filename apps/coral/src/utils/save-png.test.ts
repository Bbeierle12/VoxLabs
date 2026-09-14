import { describe, expect, it, vi } from 'vitest';
import {
  isTauriRuntime,
  pngFileName,
  pngTimestamp,
  savePng,
  type SavePngDeps,
} from './save-png';

const PNG_MAGIC = [0x89, 0x50, 0x4e, 0x47];

function makeDeps(overrides: Partial<SavePngDeps> = {}): SavePngDeps {
  return {
    isTauri: () => true,
    saveDialog: vi.fn(async () => '/tmp/out.png'),
    writeFile: vi.fn(async () => undefined),
    downloadViaAnchor: vi.fn(),
    ...overrides,
  };
}

describe('savePng', () => {
  const blob = new Blob([new Uint8Array(PNG_MAGIC)], { type: 'image/png' });

  it('in a browser hands the blob to the anchor download and never touches Tauri', async () => {
    const deps = makeDeps({ isTauri: () => false });
    await expect(savePng(blob, 'coral-x.png', deps)).resolves.toBe('saved');
    expect(deps.downloadViaAnchor).toHaveBeenCalledWith(blob, 'coral-x.png');
    expect(deps.saveDialog).not.toHaveBeenCalled();
    expect(deps.writeFile).not.toHaveBeenCalled();
  });

  it('in Tauri asks for a destination with a PNG filter, then writes the exact bytes there', async () => {
    const deps = makeDeps({ saveDialog: vi.fn(async () => 'content://docs/42') });
    await expect(savePng(blob, 'coral-x.png', deps)).resolves.toBe('saved');
    expect(deps.saveDialog).toHaveBeenCalledWith({
      defaultPath: 'coral-x.png',
      filters: [{ name: 'PNG image', extensions: ['png'] }],
    });
    const writes = vi.mocked(deps.writeFile).mock.calls;
    expect(writes).toHaveLength(1);
    const [path, bytes] = writes[0] as [string, Uint8Array];
    expect(path).toBe('content://docs/42');
    expect(Array.from(bytes)).toEqual(PNG_MAGIC);
    expect(deps.downloadViaAnchor).not.toHaveBeenCalled();
  });

  it('in Tauri reports a dismissed save sheet as cancelled without writing', async () => {
    const deps = makeDeps({ saveDialog: vi.fn(async () => null) });
    await expect(savePng(blob, 'coral-x.png', deps)).resolves.toBe('cancelled');
    expect(deps.writeFile).not.toHaveBeenCalled();
  });

  it('propagates a failed write so the UI can surface it', async () => {
    const deps = makeDeps({
      writeFile: vi.fn(async () => {
        throw new Error('forbidden path');
      }),
    });
    await expect(savePng(blob, 'coral-x.png', deps)).rejects.toThrow('forbidden path');
  });
});

describe('file naming', () => {
  it('stamps local time as coral-YYYYMMDD-HHmmss.png', () => {
    const at = new Date(2026, 8, 2, 4, 5, 6); // 2 Sep 2026 04:05:06 local
    expect(pngTimestamp(at)).toBe('20260902-040506');
    expect(pngFileName(at)).toBe('coral-20260902-040506.png');
  });
});

describe('isTauriRuntime', () => {
  it('is false without a window and true once __TAURI_INTERNALS__ is present', () => {
    const g = globalThis as { window?: unknown };
    const saved = g.window;
    try {
      delete g.window;
      expect(isTauriRuntime()).toBe(false);
      g.window = {};
      expect(isTauriRuntime()).toBe(false);
      g.window = { __TAURI_INTERNALS__: {} };
      expect(isTauriRuntime()).toBe(true);
    } finally {
      if (saved === undefined) delete g.window;
      else g.window = saved;
    }
  });
});
