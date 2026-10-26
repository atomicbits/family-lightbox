import { useEffect, useState } from "react";
import type { Item } from "../lib/api";
import { api, previewUrl } from "../lib/api";
import { fileName, takenLabel } from "../lib/months";

type Props = {
  item: Item;
  index: number;
  count: number;
  onClose: () => void;
  onStep: (delta: number) => void;
};

/** Eén foto groot (de 2048 px-preview), met pijltjes, en knoppen naar het origineel. */
export default function Lightbox({ item, index, count, onClose, onStep }: Props) {
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" || e.key === " ") { e.preventDefault(); onClose(); }
      else if (e.key === "ArrowRight") onStep(1);
      else if (e.key === "ArrowLeft") onStep(-1);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, onStep]);

  useEffect(() => setMessage(null), [item.path]);

  const run = (f: () => Promise<void>) => f().catch((e) => setMessage(String(e)));

  return (
    <div className="absolute inset-0 z-20 bg-black/95 text-white flex flex-col" onClick={onClose}>
      {/* The image is positioned absolutely in a box with a definite size: in a grid or flex item
          with an auto height, max-h-full has nothing to resolve against and a tall photo spills
          out at the bottom. */}
      <div className="relative flex-1 min-h-0 m-4" onClick={(e) => e.stopPropagation()}>
        {item.preview ? (
          <img
            src={previewUrl(item.path, false)}
            className="absolute inset-0 w-full h-full object-contain"
            draggable={false}
            alt=""
          />
        ) : (
          <div className="absolute inset-0 grid place-items-center text-neutral-400">
            geen preview voor {fileName(item.path)}
          </div>
        )}
      </div>
      <div
        className="flex items-center gap-4 px-4 py-2 text-sm bg-black/60"
        onClick={(e) => e.stopPropagation()}
      >
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={() => onStep(-1)} disabled={index === 0}>◀</button>
        <span className="tabular-nums text-neutral-400">{index + 1} / {count}</span>
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={() => onStep(1)} disabled={index === count - 1}>▶</button>
        <span className="font-medium">{fileName(item.path)}</span>
        <span className="text-neutral-400">{takenLabel(item.taken)}</span>
        {item.source && <span className="text-neutral-400">{item.source}</span>}
        <span className="flex-1" />
        {message && <span className="text-amber-300">{message}</span>}
        <button
          className="px-2 py-1 rounded bg-white/10 hover:bg-white/20 disabled:opacity-40"
          disabled={!item.local}
          title={item.local ? "toon het origineel in de Finder" : "origineel alleen op de NAS"}
          onClick={() => run(() => api.reveal(item.path))}
        >
          Toon in Finder
        </button>
        <button
          className="px-2 py-1 rounded bg-white/10 hover:bg-white/20 disabled:opacity-40"
          disabled={!item.local}
          title={item.local ? "open het origineel met het standaardprogramma" : "origineel alleen op de NAS"}
          onClick={() => run(() => api.openOriginal(item.path))}
        >
          Open origineel
        </button>
        <button className="px-2 py-1 rounded hover:bg-white/10" onClick={onClose} title="Esc">✕</button>
      </div>
    </div>
  );
}
