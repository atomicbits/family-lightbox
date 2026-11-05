// English, the reference catalog (F011): `nl.ts` has this type, so a text missing there fails
// `tsc`. A text with numbers or names in it is a function.

import { browse } from "./parts/browse.en";
import { imports } from "./parts/imports.en";
import { lists } from "./parts/lists.en";
import { settings } from "./parts/settings.en";

const list = (xs: string[]) => (xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} and ${xs[xs.length - 1]}`);

export const en = {
  months: ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],

  language: {
    title: "Language",
    system: "As the Mac",
    hint: "Changes at once; the Mac's language is used unless you choose one here.",
  },

  device: {
    nameRule: "2 to 32 characters: lowercase letters, digits and '-', starting with a letter",
    ownerRule: "2 to 32 characters: lowercase letters and digits, starting with a letter",
    freeTB: (tb: string) => `${tb} TB free`,
    freeGB: (gb: number) => `${gb} GB free`,
    hostEmpty: "the NAS's name is empty",
    hostScheme: (scheme: string) => `give only the NAS's name, without '${scheme}'`,
    hostChars: "the NAS's name can't contain '/' or spaces",
    shareEmpty: "a shared folder's name is empty",
    shareSlash: (share: string) => `'${share}': a shared folder can't contain '/'`,
    pathInside: (path: string) => `'${path}': the path must stay inside the shared folder, with '/'`,
    setUp: (share: string, make: string[], existing: string[]) => {
      const parts: string[] = [];
      if (make.length) parts.push(`Makes the folder${make.length > 1 ? "s" : ""} ${list(make)} in '${share}'`);
      if (existing.length) parts.push(`${make.length ? "uses" : "Uses"} ${list(existing)}, which ${existing.length > 1 ? "are" : "is"} there already`);
      return `${parts.join(" and ")}, and writes familylightbox.json next to them. Nothing is overwritten.`;
    },
  },

  browse,
  settings,
  imports,
  lists,

  welcome: {
    title: "Welcome to FamilyLightbox",
    looking: "one moment…",
    locations: { library: "originals", previews: "previews", lists: "lists", exports: "exports" },
    nasIntro: "First your NAS and the shared folder where the photos and the lists are. When you click Connect, macOS asks for your account on the NAS.",
    host: "Name or address of the NAS",
    hostPlaceholder: "e.g. mynas or 192.168.1.20",
    hostMissing: "give the name or address of your NAS",
    home: "The library's shared folder",
    homePlaceholder: "e.g. FamilyLightbox",
    homeHint: "the library is there, or a new one is set up there",
    homeMissing: "give the library's shared folder",
    notConnected: (share: string) => `the shared folder '${share}' isn't connected: check the NAS's name, the folder and your account, and try again`,
    withoutNas: "go on without a NAS",
    connecting: "connecting…",
    connect: "Connect",
    noLibrary: "There is no library yet in",
    libraryName: "Name of the library",
    libraryNamePlaceholder: "e.g. Our photos",
    pointOpen: "the library is elsewhere: point at existing folders",
    pointHint: "Per part a shared folder and a folder inside it (empty: the shared folder itself). This goes into familylightbox.json in",
    pathPlaceholder: "folder",
    pointClose: "a new library after all",
    back: "‹ NAS",
    saving: "saving…",
    point: "Point at the folders",
    settingUp: "setting up…",
    setUp: "Set up a new library here",
    folderTitle: "Folder for the local copies",
    folderHint: "previews, lists and the photos you keep on this Mac",
    choose: "Choose…",
    folderIntro: "Choose where FamilyLightbox keeps its copies on this Mac. Then the app starts again.",
    save: "Save",
    connectedAs: "Connected as",
    whoIntro: "Who uses this Mac? The lists, stars and hearts remember who chose what.",
    knownOnNas: "known on the NAS already",
    devicesOf: (devices: string[]) => `devices: ${devices.join(", ")}`,
    yourName: "Your name",
    ownerPlaceholder: "short name, lowercase",
    deviceName: "Name of this Mac",
    nameTaken: (owner: string) => `this name belongs to ${owner} on the NAS`,
  },
};

export type Catalog = typeof en;
