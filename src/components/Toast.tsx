/** The message at the bottom after a change, with Ongedaan maken (F013/S04). */
export default function Toast({ text, onUndo, onClose }: { text: string; onUndo: (() => void) | null; onClose: () => void }) {
  return (
    <div className="absolute bottom-12 left-1/2 -translate-x-1/2 z-30 flex items-center gap-3 rounded-lg bg-neutral-900/95 dark:bg-neutral-800/95 text-white text-sm shadow-xl px-4 py-2">
      <span>{text}</span>
      {onUndo && (
        <button className="px-2 py-0.5 rounded bg-white/15 hover:bg-white/25 font-medium" onClick={onUndo} title="⌘Z">
          Ongedaan maken
        </button>
      )}
      <button className="text-white/60 hover:text-white" onClick={onClose} title="sluiten">✕</button>
    </div>
  );
}
