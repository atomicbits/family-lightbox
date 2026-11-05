-- Lijsten bijwerken (F023/S02): reads the file FamilyLightbox writes
-- (~/.config/media/familylightbox-lightroom.json) and keeps the collection set "FamilyLightbox"
-- equal to the lists: a collection per list, archived ones in "Gearchiveerd". A list is found
-- again by its id, kept in this plug-in's preferences, so a renamed list renames its collection.
-- Nothing outside "FamilyLightbox" is touched, and nothing goes back to the lists.
local LrApplication = import "LrApplication"
local LrDialogs = import "LrDialogs"
local LrFileUtils = import "LrFileUtils"
local LrLogger = import "LrLogger"
local LrPathUtils = import "LrPathUtils"
local LrPrefs = import "LrPrefs"
local LrProgressScope = import "LrProgressScope"
local LrTasks = import "LrTasks"

local Json = require "Json"

local log = LrLogger("FamilyLightbox")
log:enable("logfile") -- in ~/Documents/LrClassicLogs/FamilyLightbox.log

local function readFile()
  local path = LrPathUtils.child(LrPathUtils.getStandardFilePath("home"), ".config/media/familylightbox-lightroom.json")
  if not LrFileUtils.exists(path) then
    return nil, "FamilyLightbox heeft nog geen lijsten voor Lightroom geschreven: open FamilyLightbox eerst."
  end
  local f = io.open(path, "r")
  local text = f:read("*a")
  f:close()
  local ok, data = pcall(Json.decode, text)
  if not ok or type(data) ~= "table" or type(data.lists) ~= "table" then
    return nil, "Het bestand van FamilyLightbox is niet leesbaar: " .. tostring(data)
  end
  return data
end

local function collectionName(list)
  return string.format("%d %s", list.year, list.name)
end

local function childByName(children, name)
  for _, c in ipairs(children) do
    if c:getName() == name then return c end
  end
  return nil
end

LrTasks.startAsyncTask(function()
  local data, problem = readFile()
  if not data then
    LrDialogs.message("FamilyLightbox", problem, "warning")
    return
  end
  local catalog = LrApplication.activeCatalog()
  local prefs = LrPrefs.prefsForPlugin()
  local known = prefs.collections or {}
  local progress = LrProgressScope { title = "FamilyLightbox: lijsten bijwerken" }

  -- 1. outside a write access: the collections known from earlier, and what is in the catalog
  local collections, toAdd, away = {}, {}, 0
  for _, list in ipairs(data.lists) do
    if known[list.id] then
      local ok, c = pcall(function() return catalog:getCollectionByLocalIdentifier(known[list.id]) end)
      if ok and c then collections[list.id] = c end
    end
    for _, p in ipairs(list.photos or {}) do
      if p.here then
        if not catalog:findPhotoByPath(p.path) then table.insert(toAdd, p.path) end
      else
        away = away + 1
      end
    end
  end
  progress:setPortionComplete(1, 4)

  -- 2. a first write access only makes what is missing: Lightroom can't read a collection back in
  -- the write access that made it
  local added = 0
  catalog:withWriteAccessDo("FamilyLightbox: collecties maken", function()
    local set = catalog:createCollectionSet("FamilyLightbox", nil, true)
    local archivedSet = catalog:createCollectionSet("Gearchiveerd", set, true)
    for _, list in ipairs(data.lists) do
      if not collections[list.id] then
        catalog:createCollection(collectionName(list), list.archived and archivedSet or set, true)
      end
    end
    for _, path in ipairs(toAdd) do
      local ok, photo = pcall(function() return catalog:addPhoto(path) end)
      if ok and photo then added = added + 1 else log:warn("niet toegevoegd: " .. path .. " " .. tostring(photo)) end
    end
  end, { timeout = 120 })
  progress:setPortionComplete(2, 4)

  -- 3. outside again: the sets and the new collections found back by name, the photos by path
  local set = childByName(catalog:getChildCollectionSets(), "FamilyLightbox")
  local archivedSet = set and childByName(set:getChildCollectionSets(), "Gearchiveerd")
  if not set or not archivedSet then
    progress:done()
    LrDialogs.message("FamilyLightbox", "De verzamelingset FamilyLightbox is niet gevonden na het maken.", "warning")
    return
  end
  local work = {}
  for _, list in ipairs(data.lists) do
    local parent = list.archived and archivedSet or set
    local c = collections[list.id] or childByName(parent:getChildCollections(), collectionName(list))
    if c then
      known[list.id] = c.localIdentifier
      local should, adds, removes, has = {}, {}, {}, {}
      for _, p in ipairs(list.photos or {}) do
        if p.here then
          local photo = catalog:findPhotoByPath(p.path)
          if photo then should[photo.localIdentifier] = photo end
        end
      end
      for _, photo in ipairs(c:getPhotos()) do
        has[photo.localIdentifier] = true
        if not should[photo.localIdentifier] then table.insert(removes, photo) end
      end
      for id, photo in pairs(should) do
        if not has[id] then table.insert(adds, photo) end
      end
      local now = c:getParent()
      table.insert(work, {
        collection = c, adds = adds, removes = removes, name = collectionName(list), parent = parent,
        rename = c:getName() ~= collectionName(list),
        move = not now or now.localIdentifier ~= parent.localIdentifier,
      })
    else
      log:warn("geen collectie voor " .. list.id)
    end
  end
  progress:setPortionComplete(3, 4)

  -- 4. a second write access: names, places and photos
  local changed = 0
  catalog:withWriteAccessDo("FamilyLightbox: lijsten bijwerken", function()
    for _, w in ipairs(work) do
      if w.rename then w.collection:setName(w.name) end
      if w.move then w.collection:setParent(w.parent) end
      if #w.adds > 0 then w.collection:addPhotos(w.adds) end
      if #w.removes > 0 then w.collection:removePhotos(w.removes) end
      if w.rename or w.move or #w.adds + #w.removes > 0 then changed = changed + 1 end
    end
  end, { timeout = 120 })
  prefs.collections = known
  progress:done()

  local text = string.format("%d lijsten, %d bijgewerkt.", #data.lists, changed)
  if added > 0 then text = text .. string.format("\n%d originelen aan de catalogus toegevoegd.", added) end
  if away > 0 then
    text = text .. string.format("\n%d originelen nog niet op deze Mac: haal ze op in FamilyLightbox (rechtsklik op de lijst, Alle originelen ophalen) en klik dan opnieuw op Lijsten bijwerken.", away)
  end
  log:info(text)
  LrDialogs.message("FamilyLightbox", text, "info")
end)
