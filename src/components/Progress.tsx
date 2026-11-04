import { fraction } from "../lib/progress";
import { percent } from "../lib/rename";

/** A turning circle, for work under way whose end isn't known yet. */
export function Spinner() {
  return <span aria-hidden className="inline-block w-3 h-3 shrink-0 rounded-full border-2 border-current border-r-transparent animate-spin" />;
}

type Props = { text: string; done?: number; total?: number; tone?: string };

/** Work under way (F018/S05): a spinner and its text, with a bar that fills when the amount is
 * known, or moves to and fro when it isn't yet. */
export default function Progress({ text, done, total, tone = "text-blue-600" }: Props) {
  const f = fraction(done, total);
  return (
    <div className={`space-y-1 text-xs ${tone}`}>
      <div className="flex items-center gap-2">
        <Spinner />
        <span className="tabular-nums flex-1">{text}</span>
        {f !== null && <span className="tabular-nums">{percent(f)}</span>}
      </div>
      <div className="h-1 rounded bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
        {f === null ? (
          <div className="h-full w-1/3 rounded bg-current animate-[slide_1.2s_ease-in-out_infinite_alternate]" />
        ) : (
          <div className="h-full rounded bg-current transition-[width] duration-300" style={{ width: `${Math.max(2, f * 100)}%` }} />
        )}
      </div>
    </div>
  );
}

/** The footer's small bar with its percentage (F018/S05, F022): moving while the amount isn't
 * known yet. */
export function MiniBar({ done }: { done: number | null }) {
  return (
    <>
      <span className="w-12 h-1 rounded bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
        {done === null ? (
          <span className="block h-full w-1/3 rounded bg-current animate-[slide_1.2s_ease-in-out_infinite_alternate]" />
        ) : (
          <span className="block h-full rounded bg-current transition-[width] duration-300" style={{ width: `${Math.max(4, done * 100)}%` }} />
        )}
      </span>
      {done !== null && <span className="tabular-nums">{percent(done)}</span>}
    </>
  );
}
