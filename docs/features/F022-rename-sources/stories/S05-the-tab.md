# Story S05: The tab *Bronnen* in the settings

**Feature:** `F022-rename-sources`
**Status:** Done (2026-11-04)
**Depends on:** S01, S03

## Progress log

### [2026-11-04] Written and built

- `SourcesTab.tsx`: every source with its count and when last used; tick one or more, a new name,
  *Plan maken* (read from the NAS), the plan per month, the confirmation *er is vandaag een
  snapshot van de NAS gemaakt*, *Uitvoeren* with its progress; an interrupted rename with
  *Doorgaan*. `rename.ts`: the plan's types, `renameSummary`, `toggleSource`.
- `pnpm test` 156 passed (2 new), `tsc` and `vite build` clean.

### [2026-11-04] Used

Peter used it for the first rename. The progress moved to the app and the footer (see S03).

### [2026-11-04] Peter's remarks

- The plan said *Drive laadt 9,5 GB opnieuw op*: true for this library's copy to Drive, not for
  every user, who may have no backup or another one. The summary now only says what moves; below
  it the tab says in general that a backup keeping files by path sees them as new and copies them
  again.
- The progress shows in the footer while a rename runs, with its small bar and a percentage; a
  click opens the settings on *Bronnen*. Every progress bar (`Progress`, the footer's `MiniBar`)
  shows its percentage once the amount is known.
- `pnpm test` 158 passed (1 new), `tsc` and `vite build` clean.
