type Props = { title: string; text: string; yes: string; onYes: () => void; onNo: () => void };

/** A question before something that is easy to click by mistake (F003, archiving a list). */
export default function ConfirmDialog({ title, text, yes, onYes, onNo }: Props) {
  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" onClick={onNo}>
      <div className="w-[26rem] max-w-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-3" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="text-neutral-600 dark:text-neutral-300">{text}</p>
        <div className="flex justify-end gap-2">
          <button autoFocus className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onNo}>Annuleren</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900" onClick={onYes}>{yes}</button>
        </div>
      </div>
    </div>
  );
}
