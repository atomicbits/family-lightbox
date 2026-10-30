import { useEffect, useRef, useState } from "react";
import { cameraButton, cameraLabel, filterText, isSet, lastUsedLabel, sortCameras, toggleCamera, type CameraCount, type CameraOrder, type FilterState, type Kind, type StarsOp } from "../lib/filter";

type Props = {
  filter: FilterState;
  onChange: (f: FilterState) => void;
  /** The people who put hearts, to tick one or more. */
  people: string[];
  /** "1 234 van 124 837" while a filter is set. */
  count: string | null;
  /** The library's cameras with their counts and when last used, "" for unknown (S04). */
  cameras: CameraCount[];
};

const OPS: { op: StarsOp; label: string }[] = [{ op: "gte", label: "≥" }, { op: "eq", label: "=" }, { op: "lte", label: "≤" }];
const KINDS: { kind: Kind; label: string }[] = [{ kind: "photo", label: "foto" }, { kind: "video", label: "video" }, { kind: "live", label: "Live" }];

const chip = (on: boolean) =>
  "px-1.5 py-0.5 rounded text-xs " + (on ? "bg-blue-600 text-white" : "text-neutral-500 hover:bg-neutral-200/70 dark:hover:bg-neutral-800");

/** The camera button and its list (S04): light in the bar, as it is used less often. */
function CameraMenu({ chosen, cameras, onChange }: { chosen: string[]; cameras: CameraCount[]; onChange: (c: string[]) => void }) {
  const [open, setOpen] = useState(false);
  const [order, setOrder] = useState<CameraOrder>("recent");
  const box = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    if (!open) return;
    const down = (e: MouseEvent) => { if (!box.current?.contains(e.target as Node)) setOpen(false); };
    const key = (e: KeyboardEvent) => { if (e.key === "Escape") { e.stopPropagation(); setOpen(false); } };
    window.addEventListener("mousedown", down);
    window.addEventListener("keydown", key, true);
    return () => { window.removeEventListener("mousedown", down); window.removeEventListener("keydown", key, true); };
  }, [open]);
  const set = chosen.length > 0;
  return (
    <span className="relative" ref={box}>
      <button className={chip(set) + " max-w-40 truncate"} onClick={() => setOpen((o) => !o)} title={set ? chosen.map(cameraLabel).join(", ") : "filter op camera"}>
        {cameraButton(chosen)} ▾
      </button>
      {open && (
        <div className="absolute left-1/2 -translate-x-1/2 top-full mt-1 z-30 min-w-56 max-h-96 overflow-y-auto rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 shadow-lg py-1 text-xs">
          <button className="flex w-full items-center gap-2 px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={() => onChange(toggleCamera(chosen, null))}>
            <input type="checkbox" readOnly checked={!set} className="pointer-events-none" />
            <span className="flex-1 text-left">Alle camera's</span>
          </button>
          <div className="my-1 border-t border-neutral-200 dark:border-neutral-700" />
          <div className="flex items-center gap-1 px-3 pb-1 text-neutral-400">
            <span>sorteer</span>
            <button className={chip(order === "recent")} onClick={() => setOrder("recent")} title="laatst gebruikt eerst">recent</button>
            <button className={chip(order === "count")} onClick={() => setOrder("count")} title="meeste foto's en video's eerst">aantal</button>
          </div>
          {cameras.length === 0 && <div className="px-3 py-1 text-neutral-400">nog geen camera's in de index</div>}
          {sortCameras(cameras, order).map(([c, n, last]) => (
            <button key={c} className="flex w-full items-center gap-2 px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={() => onChange(toggleCamera(chosen, c))}>
              <input type="checkbox" readOnly checked={chosen.includes(c)} className="pointer-events-none" />
              <span className={"flex-1 text-left " + (c ? "" : "italic text-neutral-500")}>{cameraLabel(c)}</span>
              <span className="tabular-nums text-neutral-400" title="laatst gebruikt">{c ? lastUsedLabel(last) : ""}</span>
              <span className="tabular-nums text-neutral-400 w-14 text-right">{n.toLocaleString("nl-BE")}</span>
            </button>
          ))}
        </div>
      )}
    </span>
  );
}

