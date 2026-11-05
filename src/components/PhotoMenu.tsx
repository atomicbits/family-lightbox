import { useEffect, useState } from "react";
import { menuDirect, menuSearch, type ListInfo } from "../lib/lists";
import { t } from "../i18n";

export type MenuAction = { label: string; run: () => void; disabled?: boolean; title?: string };

type Props = {
  x: number;
  y: number;
  /** What the menu acts on: "1 foto" or "12 foto's". */
  what: string;
  actions: MenuAction[];
  /** "Toevoegen aan" one of these lists. */
  lists: ListInfo[];
  blocked: string | null;
  onAddTo: (id: string) => void;
  onClose: () => void;
  /** A row of colors to pick from (a stack's color, F015/S05). */
  swatches?: { colors: { name: string; border: string }[]; current: string; onPick: (name: string) => void };
};

/** The right-click menu on the grid (F003/S05): the active list, and adding to any list. */
export default function PhotoMenu({ x, y, what, actions, lists, blocked, onAddTo, onClose, swatches }: Props) {
  // with many lists: the most recent directly, the rest behind *Andere lijst…* with a search (F003/S05)
  const { direct, more } = menuDirect(lists);
  const [searching, setSearching] = useState(false);
  const [query, setQuery] = useState("");
  const found = searching ? menuSearch(lists, query) : [];
  const firstFound = found[0]?.lists[0];
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const item = "w-full text-left px-3 py-1 rounded hover:bg-blue-600 hover:text-white disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-inherit";
  // keep the menu inside the window
  const left = Math.min(x, window.innerWidth - 260);
  const top = Math.min(y, window.innerHeight - 40 - 28 * (actions.length + direct.length + (more ? 2 : 1)) - (searching ? 300 : 0));

  return (
    <div className="absolute inset-0 z-40" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose(); }}>
      <div
        className="absolute w-60 max-h-[70vh] overflow-y-auto rounded-md shadow-xl border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-sm p-1"
        style={{ left, top: Math.max(8, top) }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="px-3 py-1 text-xs text-neutral-400">{what}</div>
        {actions.map((a) => (
          <button key={a.label} className={item} disabled={a.disabled} title={a.title} onClick={() => { a.run(); onClose(); }}>
            {a.label}
          </button>
        ))}
        {swatches && (
          <div className="flex items-center gap-1.5 px-3 py-1.5" title={blocked ?? undefined}>
            <span className="text-xs text-neutral-400 mr-1">{t().browse.menu.color}</span>
            {swatches.colors.map((c) => (
              <button
                key={c.name}
                disabled={!!blocked}
                title={t().browse.colors[c.name] ?? c.name}
                className={"w-4 h-4 rounded-full disabled:opacity-40 " + (c.name === swatches.current ? "ring-2 ring-offset-1 ring-neutral-500 dark:ring-offset-neutral-900" : "")}
                style={{ background: c.border }}
                onClick={() => { swatches.onPick(c.name); onClose(); }}
              />
            ))}
          </div>
        )}
        {lists.length > 0 && (
          <>
            <div className="my-1 border-t border-neutral-200 dark:border-neutral-700" />
            <div className="px-3 py-1 text-xs text-neutral-400">{more ? t().browse.menu.addToRecent : t().browse.menu.addTo}</div>
            {direct.map((l) => (
              <button key={l.id} className={item} disabled={!!blocked} title={blocked ?? undefined} onClick={() => { onAddTo(l.id); onClose(); }}>
                {l.name} <span className="opacity-60">{l.year}</span>
              </button>
            ))}
            {more && !searching && (
              <button className={item} disabled={!!blocked} title={blocked ?? t().browse.menu.otherListTitle} onClick={() => setSearching(true)}>
                {t().browse.menu.otherList}
              </button>
            )}
            {searching && (
              <div className="px-2 py-1 space-y-1">
                <input
                  autoFocus
                  className="w-full px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-950 text-sm"
                  placeholder={t().browse.menu.search}
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && firstFound) { onAddTo(firstFound.id); onClose(); }
                    if (e.key === "Escape") { e.stopPropagation(); setSearching(false); setQuery(""); }
                  }}
                />
                <div className="max-h-64 overflow-y-auto">
                  {found.length === 0 && <div className="px-1 py-1 text-xs text-neutral-400">{t().browse.menu.noList}</div>}
                  {found.map((g) => (
                    <div key={g.year}>
                      <div className="px-1 pt-1 text-xs text-neutral-400">{g.year}</div>
                      {g.lists.map((l) => (
                        <button key={l.id} className={item} disabled={!!blocked} onClick={() => { onAddTo(l.id); onClose(); }}>
                          {l.name}
                        </button>
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
