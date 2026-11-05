// Nederlandse teksten van het deel settings (F011/S02), met het type van settings.en.ts.
import { count } from "../index";
import type { settings as En } from "./settings.en";

export const settings: typeof En = {
  panel: {
    title: "Instellingen",
    notSaved: "niet bewaard",
    saved: "bewaard",
    thisDevice: "Dit toestel",
    ofOwner: (owner) => ` · van ${owner}`,
    noName: "nog geen naam",
    localCopies: "Lokale kopieën",
    local: { library: "originelen", previews: "previews", lists: "lijsten" },
    about: "Over",
    messages: "Meldingen",
    checkTitle: "kijken of er een nieuwere versie is",
    checking: "Controleren…",
    checkUpdates: "Controleren op updates",
    nas: "NAS",
    defaultsInUse: "De standaardwaarden zijn in gebruik; het bestand wordt pas overschreven als je bewaart.",
    nasName: "Naam van de NAS",
    homeShare: "Thuismap: de gedeelde map met ",
    layout: "Indeling",
    foundHere: "Gevonden op deze Mac",
    saveFirst: "bewaar eerst de instellingen",
    connectTitle: "verbind de gedeelde mappen die nog ontbreken",
    connecting: "Verbinden…",
    connect: "Verbinden",
    syncTitle: "manifesten en previews van de NAS ophalen",
    syncing: "Synchroniseren…",
    sync: "Synchroniseren",
    notConnected: "niet verbonden",
    index: "Index",
    rebuildTitle: "de index opnieuw opbouwen uit de manifesten in de werkkopie",
    updatingIndex: "Index bijwerken…",
    updateIndex: "Index bijwerken",
    previews: "Previews",
    retryTitle: "de mislukte previews opnieuw proberen te maken",
    retry: "Opnieuw proberen",
    failed: (n) => `${n} mislukt:`,
    close: "Sluiten",
    save: "Bewaar",
  },

  tabs: { general: "Algemeen", nas: "NAS", import: "Importeren", library: "Index en previews", sources: "Bronnen" },

  shares: { library: "Originelen", previews: "Previews", lists: "Lijsten", exports: "Exports" },

  sources: {
    renameStopped: (to, error) => `hernoemen naar ${to} gestopt: ${error}`,
    renamed: (to) => `hernoemd naar ${to}`,
    intro:
      "Een bron is de map onder de maand waarin bestanden binnenkwamen. Vink er één of meer aan om ze een nieuwe naam te geven of samen te voegen. " +
      "Alleen de naam van de map verandert; dagmappen blijven zoals ze zijn. Eerst zie je het plan, er verandert dan nog niets.",
    interruptedBefore: "Een hernoeming naar ",
    interruptedAfter: " werd onderbroken.",
    snapshotToday: "snapshot van vandaag",
    continue: "Doorgaan",
    loading: "bronnen laden…",
    chooseFirst: "vink eerst een bron aan",
    newName: "nieuwe naam",
    planning: "Plan maken…",
    makePlan: "Plan maken",
    readingPlan: "het plan lezen van de NAS…",
    backupNote: "Een back-up die bestanden per pad bijhoudt, ziet de verplaatste bestanden als nieuw en kopieert ze opnieuw.",
    month: (month, files, duplicates, conflicts) =>
      `${month}: ${files} bestanden${duplicates ? ` · ${duplicates} dubbel, blijft staan` : ""}${conflicts ? ` · ${conflicts} naamconflict` : ""}`,
    snapshotMade: "er is vandaag een snapshot van de NAS gemaakt",
    run: "Uitvoeren",
    running: (progress) =>
      `${progress} · je mag dit venster sluiten en de app zelfs herstarten: het gaat verder, en onderbroken gaat het vanzelf verder`,
  },

  lightroom: {
    installed: "geïnstalleerd: herstart Lightroom, of kies daar Bestand → Plug-inbeheer → FamilyLightbox → Plug-in opnieuw laden",
    update: "Plugin bijwerken",
    install: "Plugin installeren",
    howTo:
      "In Lightroom Classic: Bibliotheek → Plug-in-extra's → Lijsten bijwerken maakt een collectie per lijst, en zet de sterren over (een hartje is 5 sterren).",
    checking: "nakijken…",
    current: "de plugin van FamilyLightbox is geïnstalleerd",
    different: "er staat een andere versie van de plugin: werk ze bij",
    missing: "de plugin is nog niet geïnstalleerd",
  },

  moveCheck: {
    open: "een verhuis naar een andere gedeelde map nakijken",
    share: "gedeelde map",
    checking: "Nakijken…",
    check: "Nakijken",
    wouldFind: "wat de app daar zou vinden",
    version: (v) => `versie ${v}`,
    before05: "een versie van vóór 0.5",
    mustUpdate: ": moet eerst bijwerken",
  },

  firstStart: {
    title: "De bibliotheek van de NAS ophalen",
    hint: "De eerste keer duurt dit even; de foto's verschijnen zodra de index klaar is.",
    connect: "Verbinden met de NAS",
    notConnected: "niet verbonden: onderaan op Verbinden",
    overview: "Overzicht van de bibliotheek ophalen",
    folders: (done, total) => `${count(done)} van ${count(total)} mappen`,
    searching: (folder) => `mappen zoeken… ${folder}`,
    lists: "Lijsten ophalen",
    index: "De index bouwen",
    indexHint: "de maanden verschijnen zodra hij klaar is",
    counting: "Previews tellen",
    found: (n, folder) => `${count(n)} gevonden… ${folder}`,
    fetch: "Previews ophalen",
    waiting: "wacht op je bevestiging: onderaan op Previews ophalen",
    copying: (done, total, gbDone, gbTotal) => `${count(done)} van ${count(total)} · ${gbDone} van ${gbTotal} GB`,
    failed: "mislukt",
  },

  confirmPreviews: {
    title: "Previews ophalen",
    where:
      "Ze komen in je map Previews op deze Mac. Je kunt intussen gewoon verder bladeren, en stoppen kan altijd met ✕; " +
      "de volgende keer gaat het verder waar het stopte.",
    later: "Later",
    yes: "Ophalen",
  },

  nas: {
    busy: "bezig…",
    notSetUp: "niet ingesteld",
    unreachable: "NAS niet bereikbaar",
    notConnected: "niet verbonden",
    connected: "verbonden",
    partial: "deels verbonden",
    layoutNas: "uit familylightbox.json op de NAS",
    layoutCopy: "uit de kopie van familylightbox.json (de NAS is niet verbonden)",
    layoutSettings: "de gedeelde mappen van deze Mac: er staat geen familylightbox.json op de NAS",
    layoutUnread: "nog niet gelezen",
  },

  access: { readable: "leesbaar", readWrite: "leesbaar en schrijfbaar" },

  registration: {
    registered: "nu geregistreerd op de NAS",
    already: "geregistreerd op de NAS",
    refused: (m) => `niet geregistreerd: ${m}`,
    failed: (m) => `registreren mislukt: ${m}`,
    notTried: (m) => `nog niet geregistreerd: ${m}`,
  },

  update: {
    upToDate: "Je hebt de nieuwste versie",
    percent: (n) => ` ${n} %`,
    installing: (percent) => `Bijwerken…${percent}`,
    available: (v, syncing) => `Versie ${v} beschikbaar${syncing ? ", na de synchronisatie" : ""}`,
    button: "Bijwerken",
    afterSync: "na de synchronisatie",
    installTitle: (v) => `FamilyLightbox ${v} installeren en herstarten`,
    isAvailable: (v) => `Versie ${v} is beschikbaar`,
  },

  sync: {
    lessThanMinute: "minder dan een minuut",
    aboutMinute: "ongeveer een minuut",
    aboutMinutes: (m) => `ongeveer ${m} minuten`,
    confirm: (gb, files, estimate) => `${gb} GB previews ophalen (${count(files)} bestanden)? Dat duurt ${estimate}.`,
    manifestsOf: (done, total) => `manifesten ${count(done)}/${count(total)}`,
    manifests: (done) => `manifesten ${count(done)}`,
    lists: "lijsten…",
    counting: (folder) => `previews tellen… ${folder}`,
    copying: (done, total, gbDone, gbTotal) => `previews ${count(done)}/${count(total)} · ${gbDone}/${gbTotal} GB`,
    ready: (gb) => `${gb} GB previews klaar om op te halen`,
    readyNoSize: "previews klaar om op te halen",
    failed: "synchroniseren mislukt",
    cancelled: "geannuleerd",
    skipped: (n) => `${count(n)} overgeslagen`,
    manifestsCopied: (n) => `${count(n)} ${n === 1 ? "manifest" : "manifesten"}`,
    listsCopied: "lijsten",
    previewsCopied: (n) => `${count(n)} previews`,
    updated: (parts) => `bijgewerkt: ${parts}`,
  },

  previews: {
    allHave: "alle foto's en video's hebben een preview",
    missing: (n) => `${count(n)} ${n === 1 ? "foto of video" : "foto's en video's"} zonder preview`,
    paused: (progress) => `Previews gepauzeerd${progress}`,
    making: (progress) => `Previews maken${progress}`,
    resume: "Hervatten",
    pause: "Pauzeren",
    notOnNas: (n) => `${count(n)} ${n === 1 ? "preview staat" : "previews staan"} nog niet op de NAS`,
  },

  indexer: {
    updating: "index bijwerken…",
    error: (e) => `index: ${e}`,
    broken: (n, folders) => `index: ${n} ${n === 1 ? "manifest onleesbaar" : "manifesten onleesbaar"}: ${folders}`,
    outcome: (items, manifests, secs) => `${count(items)} items uit ${count(manifests)} manifesten · ${secs} s`,
  },

  renamer: {
    progress: (done, total) => `hernoemen: maand ${done} van ${total}`,
    busy: "hernoemen…",
  },

  rename: {
    nothing: "niets te hernoemen",
    files: (files, months, gb) => `${count(files)} bestanden in ${count(months)} ${months === 1 ? "maand" : "maanden"} (${gb} GB)`,
    sidecars: (n) => `${count(n)} sidecars`,
    previews: (n) => `${count(n)} previews`,
    duplicates: (n) => `${count(n)} dubbel, blijven staan`,
    conflicts: (n) => `${count(n)} naamconflicten`,
  },
};
