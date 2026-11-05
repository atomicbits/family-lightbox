// Nederlandse teksten van het deel browse (F011/S02), met het type van browse.en.ts.
import { count } from "../index";
import type { browse as En } from "./browse.en";

const photos = (n: number) => (n === 1 ? "1 foto" : `${n} foto's`);
const stars = (n: number) => `${n} ${n === 1 ? "ster" : "sterren"}`;

/** De kleuren van de reeksen heten in de events al Nederlands. */
const colors: Record<string, string> = {
  blauw: "blauw",
  amber: "amber",
  groen: "groen",
  roze: "roze",
  paars: "paars",
  turkoois: "turkoois",
  oranje: "oranje",
  grijs: "grijs",
};

export const browse: typeof En = {
  colors,
  video: "video",
  loading: "laden…",
  close: "sluiten",

  undo: {
    stack: (kind, n) => {
      switch (kind) {
        case "make": return `reeks van ${photos(n)} gemaakt`;
        case "add": return `${photos(n)} aan de reeks toegevoegd`;
        case "out": return `${photos(n)} uit de reeks gehaald`;
        case "dissolve": return `reeks van ${photos(n)} opgeheven`;
      }
    },
    hide: (n, hidden) => (hidden ? `${photos(n)} verborgen` : `${photos(n)} niet meer verborgen`),
    skipped: (n, by) => `${photos(n)} intussen ${by.length ? `door ${by.join(" en ")} ` : ""}gewijzigd, niet teruggezet`,
    list: (n, name, add) => (add ? `${photos(n)} toegevoegd aan ${name}` : `${photos(n)} uit ${name} gehaald`),
    color: (color) => `reeks ${color} gemaakt`,
    colorSkipped: "de kleur werd intussen gewijzigd, niet teruggezet",
    undone: (text, skipped) => `ongedaan: ${text}${skipped ? ` · ${skipped}` : ""}`,
    button: "Ongedaan maken",
    title: (text) => `Ongedaan maken: ${text} (⌘Z)`,
    nothing: "niets om ongedaan te maken",
  },

  shortcuts: {
    title: "Sneltoetsen",
    close: (mod) => `Esc of ${mod}? sluit`,
    groups: (mod) => [
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
    ],
  },

  hidden: {
    count: (shown, hidden, stacks) => {
      const parts = [`${shown} ${shown === 1 ? "foto of video" : "foto's en video's"}`];
      if (hidden > 0) parts.push(`${hidden} verborgen`);
      if (stacks > 0) parts.push(`${stacks} ${stacks === 1 ? "reeks" : "reeksen"}`);
      return parts.join(", ");
    },
  },

  live: {
    title: (name) => `Live Photo bij ${name}`,
    label: "Live Photo",
  },

  messages: {
    locale: "nl-BE",
    title: "Meldingen",
    copy: "Kopiëren",
    copied: "Gekopieerd",
    copyTitle: "alle meldingen als tekst, om door te sturen",
    clear: "Wissen",
    none: "geen meldingen sinds de app opende",
    logAt: (path) => `Alles staat ook in ${path}`,
    button: "meldingen",
    unseen: (n) => `${n} nieuwe meldingen`,
  },

  marks: {
    rated: (n, by) => `${stars(n)} (${by})`,
    hearts: (who) => `hartje van ${who.length === 1 ? who[0] : `${who.slice(0, -1).join(", ")} en ${who[who.length - 1]}`}`,
    noMarksYet: "nog geen sterren en hartjes op deze Mac: verbind eerst één keer met de NAS",
    noOwner: "dit toestel heeft nog geen eigenaar",
    change: (n, change) => {
      if ("stars" in change) {
        if (change.stars === 0) return `${photos(n)}: sterren weg`;
        return `${photos(n)}: ${stars(change.stars)}`;
      }
      return change.heart ? `hartje op ${photos(n)}` : `hartje weg bij ${photos(n)}`;
    },
    notSaved: (e) => `niet bewaard op de NAS, teruggezet: ${e}`,
    waiting: (n) => (n === 1 ? "1 wijziging wacht op de NAS" : `${count(n)} wijzigingen wachten op de NAS`),
    waitingTitle: "ze gaan vanzelf naar de NAS zodra die verbonden is",
    differentStars: "de gekozen foto's hebben verschillende sterren",
    removeStars: "sterren weghalen (0)",
    giveStars: (n) => `${stars(n)} (${n})`,
    heartOff: "je hartje weghalen (H)",
    heartAll: "je hartje op alle gekozen foto's (H)",
    heartOn: "een hartje (H)",
    selected: (n) => `${n} foto's`,
  },

  grid: {
    hidden: "verborgen",
    noPreview: "geen preview",
    edited: (label, versions) => `bewerkt in Lightroom: ${label}${versions > 1 ? ` (${versions} versies)` : ""}`,
    nasOnly: "origineel alleen op de NAS",
    fold: "reeks dichtklappen",
    unfold: "reeks openklappen: alle foto's tonen",
  },

  months: {
    revealed: "verborgen foto's worden getoond (klik rechts om ze weer te verbergen)",
    title: (counts, previews, local) => `${counts}; ${previews} previews, ${local} originelen lokaal`,
  },

  lightbox: {
    noPreview: (name) => `geen preview voor ${name}`,
    nasOnly: "het origineel staat alleen op de NAS",
    versions: "versies uit Lightroom",
    chosen: "deze versie toont FamilyLightbox en gaat mee in een export",
    look: "bekijken",
    showVersion: "Deze versie tonen",
    rated: (n, by) => `${stars(n)} van ${by}`,
    hearts: (who) => `hartje van ${who.join(", ")}`,
    fetchTitle: "het origineel van de NAS naar deze Mac halen",
    fetch: "Origineel ophalen",
    revealTitle: "toon het origineel in de Finder",
    reveal: "Toon in Finder",
    originalNasOnly: "origineel alleen op de NAS",
    openTitle: "open het origineel met het standaardprogramma",
    open: "Open origineel",
  },

  menu: {
    color: "Kleur",
    addTo: "Toevoegen aan",
    addToRecent: "Toevoegen aan (recent)",
    otherList: "Andere lijst… ▸",
    otherListTitle: "alle lijsten, met zoeken",
    search: "zoek een lijst",
    noList: "geen lijst met die naam",
  },

  nas: {
    showShares: "toon de gedeelde mappen",
    retry: "Opnieuw",
    connect: "Verbinden",
  },

  sync: {
    fetchTitle: "hoeveel en hoe lang, en dan ophalen of later",
    fetch: "Ophalen…",
    stop: "synchroniseren stoppen",
  },

  app: {
    back: "← Alle foto's",
    backTitle: "terug naar alle foto's",
    missing: (n) => ` · ${n} nog niet op deze Mac`,
    filterCount: (shown, total) => `${shown.toLocaleString("nl-BE")} van ${total.toLocaleString("nl-BE")}`,
    tileSize: "grootte van de miniaturen",
    deviceBroken: "device.json klopt niet: zie instellingen",
    keepsGoing: "je mag de app sluiten: het gaat daarna verder",
    exporting: (done, total) => `exporteren ${done}/${total}`,
    renameTitle: "de bronnen in de instellingen openen",
    importBusy: (text) => `${text}: klik om het importscherm te openen`,
    importIdle: "nieuwe foto's binnenhalen",
    import: "Importeren",
    settings: "Instellingen",
    lrNext: "klik in Lightroom op Bibliotheek → Plug-in-extra's → Lijsten bijwerken",
    allLocal: (next) => `alle originelen staan al op deze Mac: ${next}`,
    fetchDone: (done, next) => `${done}: ${next}`,
    plusHint: "sleep de foto's naar een lijst, of klik rechts: Toevoegen aan",
    open: "Openen",
    removeFrom: (name) => `Uit ${name} halen`,
    fetching: "er worden al originelen opgehaald",
    fetch: (n) => (n === null ? "Originelen ophalen" : `Originelen ophalen (${n})`),
    fetchTitle: "van de NAS naar deze Mac, op hun plaats in de bibliotheek",
    allHere: "ze staan al op deze Mac",
    hide: (n) => (n === null ? "Verbergen" : `Verbergen (${n})`),
    unhide: (n) => (n === null ? "Niet meer verbergen" : `Niet meer verbergen (${n})`),
    makeStack: (n) => `Reeks maken (${n})`,
    addToStack: (n) => `Aan de reeks toevoegen (${n})`,
    takeOut: (n) => (n === null ? "Uit de reeks halen" : `Uit de reeks halen (${n})`),
    dissolve: "Reeks opheffen",
    fold: "Reeks dichtklappen",
    unfold: "Reeks openklappen",
    hideAgain: "Verborgen foto's weer verbergen",
    reveal: (n) => `Verborgen foto's tonen (${n})`,
    nothingHidden: "hier is niets verborgen",
    untilRestart: "tijdelijk, tot de app herstart",
    everythingHere: "alles staat al op deze Mac",
    monthFetchTitle: "naar deze Mac, met eerst wat het inneemt",
  },
  log: {
    marks: "sterren en reeksen",
    lists: "lijsten",
    sources: "bronnen",
    filter: "filter",
    outboxNotSent: (e) => `wachtende wijzigingen niet verzonden: ${e}`,
    renameNotResumed: (e) => `hernoeming niet hervat: ${e}`,
    camerasNotRead: (e) => `camera's niet gelezen: ${e}`,
    sourcesNotRead: (e) => `bronnen niet gelezen: ${e}`,
  },
};
