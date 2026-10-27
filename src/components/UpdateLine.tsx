import type { Update } from "../lib/update";
import { updateLine } from "../lib/update";

const TONE = { muted: "text-neutral-500", amber: "text-amber-600", red: "text-red-600" };

/** "Versie X.Y.Z beschikbaar" with *Bijwerken* in the header (F004/S04). */
export default function UpdateLine({ update, syncing }: { update: Update; syncing: boolean }) {
  const line = updateLine(update.state, syncing);
  if (!line) return null;
  return (
    <span className="flex items-center gap-2">
      <span className={TONE[line.tone]}>{line.text}</span>
      {line.button && (
        <button
          className="px-2 py-0.5 rounded bg-amber-100 hover:bg-amber-200 dark:bg-amber-900/40 dark:hover:bg-amber-900/70 disabled:opacity-40"
          disabled={line.button.disabled}
          title={line.button.title}
          onClick={() => update.install()}
        >
          {line.button.label}
        </button>
      )}
    </span>
  );
}
