// What a new Mac shows while it has no index yet: the steps of the first sync and how far each
// got, instead of one line that doesn't move (F007, on Peter's request when Lynn's Mac started).
import { gb, type SyncState } from "./sync";
import { t } from "../i18n";
import type { NasSummary } from "./nas";

export type StepState = "done" | "busy" | "todo" | "waiting" | "problem";
export type Step = { label: string; state: StepState; detail: string; fraction: number | null };

const STAGE: Record<string, number> = { manifests: 1, lists: 2, counting: 3, awaiting_confirmation: 3, copying: 3, done: 4 };

/** The steps from the NAS connection to the previews, with the sync's and the index's state. */
export function firstStartSteps(sync: SyncState | null, indexRunning: boolean, nas: NasSummary["state"]): Step[] {
  const phase = sync?.phase ?? "idle";
  const stage = STAGE[phase] ?? 0;
  const connected = nas === "connected" || nas === "partial";
  const step = (label: string, state: StepState, detail = "", fraction: number | null = null): Step => ({ label, state, detail, fraction });
  const at = (n: number): StepState => (stage > n ? "done" : stage === n && sync?.running ? "busy" : "todo");

  const x = t().settings.firstStart;

  const steps = [
    nas === "busy"
      ? step(x.connect, "busy")
      : connected
        ? step(x.connect, "done")
        : step(x.connect, "problem", x.notConnected),
    step(
      x.overview,
      at(1),
      phase === "manifests" ? (sync!.files_total > 0 ? x.folders(sync!.files_done, sync!.files_total) : x.searching(sync!.folder).trimEnd()) : "",
      phase === "manifests" && sync!.files_total > 0 ? sync!.files_done / sync!.files_total : null,
    ),
    step(x.lists, at(2)),
    step(x.index, indexRunning ? "busy" : stage >= 3 ? "done" : "todo", indexRunning ? x.indexHint : ""),
  ];
  // the previews come after the index; while it is built they may already be counted
  if (phase === "counting") steps.push(step(x.counting, "busy", x.found(sync!.files_done, sync!.folder.split("/").slice(0, 2).join("/")).trimEnd()));
  if (phase === "awaiting_confirmation") steps.push(step(x.fetch, "waiting", x.waiting));
  if (phase === "copying") {
    const frac = sync!.bytes_total > 0 ? sync!.bytes_done / sync!.bytes_total : null;
    steps.push(step(x.fetch, "busy", x.copying(sync!.files_done, sync!.files_total, gb(sync!.bytes_done), gb(sync!.bytes_total)), frac));
  }
  if (phase === "failed") {
    const i = steps.findIndex((s) => s.state !== "done");
    const where = i < 0 ? steps.length - 1 : i;
    steps[where] = { ...steps[where], state: "problem", detail: sync!.message ?? x.failed, fraction: null };
  }
  return steps;
}
