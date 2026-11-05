-- The FamilyLightbox plug-in for Lightroom Classic (F023/S02): the lists of FamilyLightbox as
-- collections, on a click, one way.
return {
  LrSdkVersion = 10.0,
  LrSdkMinimumVersion = 6.0,
  LrToolkitIdentifier = "io.atomicbits.familylightbox",
  LrPluginName = "FamilyLightbox",
  LrPluginInfoUrl = "https://github.com/atomicbits/family-lightbox",
  LrLibraryMenuItems = {
    { title = "Lijsten bijwerken", file = "UpdateLists.lua" },
  },
  VERSION = { major = 0, minor = 2, revision = 0, build = 0 },
}
