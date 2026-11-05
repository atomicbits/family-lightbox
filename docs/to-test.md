# To test after v0.7.0

What is built and waits for Peter's check in the real app. Tick a line when it works; what doesn't
goes back to its story's progress log. Remove a section once its stories are *Done*.

## On Peter's Mac

### Lightroom's edits (F025, S02–S05)

- [ ] *Instellingen → Lightroom*: the plug-in says it can be updated (0.2); install it, restart
      Lightroom (or *Plug-in Manager → Reload*).
- [ ] Edit a photo in a list's collection (a crop, exposure) and make a virtual copy with a name.
- [ ] *Bibliotheek → Plug-in-extra's → Lijsten bijwerken*: no Lua error; the dialog says how many
      edited versions went to FamilyLightbox. Click again: none.
- [ ] If nothing is rendered for a photo with only exposure changes: `FamilyLightbox.log` in
      `~/Documents/LrClassicLogs/` says `hasDevelopAdjustments onbekend` (then only crops count).
- [ ] Within 30 s in FamilyLightbox: the tile shows the edited version with *LR*; on the NAS
      `Previews/…/<file>.lr.jpg`, `.lr.full.jpg` and `.lr-<copy>.jpg`; on this Mac only the small
      ones.
- [ ] Lightbox: buttons *Origineel*, *Bewerkt*, the copy's name; *Deze versie tonen* changes the
      tile, and Lynn's Mac shows the same after its sync.
- [ ] Export the list (*Exporteren…*): the edited photo comes out as edited at full size;
      `export.json` has `lightroom_edits`.
- [ ] Undo the edit in Lightroom (*Reset*), *Lijsten bijwerken*: FamilyLightbox shows the original
      again.

### Import from a card (F018/S07)

- [ ] Put in a camera card: it shows in *Importeren* within a few seconds, with *alleen kopiëren*.
- [ ] Import it: the photos land on the NAS, nothing on the card is moved or deleted.
- [ ] *Uitwerpen* ejects the card.

### Whole months and years (F009/S04)

- [ ] Right-click a month in the left column: *Originelen ophalen (n)*; the plan shows the size;
      fetch; the *NAS* marks go.
- [ ] Right-click a year: the same for all its months; above 5 GB the warning in amber.

### The welcome with one shared folder (F019/S02)

Needs an empty shared folder on the NAS and a Mac (or user account) without FamilyLightbox's
settings.

- [ ] Step 1 asks the NAS and *Gedeelde map van de bibliotheek*.
- [ ] An empty folder: *Hier een nieuwe bibliotheek inrichten* makes `Library`, `Previews`,
      `Lists`, `Exports` and `familylightbox.json`, then goes on to *who are you*.
- [ ] A folder with this library's file: goes straight on.
- [ ] A folder without write access: the welcome says so.

### Languages (F011/S01) — not in 0.7.0: the next release, or `pnpm tauri dev`

- [ ] *Instellingen → Algemeen → Taal*: *English* turns the whole window English at once (only
      messages from the Rust core stay Dutch until S03); *Zoals de Mac* gives Dutch back; the
      choice stays after a restart.
- [ ] Walk through in English: the grid and its menus, the Lightbox, lists (fetch, export,
      archive), import, every settings tab; note any text that reads oddly.

## On Lynn's Mac

After the update to 0.7.0 arrives (*Bijwerken* in the header).

- [ ] The update installs and the app restarts (F004/S04).
- [ ] The source renames done on Peter's Mac are applied: the iPhone folders have their new names
      in her local copies, nothing is fetched again (F022/S04).
- [ ] A list Peter changes shows the change on her Mac after the sync, and the other way round
      (F003/S04).
- [ ] The layout is read from the NAS's file; *Instellingen* shows it (F019/S03).
- [ ] First-start steps where they still apply: the NAS first, then who (F006/S06), the local
      folder (F006/S07), one sign-in for the shares (F006/S03), the device registered (F006/S04),
      the connection indicator (F006/S05).
- [ ] A version chosen on Peter's Mac (F025) shows the same on hers.
