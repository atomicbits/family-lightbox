import type { Step } from "../lib/firststart";

const MARK: Record<Step["state"], { sign: string; tone: string }> = {
  done: { sign: "✓", tone: "text-green-600" },
  busy: { sign: "●", tone: "text-blue-500 animate-pulse" },
  todo: { sign: "○", tone: "text-neutral-300 dark:text-neutral-600" },
  waiting: { sign: "●", tone: "text-amber-500" },
  problem: { sign: "!", tone: "text-amber-600 font-bold" },
};

/** A new Mac before its first index: the steps of the first sync and how far each got. */
export default function FirstStart({ steps }: { steps: Step[] }) {
  return (
    <div className="w-[26rem] max-w-full text-left space-y-3">
      <div>
        <div className="font-medium text-neutral-700 dark:text-neutral-200">De bibliotheek van de NAS ophalen</div>
        <div className="text-xs text-neutral-500">De eerste keer duurt dit even; de foto's verschijnen zodra de index klaar is.</div>
      </div>
      <ol className="space-y-2">
        {steps.map((s) => (
          <li key={s.label} className="flex gap-2">
            <span className={`w-4 text-center ${MARK[s.state].tone}`}>{MARK[s.state].sign}</span>
            <div className="flex-1 min-w-0">
              <div className={s.state === "todo" ? "text-neutral-400" : "text-neutral-700 dark:text-neutral-200"}>{s.label}</div>
              {s.detail && <div className={"text-xs " + (s.state === "problem" || s.state === "waiting" ? "text-amber-600" : "text-neutral-500")}>{s.detail}</div>}
              {s.fraction !== null && (
                <div className="mt-1 h-1 rounded bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
                  <div className="h-full bg-blue-500" style={{ width: `${Math.round(Math.min(1, s.fraction) * 100)}%` }} />
                </div>
              )}
            </div>
          </li>
        ))}
      </ol>
    </div>
  );
}
