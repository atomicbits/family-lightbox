import type { Sync } from "../lib/sync";

type Props = { sync: Sync; onConfirm: () => void };

/** The background sync next to the NAS indicator: progress with ✕, the confirmation, or what came in. */
export default function SyncLine({ sync, onConfirm }: Props) {
  const l = sync.line;
  if (!l) return null;
  const tone = l.tone === "amber" ? "text-amber-600" : "text-neutral-500";
  return (
    <span className={`flex items-center gap-1.5 ${tone}`} title={l.hint}>
      {l.confirm ? (
        <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={onConfirm}>
          {l.text}
        </button>
      ) : (
        <span className="tabular-nums">{l.text}</span>
      )}
      {l.hint && <span className="text-xs text-neutral-400">({l.hint})</span>}
      {l.cancellable && (
        <button className="px-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" title="synchroniseren stoppen" onClick={() => sync.cancel()}>
          ✕
        </button>
      )}
    </span>
  );
}
