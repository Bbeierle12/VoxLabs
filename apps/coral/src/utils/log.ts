/**
 * Structured logging. Emits JSON to console for ingestion by any log pipe.
 * No bare console.log anywhere else in the codebase.
 */

type LogData = Record<string, unknown>;

function emit(level: 'info' | 'warn' | 'error', msg: string, data?: LogData): void {
  const payload = JSON.stringify({ level, msg, ts: Date.now(), ...data });
  if (level === 'error') console.error(payload);
  else if (level === 'warn') console.warn(payload);
  else console.log(payload);
}

export const log = {
  info: (msg: string, data?: LogData): void => emit('info', msg, data),
  warn: (msg: string, data?: LogData): void => emit('warn', msg, data),
  error: (msg: string, err?: Error, data?: LogData): void =>
    emit('error', msg, { ...data, error: err?.message, stack: err?.stack }),
};
