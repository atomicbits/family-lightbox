import { useState } from "react";
import { heartAfterClick, heartState, markSize, sharedStars, starsAfterClick, type MarksState } from "../lib/marks";

type Props = {
  /** How many photos it applies to, shown on the tile when more than one (F013). */
  count: number;
  hashes: string[];
  marks: MarksState;
  me: string | null;
  blocked: string | null;
  onRate: (stars: number) => void;
  onHeart: (on: boolean) => void;
  /** On the selected tile, at its size; or in the lightbox's dark bar. */
  variant: { tile: number } | "lightbox";
};

/** Five stars and a heart for the selected photos: on the selected tile, or in the lightbox. */
export default function MarksBar({ count, hashes, marks, me, blocked, onRate, onHeart, variant }: Props) {
  // the star under the mouse: it and every star before it light up, as the rating a click gives
  const [hover, setHover] = useState(0);
  if (hashes.length === 0) return null;
  const shared = sharedStars(hashes, marks.ratings);
  const heart = heartState(hashes, marks.hearts, me);
  const onTile = variant !== "lightbox";
  const px = onTile ? markSize(variant.tile) + 3 : 16;
  const muted = onTile ? "text-white/45" : "text-neutral-500";
  const tip = (t: string) => blocked ?? t;
  // clicks here don't select, open or drag the tile
  const stop = { onClick: (e: React.MouseEvent) => e.stopPropagation(), onDoubleClick: (e: React.MouseEvent) => e.stopPropagation(), onMouseDown: (e: React.MouseEvent) => e.preventDefault() };
  const stars = (
    <span className="flex items-center" onMouseLeave={() => setHover(0)}>
      {shared === null && <span className={`${muted} mr-0.5`} style={{ fontSize: px * 0.8 }} title="de gekozen foto's hebben verschillende sterren">–</span>}
      {[1, 2, 3, 4, 5].map((n) => (
        <button
          key={n}
          disabled={!!blocked}
          className={
            "px-px leading-none disabled:opacity-40 " +
            (hover && !blocked ? (n <= hover ? "text-amber-400" : muted) : shared !== null && n <= shared ? "text-amber-400" : muted)
          }
          style={{ fontSize: px }}
          onMouseEnter={() => setHover(n)}
          title={tip(shared === n ? "sterren weghalen (0)" : `${n} ${n === 1 ? "ster" : "sterren"} (${n})`)}
          onClick={(e) => { e.stopPropagation(); onRate(starsAfterClick(shared, n)); }}
        >
          ★
        </button>
      ))}
    </span>
  );
  const heartButton = (
    <button
      disabled={!!blocked || !me}
      className={"px-0.5 leading-none disabled:opacity-40 " + (heart === "none" ? muted + " hover:text-red-400" : "text-red-500")}
      style={{ fontSize: px, ...(heart === "some" ? { opacity: 0.55 } : {}) }}
      title={tip(heart === "all" ? "je hartje weghalen (H)" : heart === "some" ? "je hartje op alle gekozen foto's (H)" : "een hartje (H)")}
      onClick={(e) => { e.stopPropagation(); onHeart(heartAfterClick(heart)); }}
    >
      {heart === "none" ? "♡" : "♥"}
    </button>
  );
  if (!onTile) {
    return <span className="flex items-center gap-1 whitespace-nowrap" title={blocked ?? undefined} {...stop}>{stars}{heartButton}</span>;
  }
  return (
    <>
      {count > 1 && (
        <span className="absolute top-1 left-1 rounded bg-blue-600 text-white px-1 shadow" style={{ fontSize: Math.max(10, px * 0.65) }}>
          {count} foto's
        </span>
      )}
      <span
        className="absolute inset-x-0 bottom-0 flex items-center justify-between gap-1 bg-black/60 px-1 py-0.5 whitespace-nowrap"
        title={blocked ?? undefined}
        draggable={false}
        onDragStart={(e) => { e.preventDefault(); e.stopPropagation(); }}
        {...stop}
      >
        {stars}
        {heartButton}
      </span>
    </>
  );
}
