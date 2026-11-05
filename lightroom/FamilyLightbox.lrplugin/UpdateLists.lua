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

  -- what is here and in the catalog, outside the write access (it may take a while)
  local wanted, toAdd, away = {}, {}, 0
  for n, list in ipairs(data.lists) do
    progress:setPortionComplete(n - 1, #data.lists * 2)
    local photos = {}
    for _, p in ipairs(list.photos or {}) do
      if p.here then
        local photo = catalog:findPhotoByPath(p.path)
        if photo then table.insert(photos, photo) else table.insert(toAdd, { list = list.id, path = p.path }) end
      else
        away = away + 1
      end
    end
    wanted[list.id] = photos
  end

  local added, changed = 0, 0
  catalog:withWriteAccessDo("FamilyLightbox: lijsten bijwerken", function()
    local set = catalog:createCollectionSet("FamilyLightbox", nil, true)
    local archivedSet = catalog:createCollectionSet("Gearchiveerd", set, true)
    -- originals here but not in the catalog yet: added where they are
    for _, a in ipairs(toAdd) do
      local ok, photo = pcall(function() return catalog:addPhoto(a.path) end)
      if ok and photo then
        table.insert(wanted[a.list], photo)
        added = added + 1
      else
        log:warn("niet toegevoegd: " .. a.path .. " " .. tostring(photo))
      end
    end
    for n, list in ipairs(data.lists) do
      progress:setPortionComplete(#data.lists + n - 1, #data.lists * 2)
      local parent = list.archived and archivedSet or set
      -- by its id; a collection removed by hand in Lightroom is made again
      local collection = nil
      if known[list.id] then
        local ok, c = pcall(function() return catalog:getCollectionByLocalIdentifier(known[list.id]) end)
        if ok then collection = c end
      end
      if not collection then
        collection = catalog:createCollection(collectionName(list), parent, true)
        known[list.id] = collection.localIdentifier
      end
      if collection:getName() ~= collectionName(list) then collection:setName(collectionName(list)) end
      local now = collection:getParent()
      if not now or now.localIdentifier ~= parent.localIdentifier then collection:setParent(parent) end
      local should = {}
      for _, photo in ipairs(wanted[list.id]) do should[photo.localIdentifier] = photo end
      local adds, removes, has = {}, {}, {}
      for _, photo in ipairs(collection:getPhotos()) do
        has[photo.localIdentifier] = true
        if not should[photo.localIdentifier] then table.insert(removes, photo) end
      end
      for id, photo in pairs(should) do
        if not has[id] then table.insert(adds, photo) end
      end
      if #adds > 0 then collection:addPhotos(adds) end
      if #removes > 0 then collection:removePhotos(removes) end
      if #adds + #removes > 0 then changed = changed + 1 end
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
