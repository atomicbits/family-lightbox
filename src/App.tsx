import { useCallback, useEffect, useState } from "react";
import { api, type Item, type MonthSummary, type Status } from "./lib/api";
import { monthLabel, yearOf } from "./lib/months";
import MonthList from "./components/MonthList";
import Grid from "./components/Grid";
import Lightbox from "./components/Lightbox";

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [months, setMonths] = useState<MonthSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [month, setMonth] = useState<string | null>(null);
  const [items, setItems] = useState<Item[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    api.status().then(setStatus);
    api
      .months()
      .then((m) => {
        setMonths(m);
        if (m.length && !month) setMonth(m[0].month);
      })
      .catch((e) => setError(String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!month) return;
    setSelected(null);
    setOpen(false);
    api.items(month).then(setItems).catch((e) => setError(String(e)));
  }, [month]);

  const step = useCallback(
    (delta: number) =>
      setSelected((s) => {
        if (items.length === 0) return null;
        const n = (s ?? -1) + delta;
        return Math.min(items.length - 1, Math.max(0, n));
      }),
    [items.length],
  );

  // toetsenbord op het raster: pijltjes, spatie/enter opent, Esc sluit
  useEffect(() => {
    if (open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight") step(1);
      else if (e.key === "ArrowLeft") step(-1);
      else if ((e.key === " " || e.key === "Enter") && selected !== null) { e.preventDefault(); setOpen(true); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, selected, step]);

  const current = selected !== null ? items[selected] : undefined;

  return (
    <div className="h-full flex flex-col bg-white dark:bg-neutral-950 text-neutral-900 dark:text-neutral-100">
      <header className="flex items-center gap-3 px-4 h-10 border-b border-neutral-200 dark:border-neutral-800 text-sm">
        <span className="font-semibold">
          {month ? `${monthLabel(month)} ${yearOf(month)}` : "FamilyLightbox"}
        </span>
        {month && (
          <span className="text-neutral-400">
            {items.length} {items.length === 1 ? "item" : "items"}
            {current && ` · ${selected! + 1} gekozen`}
          </span>
        )}
        <span className="flex-1" />
        {status && !status.index_exists && (
          <span className="text-amber-600">geen index: draai ./media index</span>
        )}
      </header>
      {error ? (
        <div className="flex-1 grid place-items-center p-8 text-center whitespace-pre-wrap text-neutral-500">
          {error}
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex relative">
          <div className="w-56 shrink-0">
            <MonthList months={months} selected={month} onSelect={setMonth} />
          </div>
          <main className="flex-1 min-w-0">
            <Grid
              items={items}
              selected={selected}
              onSelect={setSelected}
              onOpen={(i) => { setSelected(i); setOpen(true); }}
            />
          </main>
          {open && current && (
            <Lightbox
              item={current}
              index={selected!}
              count={items.length}
              onClose={() => setOpen(false)}
              onStep={step}
            />
          )}
        </div>
      )}
    </div>
  );
}
