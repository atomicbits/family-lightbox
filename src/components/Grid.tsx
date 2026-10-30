import { useEffect, useLayoutEffect, useMemo, useRef, useState, type DragEvent, type MouseEvent, type ReactNode } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { Item } from "../lib/api";
import { previewUrl } from "../lib/api";
import { addedText, type Member } from "../lib/lists";
import { heartView, markSize, marksLines, starsText, type MarksState } from "../lib/marks";
import { cell, lastOfStack, stackColor, type StackColor } from "../lib/stacks";
import StackIcon from "./StackIcon";
import VideoIcon from "./VideoIcon";
import LiveIcon from "./LiveIcon";
import { liveTitle } from "../lib/live";
import { columnsFor, rowWidth } from "../lib/grid";
import { fileName, monthLabel, yearOf } from "../lib/months";
import { buildLayout, rowOf, sectionsInRange, type Section, type Sel } from "../lib/timeline";

type Props = {
  sections: Section[];
  /** A month's items, or undefined while it isn't loaded yet. */
  itemsOf: (month: string) => Item[] | undefined;
  selected: Sel | null;
  /** A click, with shift or ⌘ held for picking several (F003/S05). */
  onSelect: (sel: Sel, mods: { shift: boolean; meta: boolean }) => void;
  onOpen: (sel: Sel) => void;
  /** Reports the column count, so the arrow keys can move a row up or down. */
  onColumns?: (cols: number) => void;
  /** Sections with rows on screen or in the overscan, so their items can be fetched. */
  onNeed: (sections: number[]) => void;
  /** The section whose rows are at the top of the view. */
  onTopSection: (section: number) => void;
  /** Scroll a section's header to the top; the nonce makes a repeated click scroll again. */
  jump: { section: number; nonce: number } | null;
  /** Tile size in CSS pixels (the size slider). */
  tile: number;
  /** In a list: its photos by hash, so a tile's tooltip says who added it (F003/S04). */
  marked?: Map<string, Member> | null;
  /** Tiles picked besides the selected one, as `section:index`. */
  picked?: Set<string>;
  /** Stars and hearts by hash, and this Mac's owner (F013/S02). */
  marks?: MarksState;
  me?: string | null;
  /** Show hidden photos dimmed with a mark (in a list, F014/S02); elsewhere they aren't here. */
  dimHidden?: boolean;
  /** Stacks (F015/S03): which are unfolded, folding them, and whether this is a list (thin
   * borders only). The stack ids come with `marks`. */
  unfolded?: Set<string>;
  onToggleStack?: (id: string) => void;
  inList?: boolean;
  /** The stars and heart for the selection, drawn on the selected tile (F013, on Peter's request). */
  activeBar?: (tile: number) => ReactNode;
  /** A tile starts being dragged (to a list in the sidebar). */
  onDragTile?: (sel: Sel) => void;
  /** Right-click on a tile, at this point in the window. */
  onMenu?: (sel: Sel, x: number, y: number) => void;
};

const GAP = 12;
/** How far a stack's outline reaches beyond its tiles: just outside the 3 px selection ring; two
 * stacks side by side keep GAP − 2·BAND_PAD apart. */
const BAND_PAD = 5;
const HEADER = 52;

