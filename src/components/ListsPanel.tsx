import { useState } from "react";
import { defaultYear, groupLists, type ListInfo, type Lists } from "../lib/lists";

type Props = {
  lists: Lists;
  month: string | null;
  /** Something from the grid is being dragged; a list takes it on drop (F003/S05). */
  dragging: boolean;
  onDrop: (id: string) => void;
};

const button = "px-1.5 rounded hover:bg-neutral-200/60 dark:hover:bg-neutral-800 disabled:opacity-40";
const input = "w-full px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-950";

/** The lists in the sidebar, per year (F003/S03): click to show one in the grid; + makes a new one. */
export default function ListsPanel({ lists, month, dragging, onDrop }: Props) {
  const [over, setOver] = useState<string | null>(null);
  const { state, active, blocked } = lists;
  const [creating, setCreating] = useState<{ name: string; year: number } | null>(null);
  const [menu, setMenu] = useState<string | null>(null);
  const [renaming, setRenaming] = useState<{ id: string; name: string } | null>(null);
  const [showArchived, setShowArchived] = useState(false);
  const groups = groupLists(state?.lists ?? [], showArchived);
  const anyArchived = state?.lists.some((l) => l.archived) ?? false;

  const submitCreate = async () => {
    if (creating && (await lists.create(creating.name, creating.year))) setCreating(null);
  };
  const submitRename = async () => {
    if (renaming && (await lists.rename(renaming.id, renaming.name))) setRenaming(null);
  };

  const row = (l: ListInfo) => {
    const isActive = active?.id === l.id;
    if (renaming?.id === l.id) {
      return (
        <div key={l.id} className="pl-5 pr-2 py-0.5">
          <input
            autoFocus
            className={input}
            value={renaming.name}
            onChange={(e) => setRenaming({ ...renaming, name: e.target.value })}
            onKeyDown={(e) => {
              if (e.key === "Enter") submitRename();
              if (e.key === "Escape") setRenaming(null);
            }}
          />
        </div>
      );
    }
    return (
      <div key={l.id}>
        <div
          className={
            "group flex items-center pl-5 pr-1 py-0.5 cursor-default " +
            (over === l.id
              ? "bg-blue-200 dark:bg-blue-900 ring-2 ring-blue-500"
              : isActive ? "bg-blue-600 text-white" : "text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200/60 dark:hover:bg-neutral-800")
          }
          onClick={() => lists.setActive(isActive ? null : l.id)}
          onDragOver={(e) => { if (dragging && !l.archived) { e.preventDefault(); e.dataTransfer.dropEffect = "copy"; setOver(l.id); } }}
          onDragLeave={() => setOver((o) => (o === l.id ? null : o))}
          onDrop={(e) => { e.preventDefault(); setOver(null); onDrop(l.id); }}
          title={isActive ? "terug naar alle foto's" : "toon deze lijst; sleep foto's hierheen om ze toe te voegen"}
        >
          <span className={"truncate flex-1 " + (l.archived ? "italic opacity-70" : "")}>{l.name}</span>
          <span className={"text-xs tabular-nums mr-1 " + (isActive ? "text-blue-100" : "text-neutral-400")}>{l.count}</span>
          <button
            className={"px-1 rounded opacity-0 group-hover:opacity-100 " + (isActive ? "hover:bg-blue-500" : "hover:bg-neutral-300 dark:hover:bg-neutral-700")}
            title="hernoemen of archiveren"
            onClick={(e) => { e.stopPropagation(); setMenu(menu === l.id ? null : l.id); }}
          >
            ⋯
          </button>
        </div>
        {menu === l.id && (
          <div className="pl-5 pr-2 py-1 flex gap-1 text-xs">
            <button className={button} disabled={!!blocked} title={blocked ?? undefined} onClick={() => { setMenu(null); setRenaming({ id: l.id, name: l.name }); }}>
              Hernoemen
            </button>
            <button className={button} disabled={!!blocked} title={blocked ?? undefined} onClick={() => { setMenu(null); lists.archive(l.id, !l.archived); }}>
              {l.archived ? "Terugzetten" : "Archiveren"}
            </button>
          </div>
        )}
      </div>
    );
  };

  return (
    <section className="border-t border-r border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 text-sm flex flex-col min-h-0">
      <div className="flex items-center px-3 py-1.5 font-semibold text-neutral-700 dark:text-neutral-200">
        <span className="flex-1">Lijsten</span>
        <button
          className={button}
          disabled={!!blocked}
          title={blocked ?? "nieuwe lijst"}
          onClick={() => setCreating({ name: "", year: defaultYear(month) })}
        >
          +
        </button>
      </div>
      <div className="overflow-y-auto min-h-0">
        {creating && (
          <div className="px-3 pb-2 flex gap-1">
            <input
              autoFocus
              className={input}
              placeholder="naam"
              value={creating.name}
              onChange={(e) => setCreating({ ...creating, name: e.target.value })}
              onKeyDown={(e) => {
                if (e.key === "Enter") submitCreate();
                if (e.key === "Escape") setCreating(null);
              }}
            />
            <input
              className={input + " w-16"}
              type="number"
              value={creating.year}
              onChange={(e) => setCreating({ ...creating, year: Number(e.target.value) })}
              onKeyDown={(e) => {
                if (e.key === "Enter") submitCreate();
                if (e.key === "Escape") setCreating(null);
              }}
            />
          </div>
        )}
        {groups.map((g) => (
          <div key={g.year}>
            <div className="px-3 pt-1 text-xs font-semibold text-neutral-400">{g.year}</div>
            {g.lists.map(row)}
          </div>
        ))}
        {state && state.lists.length === 0 && !creating && (
          <div className="px-3 pb-2 text-xs text-neutral-400">{blocked && state.source !== "nas" ? blocked : "nog geen lijsten: + maakt er een"}</div>
        )}
        {anyArchived && (
          <label className="flex items-center gap-1.5 px-3 py-1 text-xs text-neutral-400">
            <input type="checkbox" checked={showArchived} onChange={(e) => setShowArchived(e.target.checked)} />
            Gearchiveerd tonen
          </label>
        )}
        {state?.problem && <div className="px-3 pb-2 text-xs text-amber-600">{state.problem}</div>}
        {state && state.source === "local" && state.lists.length > 0 && (
          <div className="px-3 pb-2 text-xs text-neutral-400">zonder NAS: alleen bekijken</div>
        )}
      </div>
    </section>
  );
}
