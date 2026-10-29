import { useState } from "react";
import { heartAfterClick, heartState, sharedStars, starsAfterClick, type MarksState } from "../lib/marks";

type Props = {
  /** What the selection is: "12 foto's", or the file name of one. */
  label: string;
  hashes: string[];
  marks: MarksState;
  me: string | null;
  blocked: string | null;
  onRate: (stars: number) => void;
  onHeart: (on: boolean) => void;
  /** Darker colors in the lightbox. */
  dark?: boolean;
};

/** Five stars and a heart for the selected photos (F013/S03). */
export default function MarksBar({ label, hashes, marks, me, blocked, onRate, onHeart, dark }: Props) {
  // the star under the mouse: it and every star before it light up, as the rating a click gives
  const [hover, setHover] = useState(0);
  if (hashes.length === 0) return null;
  const shared = sharedStars(hashes, marks.ratings);
  const heart = heartState(hashes, marks.hearts, me);
  const muted = dark ? "text-neutral-500" : "text-neutral-300 dark:text-neutral-600";
  const tip = (t: string) => blocked ?? t;
  return (
    <span className="flex items-center gap-1 whitespace-nowrap" title={blocked ?? undefined}>
      <span className={dark ? "text-neutral-400" : "text-neutral-500"}>{label}</span>
      <span className="flex items-center" onMouseDown={(e) => e.preventDefault()} onMouseLeave={() => setHover(0)}>
        {shared === null && <span className={`${muted} mr-0.5`} title="de gekozen foto's hebben verschillende sterren">–</span>}
        {[1, 2, 3, 4, 5].map((n) => (
          <button
            key={n}
            disabled={!!blocked}
            className={
              "px-px text-base leading-none disabled:opacity-40 " +
              (hover && !blocked ? (n <= hover ? "text-amber-500" : muted) : shared !== null && n <= shared ? "text-amber-400" : muted)
            }
            onMouseEnter={() => setHover(n)}
            title={tip(shared === n ? "sterren weghalen (0)" : `${n} ${n === 1 ? "ster" : "sterren"} (${n})`)}
            onClick={() => onRate(starsAfterClick(shared, n))}
          >
            ★
          </button>
        ))}
      </span>
      <button
        disabled={!!blocked || !me}
        className={"px-1 text-base leading-none disabled:opacity-40 " + (heart === "none" ? muted + " hover:text-red-400" : "text-red-500")}
        style={heart === "some" ? { opacity: 0.55 } : undefined}
        title={tip(heart === "all" ? "je hartje weghalen (H)" : heart === "some" ? "je hartje op alle gekozen foto's (H)" : "een hartje (H)")}
        onMouseDown={(e) => e.preventDefault()}
        onClick={() => onHeart(heartAfterClick(heart))}
      >
        {heart === "none" ? "♡" : "♥"}
      </button>
    </span>
  );
}