/** The whole library as one grid, virtualized per row, with a header where each month begins (F001/S01). */
export default function Grid({ sections, itemsOf, selected, onSelect, onOpen, onColumns, onNeed, onTopSection, jump, tile, marked, picked, marks, me, dimHidden, unfolded, onToggleStack, inList, activeBar, onDragTile, onMenu }: Props) {
  // A callback ref, so the width is measured as soon as the scroll box exists. On the first
  // render the library is often still empty and there is no scroll box yet; a ref read once in
  // an effect would never measure.
  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);
  const [cols, setCols] = useState(1);

  useLayoutEffect(() => {
    if (!scrollEl) return;
    const update = () => setCols(columnsFor(scrollEl.clientWidth, tile, GAP));
    update();
    const ro = new ResizeObserver(update);
    ro.observe(scrollEl);
    return () => ro.disconnect();
  }, [scrollEl, tile]);

  useEffect(() => onColumns?.(cols), [cols, onColumns]);

  const layout = useMemo(() => buildLayout(sections, cols), [sections, cols]);
  const virtualizer = useVirtualizer({
    count: layout.rows.length,
    getScrollElement: () => scrollEl,
    estimateSize: (i) => (layout.rows[i].kind === "header" ? HEADER : tile + GAP),
    overscan: 6,
  });

  // A new layout (columns, tile size, library) changes row heights; re-measure so rows don't overlap.
  useEffect(() => virtualizer.measure(), [layout, tile, virtualizer]);

  const visible = virtualizer.getVirtualItems();

  // The stacks with a selected or picked photo: their border is stronger, with the symbol.
  const selectedStacks = new Set<string>();
  const stackAt = (s: number, i: number) => marks?.stacks[itemsOf(sections[s]?.month ?? "")?.[i]?.hash ?? ""];
  if (selected) { const id = stackAt(selected.section, selected.index); if (id) selectedStacks.add(id); }
  for (const k of picked ?? []) {
    const [s, i] = k.split(":").map(Number);
    const id = stackAt(s, i);
    if (id) selectedStacks.add(id);
  }
  const first = visible.length ? visible[0].index : 0;
  const last = visible.length ? visible[visible.length - 1].index : -1;

  // Fetch the months that are (nearly) on screen.
  useEffect(() => {
    if (last >= first) onNeed(sectionsInRange(layout, first, last));
  }, [first, last, layout, onNeed]);

  // Which month is at the top: the first row whose bottom is below the scroll position.
  const offset = virtualizer.scrollOffset ?? 0;
  const topRow = visible.find((v) => v.end > offset)?.index;
  const topSection = topRow !== undefined ? layout.rows[topRow]?.section : undefined;
  const lastTop = useRef<number | undefined>(undefined);
  useEffect(() => {
    if (topSection !== undefined && topSection !== lastTop.current) {
      lastTop.current = topSection;
      onTopSection(topSection);
    }
  }, [topSection, onTopSection]);

  // Keep the selected tile in view when the selection moves (arrow keys) or the columns change, but
  // not on every new layout: the grid would jump back to it while scrolling elsewhere.
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  const selKey = selected ? `${selected.section}:${selected.index}` : null;
  useEffect(() => {
    if (selected) virtualizer.scrollToIndex(rowOf(layoutRef.current, selected, cols), { align: "auto" });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selKey, cols, virtualizer]);

  // Jump from the month list: that month's header at the top.
  useEffect(() => {
    if (jump) virtualizer.scrollToIndex(layout.headerRow[jump.section] ?? 0, { align: "start" });
    // only on a new jump, not on every layout change
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jump]);

  if (sections.length === 0) {
    return <div className="h-full grid place-items-center text-neutral-400">niets om te tonen</div>;
  }

  const width = rowWidth(cols, tile, GAP);

  return (
    <div ref={setScrollEl} className="h-full overflow-y-auto overflow-x-hidden" style={{ padding: GAP }}>
      {/* As wide as a full row and centred, so leftover width is split evenly left and right. */}
      <div className="mx-auto" style={{ height: virtualizer.getTotalSize(), width, position: "relative" }}>
        {visible.map((v) => {
          const row = layout.rows[v.index];
          const section = sections[row.section];
          if (row.kind === "header") {
            return (
              <div
                key={v.key}
                className="absolute left-0 right-0 flex items-baseline justify-between pt-4 pb-1.5 text-sm text-neutral-400 dark:text-neutral-500 border-b border-neutral-300 dark:border-neutral-700"
                style={{ top: v.start, height: HEADER - GAP }}
              >
                <span className="text-base font-semibold text-neutral-700 dark:text-neutral-200">
                  {monthLabel(section.month)} {yearOf(section.month)}
                </span>
                <span className="tabular-nums">{section.count}</span>
              </div>
            );
          }
          const items = itemsOf(section.month);
          const ids = marks?.stacks ?? {};
          const start = row.row * cols;
          const n = Math.min(cols, section.count - start);
          return (
            <div key={v.key} className="absolute left-0 flex" style={{ top: v.start, height: tile, gap: GAP }}>
              {/* the stacks' outline: per tile, its background reaches into the gap towards
                  neighbors of the same stack, with a border only on the other sides (F015) */}
              {!inList && items && Array.from({ length: n }, (_, i) => {
                const k = cell(items, start + i, cols, ids);
                if (!k) return null;
                const c = stackColor(k.id, marks?.stack_colors[k.id]);
                const side = `${selectedStacks.has(k.id) ? 3 : 2}px solid ${c.border}`;
                const x = i * (tile + GAP);
                const l = k.left ? GAP / 2 : BAND_PAD, r = k.right ? GAP / 2 : BAND_PAD;
                const t = k.up ? GAP / 2 : BAND_PAD, bt = k.down ? GAP / 2 : BAND_PAD;
                const round = (a: boolean, b: boolean) => (a || b ? 0 : 10);
                return (
                  <div
                    key={`stack-${start + i}`}
                    className="absolute pointer-events-none"
                    style={{
                      left: x - l, width: tile + l + r, top: -t, height: tile + t + bt,
                      background: c.bg,
                      borderLeft: k.left ? "none" : side, borderRight: k.right ? "none" : side,
                      borderTop: k.up ? "none" : side, borderBottom: k.down ? "none" : side,
                      borderRadius: `${round(k.left, k.up)}px ${round(k.right, k.up)}px ${round(k.right, k.down)}px ${round(k.left, k.down)}px`,
                    }}
                  />
                );
              })}
              {Array.from({ length: n }, (_, i) => {
                const index = start + i;
                const item = items?.[index];
                const sel = { section: row.section, index };
                const active = selected?.section === row.section && selected.index === index;
                return item ? (
                  <Tile
                    key={item.path}
                    item={item}
                    size={tile}
                    active={active}
                    member={marked?.get(item.hash) ?? null}
                    rating={marks?.ratings[item.hash]?.stars}
                    hearts={marks?.hearts[item.hash]}
                    me={me ?? null}
                    hidden={!!dimHidden && !!marks?.hidden[item.hash]}
                    bar={active ? activeBar?.(tile) : undefined}
                    stack={(() => {
                      const id = ids[item.hash];
                      if (!id) return null;
                      const strong = selectedStacks.has(id);
                      return {
                        color: stackColor(id, marks?.stack_colors[id]),
                        strong,
                        thin: !!inList,
                        symbol: !inList && strong && !!items && lastOfStack(items, index, ids),
                        open: !!unfolded?.has(id),
                        onToggle: () => onToggleStack?.(id),
                      };
                    })()}
                    title={[fileName(item.path), ...marksLines(marks?.ratings[item.hash], marks?.hearts[item.hash])].join("\n")}
                    picked={picked?.has(`${row.section}:${index}`) ?? false}
                    onClick={(e) => onSelect(sel, { shift: e.shiftKey, meta: e.metaKey || e.ctrlKey })}
                    onDragStart={(e) => {
                      e.dataTransfer.setData("text/plain", item.path);
                      e.dataTransfer.effectAllowed = "copy";
                      onDragTile?.(sel);
                    }}
                    onContextMenu={(e) => { e.preventDefault(); onMenu?.(sel, e.clientX, e.clientY); }}
                    onDoubleClick={() => onOpen(sel)}
                  />
                ) : (
                  <div
                    key={`${section.month}#${index}`}
                    className={"rounded-md bg-neutral-200 dark:bg-neutral-800 " + (active ? "ring-3 ring-blue-500" : "")}
                    style={{ width: tile, height: tile }}
                  />
                );
              })}
            </div>
          );
        })}
      </div>
    </div>
  );
}

