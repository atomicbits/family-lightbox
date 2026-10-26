# Story S01: One continuous grid across months and years

**Feature:** `F001-browse-library`
**Status:** Planned
**Depends on:** —

## Goal

Scrolling the grid doesn't stop at the end of a month: it carries on into the next month and
year, across the whole library. A subtle header marks where each month begins. The month list on
the left becomes a way to jump, and it follows along while scrolling.

## Acceptance criteria

- **One grid for the whole library**: every month from `months()` is a section. The sections are
  in the same order as the month list, newest month first. Within a section the order stays
  oldest to newest by capture time, as today. Folders without a month (`2012/scans Peter`) are
  sections too, in the same place as in the month list.
- **Month headers**: at the start of each section a header row in small, muted text, the same
  width as the rows (centred with them): "mei 2026" on the left, the count ("1558") on the right.
  It sits in the grid, not on top of it. It scrolls with the photos and is not sticky.
- **Top bar**: the title shows the month whose rows are at the top of the view ("mei 2026 ·
  1558 items"), and "· n / 1558" when the selection is in that month.
- **Month list**: clicking a month scrolls its header to the top, without changing the selection.
  While scrolling, the highlighted month follows the top of the view, and a year that is folded
  shut stays shut. The active month is scrolled into view in the list when it would be hidden.
- **Light with 124 000 items**: only the sections that have rows on screen, or within the
  virtualizer's overscan, are fetched with `items(month)`. Each month is fetched once (cached),
  and never twice at the same time. Until a month is loaded its rows show empty tiles of the
  right size, so the scrollbar and positions don't jump. The total height is known from the
  counts in `months()` alone.
- **Selection across months**: the selection is a month plus a position.
  - Left and right step across the boundary: right on the last photo of a month goes to the first
    photo of the month below it, and left on the first photo goes to the last of the month above.
  - Up and down move a row. From the first row of a month, up goes to the same column in the last
    row of the month above, or to its last photo if that row is shorter. From the last row, down
    goes to the same column in the first row of the month below, or to its last photo if that row
    is shorter.
  - At the very top and the very bottom of the library, the selection stays where it is.
  - The selected tile stays in view.
- **Lightbox**: the arrows step across months as well. The photo of a month that isn't loaded
  yet shows as soon as it is.
- **Column count and tile size**: these keep working as now. A change of either rebuilds the row
  layout and keeps the selected tile in view, or the top month at the top when nothing is
  selected.
- **Tests** (`pnpm test`, pure functions):
  - the layout from counts and columns: header and row offsets per section, total rows, the
    section and position of a given row, and the row of a given month;
  - the moves across boundaries, all cases above, including a month with a single photo, a
    short last row, and the very first and last photo;
  - which months are needed for a range of visible rows.
  - Existing tests still pass, typecheck and build clean.
  - **By hand**: scroll from mei 2026 into april 2026 and further, check the headers, the
    month list following along, and jumping. Run the arrows over a month boundary, and the
    lightbox over a boundary. Scroll quickly through a few years and check it stays smooth and
    loads months as they appear.

## How

- `src/lib/timeline.ts` (new, pure):
  - `buildLayout(sections: {month, count}[], cols)` → an array of row descriptors
    (`{kind: "header", section}` or `{kind: "tiles", section, row}`) plus, per section, the
    index of its header row;
  - `locate(rowIndex)`;
  - `moveAcross(sel, move, cols, counts)` → the new `{section, index}`, building on
    `moveInGrid` from `lib/grid.ts` for moves within a section;
  - `monthsInRange(layout, first, last)`.
- `Grid.tsx`:
  - takes the sections, a lookup `itemsOf(month) → Item[] | undefined`, the selection, and
    `onNeed(months)`;
  - virtualizes the row descriptors, with `estimateSize` by kind (header about 32 px, tiles
    `tile + gap`);
  - calls `onNeed` for the months in the visible range plus overscan, and `onTopMonth` when the
    top row's section changes;
  - exposes `scrollToMonth`.
- `App.tsx`:
  - a cache `Map<month, Item[]>` with an in-flight set;
  - the selection as `{month, index}`;
  - the top month for the title and the month list;
  - the month list click calls `scrollToMonth`.
- `MonthList.tsx`: highlights the top month, and scrolls it into view in the list.
- No Rust changes: `months()` already gives the counts with the same filter as `items()` (photos
  and videos, grouped by `coalesce(month, folder)`).
- Deliberately not done:
  - a scrubber or date bar along the scrollbar;
  - sticky headers;
  - reversing the order to oldest first (possible later, as an option);
  - loading the 2048 px preview for big tiles.

## Files affected

- `src/lib/timeline.ts` (new), `src/lib/timeline.test.ts` (new).
- `src/components/Grid.tsx`, `src/components/MonthList.tsx`, `src/components/Lightbox.tsx` (if
  needed), `src/App.tsx`.
- `docs/features/F001-browse-library/plan.md` (stories table).

## Progress log

### [2026-10-26] Written
