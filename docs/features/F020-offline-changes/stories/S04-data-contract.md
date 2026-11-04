# Story S04: The data contract: writing later with the original moment

**Feature:** `F020-offline-changes`
**Status:** Done (2026-11-04)
**Depends on:** S02

## Goal

The contract allows what the outbox does, so the CLI and an older app read the result as before.

## Progress log

### [2026-11-04] Written and done

Data contract 1.5: section 5.2 replaces *if the NAS is unreachable, it refuses* by: a writer MAY
keep a change's lines and append them later, unchanged, with their original `ts`, in order, never
twice, keeping them until the file has every one; a list id taken meanwhile gets the next free one
first; only a registered device. Section 5.5 refers to it for the marks. The events' format doesn't
change.
