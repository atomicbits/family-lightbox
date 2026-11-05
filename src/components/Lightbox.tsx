import { useEffect, useState, type ReactNode } from "react";
import type { Item } from "../lib/api";
import { api, previewUrl } from "../lib/api";
import { fileName, takenLabel } from "../lib/months";
import { addedText, type Member } from "../lib/lists";
import { heartView, type Rating } from "../lib/marks";
import VideoIcon from "./VideoIcon";
import LiveIcon from "./LiveIcon";
import { liveTitle } from "../lib/live";
import { allVariants, chosenVariant, variantLabel } from "../lib/variants";
import { t } from "../i18n";

type Props = {
  /** Fetch this original from the NAS (F009/S03). */
  onFetch?: (path: string) => void;
  item: Item;
  index: number;
  count: number;
  onClose: () => void;
  onStep: (delta: number) => void;
  /** Whether stepping back or forward is possible; it can cross into the month above or below. */
  canPrev: boolean;
  canNext: boolean;
  /** In a list: who added this photo and when (F003/S04). */
  member: Member | null;
  /** Its stars and hearts, and this Mac's owner (F013/S02). */
  rating?: Rating;
  hearts?: Record<string, string>;
  me: string | null;
  /** The stars and heart to set (F013/S03), and their keys: true when the key was one of them. */
  bar: ReactNode;
  onMarkKey: (e: KeyboardEvent) => boolean;
  /** Right-click on the photo: the same menu as on a tile (F014). */
  onMenu: (x: number, y: number) => void;
  /** The version chosen in the marks, and choosing one (F025/S04). */
  variantMark?: string;
  onChooseVariant?: (variant: string) => void;
};

/** Eén foto groot (de 2048 px-preview), met pijltjes, en knoppen naar het origineel. */
export default function Lightbox({ item, index, count, canPrev, canNext, onClose, onStep, member, rating, hearts, me, bar, onMarkKey, onMenu, onFetch, variantMark, onChooseVariant }: Props) {
  const heart = heartView(hearts, me);
  const [message, setMessage] = useState<string | null>(null);
  // the version shown: the chosen one, until another is looked at (F025/S04)
  const chosen = chosenVariant(item.variants, variantMark);
  const [looking, setLooking] = useState<string | null>(null);
  useEffect(() => setLooking(null), [item.path]);
  const shown = looking ?? chosen;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" || e.key === " ") { e.preventDefault(); onClose(); }
      else if (e.key === "ArrowRight") onStep(1);
      else if (e.key === "ArrowLeft") onStep(-1);
      else onMarkKey(e);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, onStep, onMarkKey]);

  useEffect(() => setMessage(null), [item.path]);

  const run = (f: () => Promise<void>) => f().catch((e) => setMessage(String(e)));

  return (
    <div className="absolute inset-0 z-20 bg-black/95 text-white flex flex-col" onClick={onClose}>
      {/* The image is positioned absolutely in a box with a definite size: in a grid or flex item
          with an auto height, max-h-full has nothing to resolve against and a tall photo spills
          out at the bottom. */}
      <div
        className="relative flex-1 min-h-0 m-4"
        onClick={(e) => e.stopPropagation()}
        onContextMenu={(e) => { e.preventDefault(); onMenu(e.clientX, e.clientY); }}
      >
        {item.preview ? (
          <img
            src={previewUrl(item.path, false, shown)}
            className="absolute inset-0 w-full h-full object-contain"
            draggable={false}
            alt=""
          />
        ) : (
          <div className="absolute inset-0 grid place-items-center text-neutral-400">
            {t().browse.lightbox.noPreview(fileName(item.path))}
          </div>
        )}
      </div>
      <div
        className="flex items-center gap-4 px-4 py-2 text-sm bg-black/60"
        onClick={(e) => e.stopPropagation()}
      >
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={() => onStep(-1)} disabled={!canPrev}>◀</button>
        <span className="tabular-nums text-neutral-400">{index + 1} / {count}</span>
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={() => onStep(1)} disabled={!canNext}>▶</button>
        <span className="font-medium">{fileName(item.path)}</span>
        <span className="text-neutral-400">{takenLabel(item.taken)}</span>
        {item.source && <span className="text-neutral-400">{item.source}</span>}
        {item.kind === "video" && (
          <span className="flex items-center gap-1 text-neutral-400" title={liveTitle(item) ?? t().browse.video}>
            {item.live ? <><LiveIcon size={14} /> {t().browse.live.label}</> : <><VideoIcon size={14} /> {t().browse.video}</>}
          </span>
        )}
        {!item.local && (
          <span className="rounded bg-white/15 text-neutral-300 text-xs px-1.5 py-0.5" title={t().browse.lightbox.nasOnly}>
            NAS
          </span>
        )}
        {(item.variants?.length ?? 0) > 0 && (
          <span className="flex items-center gap-1" title={t().browse.lightbox.versions}>
            {allVariants(item.variants).map((v) => (
              <button
                key={v}
                className={"px-2 py-0.5 rounded text-xs " + (v === shown ? "bg-white/25" : "bg-white/5 hover:bg-white/15") + (v === chosen ? " ring-1 ring-white/50" : "")}
                title={v === chosen ? t().browse.lightbox.chosen : t().browse.lightbox.look}
                onClick={() => setLooking(v)}
              >
                {variantLabel(v)}
              </button>
            ))}
            {shown !== chosen && onChooseVariant && (
              <button className="px-2 py-0.5 rounded text-xs bg-blue-600 hover:bg-blue-500" onClick={() => onChooseVariant(shown)}>
                {t().browse.lightbox.showVersion}
              </button>
            )}
          </span>
        )}
        <span className="flex-1" />
        {message && <span className="text-amber-300">{message}</span>}
        <span title={rating ? t().browse.lightbox.rated(rating.stars, rating.by) : undefined}>{bar}</span>
        {heart && (
          <span title={t().browse.lightbox.hearts(heart.who)}>
            <span className={heart.mine ? "text-red-500" : "text-white"}>♥</span>
            <span className="text-neutral-400"> {heart.who.join(", ")}</span>
          </span>
        )}
        {member && <span className="text-neutral-400">{addedText(member)}</span>}
        {!item.local && onFetch && (
          <button className="px-2 py-1 rounded bg-white/10 hover:bg-white/20" title={t().browse.lightbox.fetchTitle} onClick={() => onFetch(item.path)}>
            {t().browse.lightbox.fetch}
          </button>
        )}
        <button
          className="px-2 py-1 rounded bg-white/10 hover:bg-white/20 disabled:opacity-40"
          disabled={!item.local}
          title={item.local ? t().browse.lightbox.revealTitle : t().browse.lightbox.originalNasOnly}
          onClick={() => run(() => api.reveal(item.path))}
        >
          {t().browse.lightbox.reveal}
        </button>
        <button
          className="px-2 py-1 rounded bg-white/10 hover:bg-white/20 disabled:opacity-40"
          disabled={!item.local}
          title={item.local ? t().browse.lightbox.openTitle : t().browse.lightbox.originalNasOnly}
          onClick={() => run(() => api.openOriginal(item.path))}
        >
          {t().browse.lightbox.open}
        </button>
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={onClose} title="Esc">✕</button>
      </div>
    </div>
  );
}
