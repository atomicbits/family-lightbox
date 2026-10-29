import { useEffect } from "react";
import type { ListInfo } from "../lib/lists";

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
};

/** The right-click menu on the grid (F003/S05): the active list, and adding to any list. */
export default function PhotoMenu({ x, y, what, actions, lists, blocked, onAddTo, onClose }: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const item = "w-full text-left px-3 py-1 rounded hover:bg-blue-600 hover:text-white disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-inherit";
  // keep the menu inside the window
  const left = Math.min(x, window.innerWidth - 260);
  const top = Math.min(y, window.innerHeight - 40 - 28 * (actions.length + lists.length + 1));

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
        {lists.length > 0 && (
          <>
            <div className="my-1 border-t border-neutral-200 dark:border-neutral-700" />
            <div className="px-3 py-1 text-xs text-neutral-400">Toevoegen aan</div>
            {lists.map((l) => (
              <button key={l.id} className={item} disabled={!!blocked} title={blocked ?? undefined} onClick={() => { onAddTo(l.id); onClose(); }}>
                {l.name} <span className="opacity-60">{l.year}</span>
              </button>
            ))}
          </>
        )}
      </div>
    </div>
  );
}
