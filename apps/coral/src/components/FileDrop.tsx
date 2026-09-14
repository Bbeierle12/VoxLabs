import { useCallback, useRef, useState } from 'react';

interface Props {
  onFile: (file: File) => void;
  disabled?: boolean;
}

/**
 * Drop-zone that accepts an audio file and emits it via onFile.
 * Click anywhere on the zone to open the native file picker.
 */
export function FileDrop({ onFile, disabled = false }: Props) {
  const [isOver, setIsOver] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const onDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    if (!disabled) setIsOver(true);
  }, [disabled]);

  const onDragLeave = useCallback(() => setIsOver(false), []);

  const onDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsOver(false);
    if (disabled) return;
    const file = e.dataTransfer.files[0];
    if (file) onFile(file);
  }, [disabled, onFile]);

  const onChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) onFile(file);
    e.target.value = ''; // allow re-selecting the same file
  }, [onFile]);

  const onClick = useCallback(() => {
    if (!disabled) inputRef.current?.click();
  }, [disabled]);

  return (
    <div
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onClick={onClick}
      className={[
        'cursor-pointer rounded-lg border-2 border-dashed px-6 py-12 text-center transition-colors',
        isOver
          ? 'border-emerald-400 bg-emerald-950/30'
          : 'border-neutral-600 bg-neutral-900/50 hover:border-neutral-400',
        disabled ? 'pointer-events-none opacity-40' : '',
      ].join(' ')}
    >
      <p className="text-neutral-200">
        Drop an audio file here, or click to choose
      </p>
      <p className="mt-1 text-xs text-neutral-500">
        WAV, MP3, FLAC, OGG — whatever your browser can decode
      </p>
      <input
        ref={inputRef}
        type="file"
        accept="audio/*"
        onChange={onChange}
        className="hidden"
      />
    </div>
  );
}
