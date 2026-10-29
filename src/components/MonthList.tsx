import { useEffect, useRef, useState } from "react";
import type { MonthSummary } from "../lib/api";
import { groupByYear, monthLabel } from "../lib/months";
import { countTitle, shownCount } from "../lib/hidden";

type Props = {
  months: MonthSummary[];
  selected: string | null;
  onSelect: (month: string) => void;
  /** Right-click on a year or a month (F014/S04). */
  onMenu?: (kind: "year" | "month", key: string, x: number, y: number) => void;
};

/** Linkerkolom: jaren met hun maanden en tellingen. Een jaar klapt open of dicht. */
export default function MonthList({ months, selected, onSelect, onMenu }: Props) {
  const groups = groupByYear(months);
  const [closed, setClosed] = useState<Set<string>>(new Set());
  // The highlighted month follows the grid's scrolling; keep it visible in the list.
  const activeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => activeRef.current?.scrollIntoView({ block: "nearest" }), [selected]);
  const toggle = (year: string) =>
    setClosed((s) => {
      const n = new Set(s);
      n.has(year) ? n.delete(year) : n.add(year);
      return n;
    });

  return (
    <nav className="h-full overflow-y-auto border-r border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 text-sm">
      {groups.map((g) => (
        <div key={g.year}>
          <button
            onClick={() => toggle(g.year)}
            onContextMenu={(e) => { e.preventDefault(); onMenu?.("year", g.year, e.clientX, e.clientY); }}
            className="w-full flex items-center justify-between px-3 py-1.5 font-semibold text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200/60 dark:hover:bg-neutral-800"
          >
            <span>
              <span className="inline-block w-4 text-neutral-400">{closed.has(g.year) ? "▸" : "▾"}</span>
              {g.year}
              {g.months.some((m) => m.revealed && m.hidden) && <RevealedMark />}
            </span>
            <span className="text-xs font-normal text-neutral-400 tabular-nums" title={countTitle(g.photos + g.videos - g.hidden, g.hidden)}>
              {g.photos + g.videos - g.hidden}
            </span>
          </button>
          {!closed.has(g.year) &&
            g.months.map((m) => {
              const active = m.month === selected;
              return (
                <button
                  key={m.month}
                  ref={active ? activeRef : undefined}
                  onClick={() => onSelect(m.month)}
                  onContextMenu={(e) => { e.preventDefault(); onMenu?.("month", m.month, e.clientX, e.clientY); }}
                  className={
                    "w-full flex items-center justify-between pl-9 pr-3 py-1 text-left " +
                    (active
                      ? "bg-blue-600 text-white"
                      : "text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200/60 dark:hover:bg-neutral-800")
                  }
                  title={`${countTitle(shownCount(m), m.revealed ? 0 : m.hidden ?? 0, m.stacks ?? 0)}; ${m.previews} previews, ${m.local} originelen lokaal`}
                >
                  <span className="truncate">
                    {monthLabel(m.month)}
                    {m.revealed && !!m.hidden && <RevealedMark />}
                  </span>
                  <span className={"text-xs tabular-nums " + (active ? "text-blue-100" : "text-neutral-400")}>
                    {shownCount(m)}
                  </span>
                </button>
              );
            })}
        </div>
      ))}
    </nav>
  );
}

/** A month (or a year with such a month) whose hidden photos are shown for now. */
function RevealedMark() {
  return (
    <span className="ml-1.5 text-[10px] align-middle text-amber-500" title="verborgen foto's worden getoond (klik rechts om ze weer te verbergen)">
      ◉
    </span>
  );
}
