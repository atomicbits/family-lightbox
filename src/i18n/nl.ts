// Nederlands (F011): elke tekst van `en.ts`, met hetzelfde type.
import type { Catalog } from "./en";

const list = (xs: string[]) => (xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} en ${xs[xs.length - 1]}`);

export const nl: Catalog = {
  months: ["januari", "februari", "maart", "april", "mei", "juni", "juli", "augustus", "september", "oktober", "november", "december"],

  language: {
    title: "Taal",
    system: "Zoals de Mac",
    hint: "Verandert meteen; zonder keuze hier volgt de app de taal van de Mac.",
  },

  device: {
    nameRule: "2 tot 32 tekens: kleine letters, cijfers en '-', beginnend met een letter",
    ownerRule: "2 tot 32 tekens: kleine letters en cijfers, beginnend met een letter",
    freeTB: (tb) => `${tb} TB vrij`,
    freeGB: (gb) => `${gb} GB vrij`,
    hostEmpty: "de naam van de NAS is leeg",
    hostScheme: (scheme) => `geef alleen de naam van de NAS, zonder '${scheme}'`,
    hostChars: "de naam van de NAS mag geen '/' of spaties bevatten",
    shareEmpty: "de naam van een gedeelde map is leeg",
    shareSlash: (share) => `'${share}': een gedeelde map mag geen '/' bevatten`,
    pathInside: (path) => `'${path}': het pad moet binnen de gedeelde map blijven, met '/'`,
    setUp: (share, make, existing) => {
      const parts: string[] = [];
      if (make.length) parts.push(`Maakt in '${share}' de map${make.length > 1 ? "pen" : ""} ${list(make)}`);
      if (existing.length) parts.push(`${make.length ? "gebruikt" : "Gebruikt"} ${list(existing)} die er al ${existing.length > 1 ? "zijn" : "is"}`);
      return `${parts.join(" en ")}, en schrijft er familylightbox.json bij. Er wordt niets overschreven.`;
    },
  },

  welcome: {
    title: "Welkom bij FamilyLightbox",
    looking: "even kijken…",
    locations: { library: "originelen", previews: "previews", lists: "lijsten", exports: "exports" },
    nasIntro: "Eerst je NAS en de gedeelde map waar de foto's en de lijsten staan. Bij Verbinden vraagt macOS je account op de NAS.",
    host: "Naam of adres van de NAS",
    hostPlaceholder: "bv. mijnnas of 192.168.1.20",
    hostMissing: "geef de naam of het adres van je NAS",
    home: "Gedeelde map van de bibliotheek",
    homePlaceholder: "bv. FamilyLightbox",
    homeHint: "daar staat de bibliotheek, of daar wordt een nieuwe ingericht",
    homeMissing: "geef de gedeelde map van de bibliotheek",
    notConnected: (share) => `de gedeelde map '${share}' is niet verbonden: kijk de naam van de NAS, de map en je account na, en probeer opnieuw`,
    withoutNas: "zonder NAS verder",
    connecting: "verbinden…",
    connect: "Verbinden",
    noLibrary: "Er staat nog geen bibliotheek in",
    libraryName: "Naam van de bibliotheek",
    libraryNamePlaceholder: "bv. Onze foto's",
    pointOpen: "de bibliotheek staat elders: bestaande mappen aanwijzen",
    pointHint: "Per onderdeel een gedeelde map en een map daarin (leeg: de gedeelde map zelf). Dit komt in familylightbox.json in",
    pathPlaceholder: "map",
    pointClose: "toch een nieuwe bibliotheek",
    back: "‹ NAS",
    saving: "bewaren…",
    point: "Mappen aanwijzen",
    settingUp: "inrichten…",
    setUp: "Hier een nieuwe bibliotheek inrichten",
    folderTitle: "Map voor de lokale kopieën",
    folderHint: "previews, lijsten en de foto's die je lokaal houdt",
    choose: "Kies…",
    folderIntro: "Kies waar FamilyLightbox op deze laptop zijn kopieën bewaart. Daarna start de app opnieuw.",
    save: "Bewaar",
    connectedAs: "Verbonden als",
    whoIntro: "Wie gebruikt deze laptop? De lijsten, sterren en hartjes onthouden wie wat koos.",
    knownOnNas: "al bekend op de NAS",
    devicesOf: (devices) => `toestellen: ${devices.join(", ")}`,
    yourName: "Jouw naam",
    ownerPlaceholder: "korte naam, kleine letters",
    deviceName: "Naam van deze laptop",
    nameTaken: (owner) => `deze naam is op de NAS al van ${owner}`,
  },
};
