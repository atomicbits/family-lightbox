// The keyboard shortcuts, for the overlay that ⌘? (Ctrl+? elsewhere) opens.

/** ⌘ on a Mac, Ctrl elsewhere. */
export function modKey(platform: string = typeof navigator === "undefined" ? "" : navigator.platform): string {
  return /mac/i.test(platform) ? "⌘" : "Ctrl";
}

/** Whether a key press opens the shortcuts: ⌘? or Ctrl+? (? is shift-/). */
export function isShortcutsKey(e: { key: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }): boolean {
  return (e.metaKey || e.ctrlKey) && (e.key === "?" || (e.shiftKey && e.key === "/"));
}

export type Shortcut = { keys: string; what: string };
export type ShortcutGroup = { title: string; items: Shortcut[] };

export function shortcuts(mod: string): ShortcutGroup[] {
  return [
    {
      title: "Bladeren",
      items: [
        { keys: "← → ↑ ↓", what: "volgende foto kiezen" },
        { keys: "Spatie of Enter", what: "foto openen (ook: dubbelklik)" },
        { keys: "Esc", what: "foto sluiten, keuze wissen" },
      ],
    },
    {
      title: "Kiezen",
      items: [
        { keys: "Shift-klik", what: "een reeks kiezen, ook over maanden" },
        { keys: "Shift + pijl", what: "de reeks uitbreiden" },
        { keys: `${mod}-klik`, what: "een foto aan de keuze toevoegen of eruit halen" },
      ],
    },
    {
      title: "Sterren en hartje",
      items: [
        { keys: "1 – 5", what: "sterren geven" },
        { keys: "0", what: "sterren weghalen" },
        { keys: "H", what: "je hartje aan of uit" },
      ],
    },
    {
      title: "Lijsten en verbergen",
      items: [
        { keys: "slepen", what: "foto's naar een lijst in de zijbalk" },
        { keys: "rechtsklik", what: "toevoegen aan een lijst, verbergen" },
        { keys: "⌫", what: "in een lijst: de foto's eruit halen" },
        { keys: "rechtsklik op maand of jaar", what: "verborgen foto's even tonen" },
      ],
    },
    {
      title: "Algemeen",
      items: [
        { keys: `${mod}Z`, what: "ongedaan maken" },
        { keys: `${mod}?`, what: "dit overzicht" },
      ],
    },
  ];
}
