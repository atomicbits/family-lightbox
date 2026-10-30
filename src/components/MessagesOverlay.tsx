import { useEffect, useState } from "react";
import { messagesText, timeLabel, type Messages } from "../lib/messages";

const LEVEL = { error: "text-red-400", warn: "text-amber-300", info: "text-white/60" };

/** The warnings and errors of this session on a translucent overlay (F016/S02). */
export default function MessagesOverlay({ messages, onClose }: { messages: Messages; onClose: () => void }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.preventDefault(); e.stopImmediatePropagation(); onClose(); } };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onClose]);
  const [copied, setCopied] = useState(false);
  const copy = () =>
    navigator.clipboard.writeText(messagesText(messages.list)).then(() => { setCopied(true); setTimeout(() => setCopied(false), 1500); }).catch(() => {});
  return (
    <div className="absolute inset-0 z-50 bg-black/60 backdrop-blur-sm grid place-items-center p-8" onClick={onClose}>
      <div className="max-w-4xl w-full max-h-full flex flex-col rounded-xl bg-neutral-900/90 text-white shadow-2xl p-5" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-baseline gap-3 mb-3">
          <h2 className="text-lg font-semibold flex-1">Meldingen</h2>
          <button className="px-2 py-0.5 rounded bg-white/10 hover:bg-white/20 text-sm disabled:opacity-40" disabled={messages.list.length === 0} onClick={copy} title="alle meldingen als tekst, om door te sturen">
            {copied ? "Gekopieerd" : "Kopiëren"}
          </button>
          <button className="px-2 py-0.5 rounded bg-white/10 hover:bg-white/20 text-sm disabled:opacity-40" disabled={messages.list.length === 0} onClick={messages.clear}>
            Wissen
          </button>
          <button className="text-white/60 hover:text-white" onClick={onClose} title="Esc">✕</button>
        </div>
        {messages.list.length === 0 ? (
          <div className="text-white/50 text-sm py-6 text-center">geen meldingen sinds de app opende</div>
        ) : (
          <ul className="overflow-auto min-h-0 text-xs font-mono space-y-1 select-text cursor-text">
            {messages.list.map((m, i) => (
              <li key={`${m.at}-${i}`} className="grid grid-cols-[auto_auto_1fr] gap-3">
                <span className="text-white/40 whitespace-nowrap">{timeLabel(m.at)}</span>
                <span className={`${LEVEL[m.level]} whitespace-nowrap`}>{m.source}</span>
                <span className="text-white/80 break-all">{m.text}</span>
              </li>
            ))}
          </ul>
        )}
        {messages.logPath && <div className="mt-3 text-xs text-white/40">Alles staat ook in {messages.logPath}</div>}
      </div>
    </div>
  );
}