/** The filter in the toolbar, centered above the grid (F002/S03): stars, hearts, kind and camera. */
export default function FilterBar({ filter, onChange, people, count, cameras }: Props) {
  const [hover, setHover] = useState(0);
  const set = isSet(filter);
  const stars = filter.stars;
  const op = stars && stars.op !== "none" ? stars.op : "gte";
  const shown = stars && stars.op !== "none" ? stars.n : 0;

  const clickStar = (n: number) =>
    onChange({ ...filter, stars: stars && stars.op === op && stars.n === n ? null : { op, n } });
  const setOp = (o: StarsOp) => onChange({ ...filter, stars: { op: o, n: stars && stars.op !== "none" ? stars.n : 1 } });
  const toggleKind = (k: Kind) =>
    onChange({ ...filter, kinds: filter.kinds.includes(k) ? filter.kinds.filter((x) => x !== k) : [...filter.kinds, k] });
  const hearts = filter.hearts;
  const toggleAnyone = () => onChange({ ...filter, hearts: hearts?.anyone ? null : { anyone: true, people: [] } });
  const togglePerson = (p: string) => {
    const now = hearts && !hearts.anyone ? hearts.people : [];
    const next = now.includes(p) ? now.filter((x) => x !== p) : [...now, p];
    onChange({ ...filter, hearts: next.length ? { anyone: false, people: next } : null });
  };

  return (
    <div
      className={"flex items-center gap-2 rounded-md px-2 py-0.5 whitespace-nowrap " + (set ? "bg-blue-50 dark:bg-blue-950/60 ring-1 ring-blue-300 dark:ring-blue-800" : "")}
      title={set ? `Filter: ${filterText(filter)}` : "filter op sterren, hartjes, soort en camera"}
    >
      <span className="text-xs text-neutral-400">Filter</span>
      {/* stars */}
      <span className="flex items-center">
        {OPS.map((o) => (
          <button key={o.op} className={chip(!!stars && stars.op === o.op)} onClick={() => setOp(o.op)} title={o.op === "gte" ? "minstens" : o.op === "eq" ? "precies" : "hoogstens"}>
            {o.label}
          </button>
        ))}
        <span className="flex items-center ml-0.5" onMouseLeave={() => setHover(0)}>
          {[1, 2, 3, 4, 5].map((n) => (
            <button
              key={n}
              className={"px-px text-sm leading-none " + ((hover ? n <= hover : n <= shown) ? "text-amber-400" : "text-neutral-300 dark:text-neutral-600")}
              onMouseEnter={() => setHover(n)}
              onClick={() => clickStar(n)}
              title={`${OPS.find((o) => o.op === op)?.label} ${n} ${n === 1 ? "ster" : "sterren"}`}
            >
              ★
            </button>
          ))}
        </span>
        <button
          className={chip(stars?.op === "none") + " ml-0.5"}
          onClick={() => onChange({ ...filter, stars: stars?.op === "none" ? null : { op: "none", n: 0 } })}
          title="alleen foto's zonder sterren"
        >
          geen
        </button>
      </span>
      <span className="text-neutral-300 dark:text-neutral-700">|</span>
      {/* hearts */}
      <span className="flex items-center gap-0.5">
        <button className={chip(!!hearts?.anyone)} onClick={toggleAnyone} title="foto's met een hartje van iemand">
          <span className={hearts?.anyone ? "" : "text-red-500"}>♥</span> iemand
        </button>
        {people.map((p) => (
          <button key={p} className={chip(!!hearts && !hearts.anyone && hearts.people.includes(p))} onClick={() => togglePerson(p)} title={`foto's met een hartje van ${p}`}>
            {p}
          </button>
        ))}
      </span>
      <span className="text-neutral-300 dark:text-neutral-700">|</span>
      {/* kind */}
      <span className="flex items-center gap-0.5">
        {KINDS.map((k) => (
          <button key={k.kind} className={chip(filter.kinds.includes(k.kind))} onClick={() => toggleKind(k.kind)}>
            {k.label}
          </button>
        ))}
      </span>
      <span className="text-neutral-300 dark:text-neutral-700">|</span>
      <CameraMenu chosen={filter.cameras} cameras={cameras} onChange={(c) => onChange({ ...filter, cameras: c })} />
      {set && (
        <>
          {count && <span className="text-xs text-blue-700 dark:text-blue-300 tabular-nums">{count}</span>}
          <button className="text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200" title="filter wissen" onClick={() => onChange({ stars: null, hearts: null, kinds: [], cameras: [] })}>
            ✕
          </button>
        </>
      )}
    </div>
  );
}
