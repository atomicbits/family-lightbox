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
