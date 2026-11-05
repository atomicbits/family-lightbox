-- A small JSON decoder (Lua 5.1, as in Lightroom): objects, arrays, strings with their escapes
-- (\uXXXX as UTF-8), numbers, true, false and null (nil). Enough for the file the app writes.
local Json = {}

local function utf8(code)
  if code < 0x80 then return string.char(code) end
  if code < 0x800 then return string.char(0xC0 + math.floor(code / 0x40), 0x80 + code % 0x40) end
  return string.char(0xE0 + math.floor(code / 0x1000), 0x80 + math.floor(code / 0x40) % 0x40, 0x80 + code % 0x40)
end

local escapes = { ['"'] = '"', ["\\"] = "\\", ["/"] = "/", b = "\b", f = "\f", n = "\n", r = "\r", t = "\t" }

function Json.decode(s)
  local i = 1
  local value

  local function skip()
    i = string.find(s, "[^ \t\r\n]", i) or (#s + 1)
  end

  local function fail(what)
    error(string.format("JSON: %s at %d", what, i))
  end

  local function str()
    i = i + 1
    local parts = {}
    while true do
      local j = string.find(s, '["\\]', i)
      if not j then fail("unterminated string") end
      table.insert(parts, string.sub(s, i, j - 1))
      if string.sub(s, j, j) == '"' then
        i = j + 1
        return table.concat(parts)
      end
      local e = string.sub(s, j + 1, j + 1)
      if e == "u" then
        local code = tonumber(string.sub(s, j + 2, j + 5), 16) or fail("bad \\u")
        if code >= 0xD800 and code <= 0xDBFF and string.sub(s, j + 6, j + 7) == "\\u" then
          local low = tonumber(string.sub(s, j + 8, j + 11), 16)
          code = 0x10000 + (code - 0xD800) * 0x400 + (low - 0xDC00)
          local b = { 0xF0 + math.floor(code / 0x40000), 0x80 + math.floor(code / 0x1000) % 0x40, 0x80 + math.floor(code / 0x40) % 0x40, 0x80 + code % 0x40 }
          table.insert(parts, string.char(b[1], b[2], b[3], b[4]))
          i = j + 12
        else
          table.insert(parts, utf8(code))
          i = j + 6
        end
      else
        table.insert(parts, escapes[e] or fail("bad escape"))
        i = j + 2
      end
    end
  end

  function value()
    skip()
    local c = string.sub(s, i, i)
    if c == "{" then
      local t = {}
      i = i + 1
      skip()
      if string.sub(s, i, i) == "}" then i = i + 1 return t end
      while true do
        skip()
        if string.sub(s, i, i) ~= '"' then fail("expected a key") end
        local k = str()
        skip()
        if string.sub(s, i, i) ~= ":" then fail("expected ':'") end
        i = i + 1
        t[k] = value()
        skip()
        local d = string.sub(s, i, i)
        i = i + 1
        if d == "}" then return t end
        if d ~= "," then fail("expected ',' or '}'") end
      end
    elseif c == "[" then
      local t = {}
      i = i + 1
      skip()
      if string.sub(s, i, i) == "]" then i = i + 1 return t end
      while true do
        table.insert(t, value())
        skip()
        local d = string.sub(s, i, i)
        i = i + 1
        if d == "]" then return t end
        if d ~= "," then fail("expected ',' or ']'") end
      end
    elseif c == '"' then
      return str()
    elseif string.sub(s, i, i + 3) == "true" then
      i = i + 4 return true
    elseif string.sub(s, i, i + 4) == "false" then
      i = i + 5 return false
    elseif string.sub(s, i, i + 3) == "null" then
      i = i + 4 return nil
    else
      local num = string.match(s, "^-?%d+%.?%d*[eE]?[-+]?%d*", i)
      if not num or num == "" then fail("unexpected character") end
      i = i + #num
      return tonumber(num)
    end
  end

  return value()
end

return Json
