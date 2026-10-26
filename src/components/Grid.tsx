import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { Item } from "../lib/api";
import { previewUrl } from "../lib/api";
import { columnsFor, rowWidth } from "../lib/grid";
import { fileName, monthLabel, yearOf } from "../lib/months";
import { buildLayout, rowOf, sectionsInRange, type Section, type Sel } from "../lib/timeline";

type Props = {
  sections: Section[];
  /** A month's items, or undefined while it isn't loaded yet. */
  itemsOf: (month: string) => Item[] | undefined;
  selected: Sel | null;
  onSelect: (sel: Sel) => void;
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
};

const GAP = 8;
const HEADER = 52;

/** The whole library as one grid, virtualized per row, with a header where each month begins (F001/S01). */
export default function Grid({ sections, itemsOf, selected, onSelect, onOpen, onColumns, onNeed, onTopSection, jump, tile }: Props) {
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

  // Keep the selected tile in view (arrow keys), also after the layout changes.
  useEffect(() => {
    if (selected) virtualizer.scrollToIndex(rowOf(layout, selected, cols), { align: "auto" });
  }, [selected, layout, cols, virtualizer]);

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
          const start = row.row * cols;
          const n = Math.min(cols, section.count - start);
          return (
            <div key={v.key} className="absolute left-0 flex" style={{ top: v.start, height: tile, gap: GAP }}>
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
                    onClick={() => onSelect(sel)}
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

function Tile({ item, size, active, onClick, onDoubleClick }: {
  item: Item; size: number; active: boolean; onClick: () => void; onDoubleClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      title={fileName(item.path)}
      className={
        "relative rounded-md overflow-hidden bg-neutral-200 dark:bg-neutral-800 cursor-default " +
        (active ? "ring-3 ring-blue-500" : "")
      }
      style={{ width: size, height: size }}
    >
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
      {item.kind === "video" && (
        <span className="absolute bottom-1 left-1 rounded bg-black/60 text-white text-[10px] px-1">video</span>
      )}
      {!item.local && (
        <span className="absolute top-1 right-1 rounded bg-black/50 text-white text-[10px] px-1" title="origineel alleen op de NAS">
          NAS
        </span>
      )}
    </div>
  );
}
