-- The edited photos of the lists for FamilyLightbox (F025/S02): after Lijsten bijwerken, every
-- photo in the lists' collections that has an edit (and every virtual copy of such a photo) is
-- rendered twice, at 2 048 px and at full size, into ~/.config/media/lightroom-out/, with a JSON
-- next to them that the app reads. What was rendered is kept in the prefs by photo and edit time,
-- so a second click renders nothing new. A photo whose edit is gone gets a JSON that says so.
local LrDate = import "LrDate"
local LrExportSession = import "LrExportSession"
local LrFileUtils = import "LrFileUtils"
local LrLogger = import "LrLogger"
local LrPathUtils = import "LrPathUtils"
local LrPrefs = import "LrPrefs"
local LrProgressScope = import "LrProgressScope"
local LrTasks = import "LrTasks"

local log = LrLogger("FamilyLightbox")

local OUT = LrPathUtils.child(LrPathUtils.getStandardFilePath("home"), ".config/media/lightroom-out")

-- a copy name as a slug (data contract 5.4): lowercase, a-z0-9 and '-'
local function slug(name)
  local s = string.lower(name or "")
  s = s:gsub("[àáâäã]", "a"):gsub("[èéêë]", "e"):gsub("[ìíîï]", "i"):gsub("[òóôöõ]", "o"):gsub("[ùúûü]", "u"):gsub("ç", "c")
  s = s:gsub("[^a-z0-9]+", "-"):gsub("^-+", ""):gsub("-+$", "")
  return s
end

-- whether Lightroom changed how the photo looks: its own flag where this Lightroom has it,
-- otherwise only a crop (editCount counts stars and keywords too)
local warned = false
local function edited(photo)
  local ok, has = LrTasks.pcall(function() return photo:getRawMetadata("hasDevelopAdjustments") end)
  if ok and has ~= nil then return has end
  if not warned then
    log:warn("hasDevelopAdjustments onbekend in deze Lightroom: alleen uitgesneden foto's tellen als bewerkt")
    warned = true
  end
  return photo:getRawMetadata("isCropped") == true
end

-- the variants of the masters: { photo, path, variant, time }
local function variants(masters)
  local out = {}
  for _, m in pairs(masters) do
    local path = m:getRawMetadata("path")
    if m:getRawMetadata("fileFormat") ~= "VIDEO" then
      if edited(m) then
        table.insert(out, { photo = m, path = path, variant = "lr", time = m:getRawMetadata("lastEditTime") or 0 })
      end
      local seen = {}
      for _, c in ipairs(m:getRawMetadata("virtualCopies") or {}) do
        local base = slug(c:getFormattedMetadata("copyName"))
        if base == "" then base = "copy" end
        local s, n = base, 1
        while seen[s] do n = n + 1; s = base .. "-" .. n end
        seen[s] = true
        table.insert(out, { photo = c, path = path, variant = "lr-" .. s, time = c:getRawMetadata("lastEditTime") or 0 })
      end
    end
  end
  return out
end

local function settings(dir, long)
  local s = {
    LR_format = "JPEG", LR_jpeg_quality = 0.9, LR_export_colorSpace = "sRGB",
    LR_export_destinationType = "specificFolder", LR_export_destinationPathPrefix = dir,
    LR_export_useSubfolder = false, LR_collisionHandling = "rename", LR_reimportExportedPhoto = false,
    LR_minimizeEmbeddedMetadata = true, LR_removeLocationMetadata = true, LR_outputSharpeningOn = false,
    LR_includeVideoFiles = false, LR_size_doConstrain = false, LR_useWatermark = false,
  }
  if long then
    s.LR_size_doConstrain = true
    s.LR_size_resizeType = "longEdge"
    s.LR_size_maxHeight = long
    s.LR_size_maxWidth = long
    s.LR_size_units = "pixels"
    s.LR_size_doNotEnlarge = true
  end
  return s
end

