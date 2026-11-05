import { fetchPlanText, type FetchPlan } from "../lib/fetcher";

type Props = { plan: FetchPlan; onYes: () => void; onNo: () => void };

/** Before fetching (F009/S03): what it takes, and whether it fits. */
export default function FetchConfirm({ plan, onYes, onNo }: Props) {
  const nothing = plan.files.length === 0;
  const tight = plan.fits === false;
  return (
    <div className="absolute inset-0 z-40 bg-black/40 flex items-center justify-center p-4" onClick={onNo}>
      <div className="w-[28rem] max-w-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-3" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">Originelen ophalen</h2>
        <p>{fetchPlanText(plan)}</p>
        <p className="text-neutral-500">
          Ze komen in je werkkopie, op hun eigen plaats in de bibliotheek, zodat ook Lightroom ze vindt. Je mag dit venster sluiten en de app herstarten:
          het ophalen gaat verder, de voortgang staat onderaan.
        </p>
        {tight && <p className="text-amber-600">Er is niet genoeg plaats op deze Mac (20 GB blijft altijd vrij).</p>}
        <div className="flex justify-end gap-2">
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onNo}>Annuleren</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40" disabled={nothing || tight} onClick={onYes}>
            Ophalen
          </button>
        </div>
      </div>
    </div>
  );
}
