// What a new Mac shows while it has no index yet: the steps of the first sync and how far each
// got, instead of one line that doesn't move (F007, on Peter's request when Lynn's Mac started).
import { count, gb, type SyncState } from "./sync";
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

  const steps = [
    nas === "busy"
      ? step("Verbinden met de NAS", "busy")
      : connected
        ? step("Verbinden met de NAS", "done")
        : step("Verbinden met de NAS", "problem", "niet verbonden: onderaan op Verbinden"),
    step(
      "Overzicht van de bibliotheek ophalen",
      at(1),
      phase === "manifests" ? (sync!.files_total > 0 ? `${count(sync!.files_done)} van ${count(sync!.files_total)} mappen` : `mappen zoeken… ${sync!.folder}`.trimEnd()) : "",
      phase === "manifests" && sync!.files_total > 0 ? sync!.files_done / sync!.files_total : null,
    ),
    step("Lijsten ophalen", at(2)),
    step("De index bouwen", indexRunning ? "busy" : stage >= 3 ? "done" : "todo", indexRunning ? "de maanden verschijnen zodra hij klaar is" : ""),
  ];
  // the previews come after the index; while it is built they may already be counted
  if (phase === "counting") steps.push(step("Previews tellen", "busy", `${count(sync!.files_done)} gevonden… ${sync!.folder.split("/").slice(0, 2).join("/")}`.trimEnd()));
  if (phase === "awaiting_confirmation") steps.push(step("Previews ophalen", "waiting", "wacht op je bevestiging: onderaan op Previews ophalen"));
  if (phase === "copying") {
    const frac = sync!.bytes_total > 0 ? sync!.bytes_done / sync!.bytes_total : null;
    steps.push(step("Previews ophalen", "busy", `${count(sync!.files_done)} van ${count(sync!.files_total)} · ${gb(sync!.bytes_done)} van ${gb(sync!.bytes_total)} GB`, frac));
  }
  if (phase === "failed") {
    const i = steps.findIndex((s) => s.state !== "done");
    const where = i < 0 ? steps.length - 1 : i;
    steps[where] = { ...steps[where], state: "problem", detail: sync!.message ?? "mislukt", fraction: null };
  }
  return steps;
}