-- renders the photos into dir; returns { [photo] = path of the file }
local function render(photos, dir, long, progress, from, to)
  local done = {}
  local session = LrExportSession { photosToExport = photos, exportSettings = settings(dir, long) }
  local n = 0
  for _, r in session:renditions() do
    if progress:isCanceled() then r:skipRender() else
      local ok, path = r:waitForRender()
      if ok then done[r.photo] = path else log:warn("niet gerenderd: " .. tostring(path)) end
    end
    n = n + 1
    progress:setPortionComplete(from + (to - from) * n / #photos, 1)
  end
  return done
end

local function writeJson(name, fields)
  local parts = {}
  for k, v in pairs(fields) do
    local value = type(v) == "string" and string.format("%q", v):gsub("\\\n", "\\n") or tostring(v)
    table.insert(parts, string.format("%q: %s", k, value))
  end
  local tmp = LrPathUtils.child(OUT, name .. ".tmp")
  local f = io.open(tmp, "w")
  f:write("{" .. table.concat(parts, ", ") .. "}\n")
  f:close()
  LrFileUtils.move(tmp, LrPathUtils.child(OUT, name))
end

local function iso(time)
  return LrDate.timeToW3CDate(time)
end

-- masters: { [localIdentifier] = LrPhoto } of the lists' collections. Returns rendered, removed.
return function(masters)
  local prefs = LrPrefs.prefsForPlugin()
  local done = prefs.renditions or {} -- key path|variant → the edit time rendered
  local list = variants(masters)
  local todo, present, masterPaths = {}, {}, {}
  for _, m in pairs(masters) do masterPaths[m:getRawMetadata("path")] = true end
  for _, v in ipairs(list) do
    local key = v.path .. "|" .. v.variant
    present[key] = true
    if done[key] ~= v.time then table.insert(todo, v) end
  end
  -- edits gone: from a photo still in a list, or a copy deleted
  local removed = 0
  LrFileUtils.createAllDirectories(OUT)
  local stamp = string.format("%d", LrDate.currentTime() * 1000)
  for key in pairs(done) do
    local path, variant = key:match("^(.*)|([^|]*)$")
    if path and masterPaths[path] and not present[key] then
      removed = removed + 1
      writeJson(string.format("%s-r%d.json", stamp, removed), { version = 1, path = path, variant = variant, removed = true })
      done[key] = nil
    end
  end
  if #todo == 0 then
    prefs.renditions = done
    return 0, removed
  end

  local progress = LrProgressScope { title = "FamilyLightbox: bewerkte foto's" }
  progress:setCaption(string.format("%d bewerkte foto's renderen", #todo))
  local work = LrPathUtils.child(OUT, ".render-" .. stamp)
  LrFileUtils.createAllDirectories(LrPathUtils.child(work, "small"))
  LrFileUtils.createAllDirectories(LrPathUtils.child(work, "full"))
  local photos = {}
  for _, v in ipairs(todo) do table.insert(photos, v.photo) end
  local small = render(photos, LrPathUtils.child(work, "small"), 2048, progress, 0, 0.4)
  local full = progress:isCanceled() and {} or render(photos, LrPathUtils.child(work, "full"), nil, progress, 0.4, 1)

  local rendered = 0
  for n, v in ipairs(todo) do
    local s, f = small[v.photo], full[v.photo]
    if s and f then
      local id = string.format("%s-%d", stamp, n)
      LrFileUtils.move(s, LrPathUtils.child(OUT, id .. ".jpg"))
      LrFileUtils.move(f, LrPathUtils.child(OUT, id .. ".full.jpg"))
      -- the JSON last: the app takes a rendition only when its JSON is there
      writeJson(id .. ".json", {
        version = 1, path = v.path, variant = v.variant, edited = iso(v.time),
        small = id .. ".jpg", full = id .. ".full.jpg",
      })
      done[v.path .. "|" .. v.variant] = v.time
      rendered = rendered + 1
    end
  end
  LrFileUtils.delete(work)
  prefs.renditions = done
  progress:done()
  return rendered, removed
end
