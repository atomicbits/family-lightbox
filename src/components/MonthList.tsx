import { useState } from "react";
import type { MonthSummary } from "../lib/api";
import { groupByYear, monthLabel } from "../lib/months";

type Props = {
  months: MonthSummary[];
  selected: string | null;
  onSelect: (month: string) => void;
};

/** Linkerkolom: jaren met hun maanden en tellingen. Een jaar klapt open of dicht. */
export default function MonthList({ months, selected, onSelect }: Props) {
  const groups = groupByYear(months);
  const [closed, setClosed] = useState<Set<string>>(new Set());
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
            className="w-full flex items-center justify-between px-3 py-1.5 font-semibold text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200/60 dark:hover:bg-neutral-800"
          >
            <span>
              <span className="inline-block w-4 text-neutral-400">{closed.has(g.year) ? "▸" : "▾"}</span>
              {g.year}
            </span>
            <span className="text-xs font-normal text-neutral-400 tabular-nums">{g.photos + g.videos}</span>
          </button>
          {!closed.has(g.year) &&
            g.months.map((m) => {
              const active = m.month === selected;
              return (
                <button
                  key={m.month}
                  onClick={() => onSelect(m.month)}
                  className={
                    "w-full flex items-center justify-between pl-9 pr-3 py-1 text-left " +
                    (active
                      ? "bg-blue-600 text-white"
                      : "text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200/60 dark:hover:bg-neutral-800")
                  }
                  title={`${m.photos} foto's, ${m.videos} video's; ${m.previews} previews, ${m.local} originelen lokaal`}
                >
                  <span className="truncate">{monthLabel(m.month)}</span>
                  <span className={"text-xs tabular-nums " + (active ? "text-blue-100" : "text-neutral-400")}>
                    {m.photos + m.videos}
                  </span>
                </button>
              );
            })}
        </div>
      ))}
    </nav>
  );
}
