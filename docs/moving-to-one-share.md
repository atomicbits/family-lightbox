# Moving a library to one share

How a library spread over several shares (`Library`, `Previews`, `Lists`, `Exports`) moves into
one shared folder with four subfolders (F019/S04). The paths inside the four don't change, so
nothing in the manifests, previews or lists is rewritten; only where the four roots are changes,
in `familylightbox.json` (data contract 2).

What else reads the shares (a command-line tool, scripts on the NAS, a copy to the cloud) moves at
the same time, by its own instructions.

## Before

1. **Every device on FamilyLightbox 0.5 or later.** An older app doesn't read
   `familylightbox.json` and would go on writing its lists to the old share. In the settings,
   under *NAS*, *een verhuis … nakijken* with the current home share (`Lists`) lists every device
   with the version it last started with; each must be 0.5 or later.
2. **A snapshot or backup** of the four shares, as before any large change on the NAS.
3. **Nobody imports or syncs** during the move: quit the app on every device.

## On the NAS

4. Create the new shared folder (say `Photos`) with the same users and rights as the old shares:
   read and write for everyone who uses the app.
5. Copy the contents of each old share into a subfolder of the new one: `Library`, `Previews`,
   `Lists`, `Exports`, keeping times (`rsync -aH`). On a NAS where every shared folder is its own
   file-system volume (Btrfs on Synology), moving between shares is a copy anyway; copying leaves
   the old shares as they were, so going back stays possible until they are removed. Compare
   afterwards (`rsync -n --itemize-changes`: nothing left), and check a sample of hashes against
   the manifests.
   A copy that sets file modes can drop the rights the files inherited: apply the new share's
   inheritance again from the top, and set again any rights a folder had of its own (and only then
   let what is below it inherit, so that folder keeps them).
6. Make the old shares read-only, so nothing can be written to the old places by mistake.
7. Write the new `familylightbox.json`, with `home` naming the new share, in the root of the new
   share **and** in the root of the old home share (which is empty now apart from it):

   ```json
   {
     "contract_version": "1.4",
     "name": "Our photos",
     "home": "Photos",
     "library":  { "share": "Photos", "path": "Library" },
     "previews": { "share": "Photos", "path": "Previews" },
     "lists":    { "share": "Photos", "path": "Lists" },
     "exports":  { "share": "Photos", "path": "Exports" }
   }
   ```

## Each device

8. Open the app. It reads the file in its old home share, connects the new share, finds the file
   there too, and makes the new share its home: nothing to set by hand.
9. Check, in the settings: the layout comes *uit familylightbox.json op de NAS*, every location is
   in the new share and found, the home share is the new one. A sync finds nothing new; the lists,
   stars and hearts are as before; the photos are all there.

## After

10. When every device has moved its home (its settings say so), the old shares can go: first
    unmounted on every Mac, then removed on the NAS, once a snapshot of the new share exists.

## Checking before step 4

*Een verhuis … nakijken* with the new share's name, after steps 4–7, says per location what the
app would find there (the year folders of the library, `devices.json` in the lists) and which
devices still have an older app, without using any of it.