type TileStack = { color: StackColor; strong: boolean; thin: boolean; symbol: boolean; open: boolean; onToggle: () => void };

function Tile({ item, size, active, member, picked, rating, hearts, me, hidden, stack, bar, title, onClick, onDoubleClick, onDragStart, onContextMenu }: {
  item: Item; size: number; active: boolean; member: Member | null; picked: boolean; hidden: boolean; stack: TileStack | null;
  bar?: ReactNode;
  rating: number | undefined; hearts: Record<string, string> | undefined; me: string | null; title: string;
  onClick: (e: MouseEvent) => void; onDoubleClick: () => void;
  onDragStart: (e: DragEvent) => void; onContextMenu: (e: MouseEvent) => void;
}) {
  return (
    <div
      draggable
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      onDragStart={onDragStart}
      onContextMenu={onContextMenu}
      title={[title, member && addedText(member), hidden && "verborgen"].filter(Boolean).join("\n")}
      className={
        "relative rounded-md overflow-hidden bg-neutral-200 dark:bg-neutral-800 cursor-default " +
        (active ? "ring-3 ring-blue-500" : picked ? "ring-3 ring-blue-300" : "")
      }
      style={{
        width: size,
        height: size,
        // in a list a stack shows as a thin border per tile; in the library the row draws a band
        ...(stack?.thin ? { border: `2px solid ${stack.color.border}` } : {}),
      }}
    >
      {picked && <span className="absolute inset-0 bg-blue-500/25 pointer-events-none" />}
      {hidden && (
        <>
          <span className="absolute inset-0 bg-white/55 dark:bg-black/55 pointer-events-none" />
          <span className="absolute inset-0 grid place-items-center pointer-events-none"><span className="rounded bg-black/60 text-white text-[10px] px-1">verborgen</span></span>
        </>
      )}
      {item.preview ? (
        <img
          src={previewUrl(item.path, true)}
          loading="lazy"
          decoding="async"
          draggable={false}
          className="w-full h-full object-cover"
          alt=""
        />
      ) : (
        <div className="w-full h-full grid place-items-center text-xs text-neutral-400 p-2 text-center">
          geen preview
        </div>
      )}
      {(item.kind === "video" || !item.local) && (
        <span className="absolute top-1 left-1 flex items-center gap-1 leading-none">
          {item.kind === "video" && (
            <span title={liveTitle(item) ?? "video"}>
              {item.live ? <LiveIcon size={Math.max(12, markSize(size))} /> : <VideoIcon size={Math.max(12, markSize(size))} />}
            </span>
          )}
          {!item.local && (
            <span className="rounded bg-black/50 text-white text-[10px] px-1 py-px" title="origineel alleen op de NAS">
              NAS
            </span>
          )}
        </span>
      )}
      {bar ?? <TileMarks size={size} rating={rating} hearts={hearts} me={me} shiftHeart={!!stack?.symbol} />}
      {stack?.symbol && (
        <button
          className="absolute top-1 right-1 rounded-md bg-white/90 dark:bg-neutral-900/90 p-0.5 shadow"
          style={{ color: stack.color.border }}
          title={stack.open ? "reeks dichtklappen" : "reeks openklappen: alle foto's tonen"}
          onClick={(e) => { e.stopPropagation(); stack.onToggle(); }}
          onDoubleClick={(e) => e.stopPropagation()}
        >
          <StackIcon size={Math.max(16, markSize(size) + 4)} open={stack.open} />
        </button>
      )}
    </div>
  );
}

/** Stars bottom right and the heart top right, scaled with the tile (F013/S02). */
function TileMarks({ size, rating, hearts, me, shiftHeart }: {
  size: number; rating: number | undefined; hearts: Record<string, string> | undefined; me: string | null; shiftHeart: boolean;
}) {
  const px = markSize(size);
  const heart = heartView(hearts, me);
  return (
    <>
      {heart && (
        <span
          className="absolute top-1 flex items-center gap-0.5 rounded-full bg-black/40 px-1 leading-none"
          // left of the stack symbol when that is shown
          style={{ fontSize: px, right: shiftHeart ? Math.max(16, px + 4) + 14 : 4 }}
        >
          <span className={heart.mine ? "text-red-500" : "text-white"}>♥</span>
          {heart.count > 1 && <span className="text-white tabular-nums" style={{ fontSize: Math.round(px * 0.7) }}>{heart.count}</span>}
        </span>
      )}
      {rating ? (
        <span
          className="absolute bottom-1 right-1 rounded bg-black/50 px-1 text-amber-300 leading-none tracking-tight"
          style={{ fontSize: px * 0.8, paddingTop: 2, paddingBottom: 2 }}
        >
          {starsText(rating)}
        </span>
      ) : null}
    </>
  );
}
