import type { Previews } from "../lib/previews";

/** "Previews maken: 1 204 / 85 874" with Pauzeren or Hervatten in the header (F012/S03). */
export default function PreviewsLine({ previews }: { previews: Previews }) {
  const l = previews.line;
  if (!l) return null;
  return (
    <span className={`flex items-center gap-1.5 ${l.tone === "amber" ? "text-amber-600" : "text-neutral-500"}`}>
      <span className="tabular-nums">{l.text}</span>
      {l.button && (
        <button
          className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700"
          onClick={() => (l.button?.action === "pause" ? previews.pause() : previews.resume()).catch(() => {})}
        >
          {l.button.label}
        </button>
      )}
    </span>
  );
}
