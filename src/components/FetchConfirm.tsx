import { BIG_FETCH, fetchPlanText, type FetchPlan } from "../lib/fetcher";
import { estimate, gb } from "../lib/sync";
import { t } from "../i18n";

type Props = { plan: FetchPlan; onYes: () => void; onNo: () => void };

/** Before fetching (F009/S03): what it takes, and whether it fits. */
export default function FetchConfirm({ plan, onYes, onNo }: Props) {
  const nothing = plan.files.length === 0;
  const tight = plan.fits === false;
  const big = plan.bytes > BIG_FETCH;
  const w = t().lists.fetch;
  return (
    <div className="absolute inset-0 z-40 bg-black/40 flex items-center justify-center p-4" onClick={onNo}>
      <div className="w-[28rem] max-w-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-3" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">{w.title}</h2>
        <p>{fetchPlanText(plan)}</p>
        <p className="text-neutral-500">{w.where}</p>
        {tight && <p className="text-amber-600">{w.tight}</p>}
        {big && !tight && (
          <p className="text-amber-600">{w.big(gb(plan.bytes), estimate(plan.bytes))}</p>
        )}
        <div className="flex justify-end gap-2">
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onNo}>{w.cancel}</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40" disabled={nothing || tight} onClick={onYes}>
            {big ? w.anyway(gb(plan.bytes)) : w.yes}
          </button>
        </div>
      </div>
    </div>
  );
}
