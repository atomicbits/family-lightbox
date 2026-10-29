import { useEffect } from "react";
import { modKey, shortcuts } from "../lib/shortcuts";

/** The keyboard shortcuts on a translucent overlay; Esc, ⌘? or a click closes it. */
export default function ShortcutsOverlay({ onClose }: { onClose: () => void }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.preventDefault(); e.stopImmediatePropagation(); onClose(); } };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onClose]);
  const mod = modKey();
  return (
    <div className="absolute inset-0 z-50 bg-black/60 backdrop-blur-sm grid place-items-center p-8" onClick={onClose}>
      <div className="max-w-3xl w-full rounded-xl bg-neutral-900/85 text-white shadow-2xl p-6" onClick={onClose}>
        <div className="flex items-baseline mb-4">
          <h2 className="text-lg font-semibold flex-1">Sneltoetsen</h2>
          <span className="text-xs text-white/50">Esc of {mod}? sluit</span>
        </div>
        <div className="grid grid-cols-2 gap-x-10 gap-y-5 text-sm">
          {shortcuts(mod).map((g) => (
            <section key={g.title}>
              <h3 className="text-xs uppercase tracking-wide text-white/50 mb-1.5">{g.title}</h3>
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
                {g.items.map((s) => (
                  <div key={s.keys} className="contents">
                    <dt>
                      <kbd className="rounded bg-white/15 px-1.5 py-0.5 font-mono text-xs whitespace-nowrap">{s.keys}</kbd>
                    </dt>
                    <dd className="text-white/80">{s.what}</dd>
                  </div>
                ))}
              </dl>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}
