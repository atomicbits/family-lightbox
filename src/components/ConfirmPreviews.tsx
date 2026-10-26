import { confirmText, type PreviewsSummary } from "../lib/sync";

type Props = { summary: PreviewsSummary; onYes: () => void; onLater: () => void };

/** Asked once, before the first full previews pass on this Mac (F007/S04, S05). */
export default function ConfirmPreviews({ summary, onYes, onLater }: Props) {
  return (
    <div className="absolute inset-0 z-30 bg-black/40 grid place-items-center" onClick={onLater}>
      <div className="w-[26rem] rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">Previews ophalen</h2>
        <p>{confirmText(summary)}</p>
        <p className="text-neutral-500">
          Ze komen in je map Previews op deze Mac. Je kunt intussen gewoon verder bladeren, en stoppen kan altijd met ✕;
          de volgende keer gaat het verder waar het stopte.
        </p>
        <div className="flex justify-end gap-2">
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onLater}>Later</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900" onClick={onYes}>Ophalen</button>
        </div>
      </div>
    </div>
  );
}
