import { nasSummary, type Nas } from "../lib/nas";

type Props = { nas: Nas; onOpenSettings: () => void };

const DOT = { green: "bg-green-500", amber: "bg-amber-500", grey: "bg-neutral-400" };

/** NAS state in the header: dot and label (opens the settings panel), plus Verbinden or Opnieuw. */
export default function NasIndicator({ nas, onOpenSettings }: Props) {
  const s = nasSummary(nas.mounts, nas.check, nas.attempts, nas.busy);
  return (
    <span className="flex items-center gap-2">
      <button
        className="flex items-center gap-1.5 text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200"
        title="toon de gedeelde mappen"
        onClick={onOpenSettings}
      >
        <span className={`inline-block w-2 h-2 rounded-full ${DOT[s.tone]}`} />
        NAS {s.label}
      </button>
      {s.action && (
        <button
          className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700"
          onClick={() => nas.connect(true)}
        >
          {s.action === "retry" ? "Opnieuw" : "Verbinden"}
        </button>
      )}
    </span>
  );
}
