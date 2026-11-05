import type { Sync } from "../lib/sync";
import { Spinner } from "./Progress";
import { t } from "../i18n";

type Props = { sync: Sync; onConfirm: () => void };

/** The background sync next to the NAS indicator: progress with ✕, the confirmation, or what came in. */
export default function SyncLine({ sync, onConfirm }: Props) {
  const l = sync.line;
  if (!l) return null;
  const tone = l.tone === "amber" ? "text-amber-600" : "text-neutral-500";
  return (
    <span className={`flex items-center gap-1.5 ${tone}`}>
      {l.confirm ? (
        <>
          <span className="tabular-nums">{l.text}</span>
          <button className="text-blue-600 hover:underline" onClick={onConfirm} title={t().browse.sync.fetchTitle}>
            {t().browse.sync.fetch}
          </button>
        </>
      ) : (
        <>
          {sync.state?.running && <Spinner />}
          <span className="tabular-nums">{l.text}</span>
        </>
      )}
      {l.cancellable && (
        <button className="px-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" title={t().browse.sync.stop} onClick={() => sync.cancel()}>
          ✕
        </button>
      )}
    </span>
  );
}
