import { useEffect, useLayoutEffect, useMemo, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { Item } from "../lib/api";
import { previewUrl } from "../lib/api";
import { columnsFor } from "../lib/grid";
import { fileName } from "../lib/months";

type Props = {
  items: Item[];
  selected: number | null;
  onSelect: (index: number) => void;
  onOpen: (index: number) => void;
  /** Reports the column count, so the arrow keys can move a row up or down. */
  onColumns?: (cols: number) => void;
};

const TILE = 176; // px, miniatuur is 400 px lang, dus scherp op Retina
const GAP = 8;

/** Het raster: gevirtualiseerd per rij, zodat een maand met duizenden foto's licht blijft. */
export default function Grid({ items, selected, onSelect, onOpen, onColumns }: Props) {
  // A callback ref, so the width is measured as soon as the scroll box exists. On the first
  // render the month is often still empty and there is no scroll box yet; a ref read once in
  // an effect would never measure and the grid stayed at its initial 6 columns.
  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);
  const [cols, setCols] = useState(1);

  useLayoutEffect(() => {
    if (!scrollEl) return;
    const update = () => setCols(columnsFor(scrollEl.clientWidth, TILE, GAP));
    update();
    const ro = new ResizeObserver(update);
    ro.observe(scrollEl);
    return () => ro.disconnect();
  }, [scrollEl]);

  useEffect(() => onColumns?.(cols), [cols, onColumns]);

  const rows = useMemo(() => Math.ceil(items.length / cols), [items.length, cols]);
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => scrollEl,
    estimateSize: () => TILE + GAP,
    overscan: 4,
  });

  // de geselecteerde tegel in beeld houden (pijltjestoetsen)
  useEffect(() => {
    if (selected !== null) virtualizer.scrollToIndex(Math.floor(selected / cols), { align: "auto" });
  }, [selected, cols, virtualizer]);

  if (items.length === 0) {
    return <div className="h-full grid place-items-center text-neutral-400">niets in deze maand</div>;
  }

  return (
    <div ref={setScrollEl} className="h-full overflow-y-auto overflow-x-hidden" style={{ padding: GAP }}>
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((row) => (
          <div
            key={row.key}
            className="absolute left-0 flex"
            style={{ top: row.start, height: TILE, gap: GAP }}
          >
            {items.slice(row.index * cols, row.index * cols + cols).map((item, i) => {
              const index = row.index * cols + i;
              return (
                <Tile
                  key={item.path}
                  item={item}
                  active={index === selected}
                  onClick={() => onSelect(index)}
                  onDoubleClick={() => onOpen(index)}
                />
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

function Tile({ item, active, onClick, onDoubleClick }: {
  item: Item; active: boolean; onClick: () => void; onDoubleClick: () => void;
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
      style={{ width: TILE, height: TILE }}
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
