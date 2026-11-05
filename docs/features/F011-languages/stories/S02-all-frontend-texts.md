# Story S02: All frontend texts in the catalogs

**Feature:** `F011-languages`
**Status:** Built: to try in the app
**Depends on:** S01

## Acceptance criteria

- No UI text left in a component or a helper in `src/`: every one through `t()` from
  `src/i18n`, in four parts (`src/i18n/parts/<part>.en.ts` and `.nl.ts`):
  - `browse`: the window, the grid, the left column, the Lightbox, menus, marks, stacks, undo,
    shortcuts, messages, the header's lines;
  - `settings`: *Instellingen* and its tabs, the NAS, sync, index, previews, updates, Lightroom,
    sources and renames, the first start;
  - `imports`: the import view, its settings and its plans and jobs;
  - `lists`: the lists panel, filters, fetching, exporting, confirmations, versions.
- Numbers through `count`/`decimal`, plurals per language.
- The Dutch texts stay exactly as they are (the tests run in Dutch and pass unchanged).
- Error messages from Rust stay as they come (S03).

## Progress log

### [2026-11-05] Written

Done in parallel by four helpers, one part each, on disjoint files.

### [2026-11-05] Built

- About 590 texts in four parts: `browse` (~150), `settings` (~190), `imports` (~115), `lists`
  (~135), each `<part>.en.ts` and `<part>.nl.ts`; the Dutch copied character for character, so
  every existing test passes unchanged.
- Module-level text tables became lookups at call time (`settingsTabs.tabs()`, the import's step
  names, the filter's kinds, `shortcuts()`), so a switch needs no restart. Exported helpers kept
  their names and signatures; `update.ts` keeps `UP_TO_DATE` for its Dutch test.
- Date locales from the catalog (`nl-BE` / `en-US`) where lists, filters and messages format
  dates. The stack colors stay Dutch as data; `browse.colors` names them in English.
- Small change: the per-month count in *Bronnen* uses `count()` (a space in Dutch, as elsewhere)
  instead of `nl-BE`'s dot.
- Left for S03: Rust's messages passed through, and the `log_message` texts the window writes into
  the messages (they show in *Meldingen*).
- In the plug-in's how-to the English keeps *Lijsten bijwerken*: that is the plug-in's menu item.
- `i18n.test.ts` renders texts of every part in English. `pnpm test` 174, `tsc`, `vite build`
  clean.
