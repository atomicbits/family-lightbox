import { useState } from "react";
import { defaultYear, groupLists, type ListInfo, type Lists } from "../lib/lists";
import PhotoMenu from "./PhotoMenu";
import ConfirmDialog from "./ConfirmDialog";
import { t } from "../i18n";

type Props = {
  lists: Lists;
  month: string | null;
  /** Something from the grid is being dragged; a list takes it on drop (F003/S05). */
  dragging: boolean;
  onDrop: (id: string) => void;
  /** Fetch every original of this list that isn't on this Mac (F009/S03); for Lightroom, saying
   * afterwards to update the lists there (F023/S03). */
  onFetch: (id: string, forLightroom: boolean) => void;
  /** Whether a fetch runs already. */
  fetching: boolean;
  /** Export this list (F024/S03). */
  onExport: (list: ListInfo) => void;
};

const button = "px-1.5 rounded hover:bg-neutral-200/60 dark:hover:bg-neutral-800 disabled:opacity-40";
const input = "w-full px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-950";

/** The lists in the sidebar, per year (F003/S03): click to show one in the grid; + makes a new one. */
export default function ListsPanel({ lists, month, dragging, onDrop, onFetch, fetching, onExport }: Props) {
  const [over, setOver] = useState<string | null>(null);
  const { state, active, blocked } = lists;
  const [creating, setCreating] = useState<{ name: string; year: number } | null>(null);
  // the list's menu, by right-click or ⋯ (F003/S03, F009/S03); archiving asks first
  const [menu, setMenu] = useState<{ list: ListInfo; x: number; y: number } | null>(null);
  const [archiving, setArchiving] = useState<ListInfo | null>(null);
  const [renaming, setRenaming] = useState<{ id: string; name: string } | null>(null);
  const [showArchived, setShowArchived] = useState(false);
  const groups = groupLists(state?.lists ?? [], showArchived);
  const anyArchived = state?.lists.some((l) => l.archived) ?? false;
  const w = t().lists.panel;

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
          onContextMenu={(e) => { e.preventDefault(); setMenu({ list: l, x: e.clientX, y: e.clientY }); }}
          onDragOver={(e) => { if (dragging && !l.archived) { e.preventDefault(); e.dataTransfer.dropEffect = "copy"; setOver(l.id); } }}
          onDragLeave={() => setOver((o) => (o === l.id ? null : o))}
          onDrop={(e) => { e.preventDefault(); setOver(null); onDrop(l.id); }}
          title={isActive ? w.backToAll : w.showList}
        >
          <span className={"truncate flex-1 " + (l.archived ? "italic opacity-70" : "")}>{l.name}</span>
          <span className={"text-xs tabular-nums mr-1 " + (isActive ? "text-blue-100" : "text-neutral-400")}>{l.count}</span>
          <button
            className={"px-1 rounded opacity-0 group-hover:opacity-100 " + (isActive ? "hover:bg-blue-500" : "hover:bg-neutral-300 dark:hover:bg-neutral-700")}
            title={w.menuTip}
            onClick={(e) => { e.stopPropagation(); const r = e.currentTarget.getBoundingClientRect(); setMenu({ list: l, x: r.left, y: r.bottom }); }}
          >
            ⋯
          </button>
        </div>
      </div>
    );
  };

  const listMenu = menu && (
    <PhotoMenu
      x={menu.x}
      y={menu.y}
      what={menu.list.name}
      lists={[]}
      blocked={null}
      onAddTo={() => {}}
      onClose={() => setMenu(null)}
      actions={[
        {
          label: w.fetchAll,
          disabled: fetching,
          title: fetching ? w.fetchingAlready : w.fetchAllTip,
          run: () => { setMenu(null); onFetch(menu.list.id, false); },
        },
        {
          label: w.toLightroom,
          disabled: fetching,
          title: w.toLightroomTip,
          run: () => { setMenu(null); onFetch(menu.list.id, true); },
        },
        { label: w.export, title: w.exportTip, run: () => { setMenu(null); onExport(menu.list); } },
        { label: w.rename, disabled: !!blocked, title: blocked ?? undefined, run: () => { setMenu(null); setRenaming({ id: menu.list.id, name: menu.list.name }); } },
        menu.list.archived
          ? { label: w.restore, disabled: !!blocked, title: blocked ?? undefined, run: () => { setMenu(null); lists.archive(menu.list.id, false); } }
          : { label: w.archive, disabled: !!blocked, title: blocked ?? w.archiveTip, run: () => { setMenu(null); setArchiving(menu.list); } },
      ]}
    />
  );

  return (
    <section className="border-t border-r border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 text-sm flex flex-col min-h-0">
      {listMenu}
      {archiving && (
        <ConfirmDialog
          title={w.archiveTitle(archiving.name)}
          text={w.archiveText}
          yes={w.archiveYes}
          onNo={() => setArchiving(null)}
          onYes={() => { const id = archiving.id; setArchiving(null); lists.archive(id, true); }}
        />
      )}
      <div className="flex items-center px-3 py-1.5 font-semibold text-neutral-700 dark:text-neutral-200">
        <span className="flex-1">{w.title}</span>
        <button
          className={button}
          disabled={!!blocked}
          title={blocked ?? w.newList}
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
              placeholder={w.namePlaceholder}
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
        {state && state.lists.length === 0 && !creating && !state.problem && (
          <div className="px-3 pb-2 text-xs text-neutral-400">{blocked ?? w.empty}</div>
        )}
        {anyArchived && (
          <label className="flex items-center gap-1.5 px-3 py-1 text-xs text-neutral-400">
            <input type="checkbox" checked={showArchived} onChange={(e) => setShowArchived(e.target.checked)} />
            {w.showArchived}
          </label>
        )}
        {state?.problem && (
          <div className="px-3 pb-2 text-xs text-amber-600" title={state.problem}>
            {w.unreadable}
          </div>
        )}
        {state && state.source === "local" && state.lists.length > 0 && (
          <div className="px-3 pb-2 text-xs text-neutral-400">{w.withoutNas}</div>
        )}
      </div>
    </section>
  );
}
