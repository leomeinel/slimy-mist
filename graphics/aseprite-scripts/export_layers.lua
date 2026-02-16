--[[

Description:
A script to save all different layers in different files.

Made by Gaspi.
   - Itch.io: https://gaspi.itch.io/
   - Twitter: @_Gaspi
Further Contributors:
    - Levy E ("StoneLabs")
    - David Höchtl ("DavidHoechtl")
    - Demonkiller8973
--]]

-- Auxiliary functions

-- Path handling.

-- Return the path to the dir containing a file.
-- Source: https://stackoverflow.com/questions/9102126/lua-return-directory-path-from-path
local function dirName(str)
   return str:match("(.*" .. Sep .. ")")
end

-- Return the name of a file given its full path..
-- Source: https://codereview.stackexchange.com/questions/90177/get-file-name-with-extension-and-get-only-extension
local function baseName(str)
   return str:match("^.*" .. Sep .. "(.+)$") or str
end

-- Return the name of a file excluding the extension, this being, everything after the dot.
-- Source: https://stackoverflow.com/questions/18884396/extracting-filename-only-with-pattern-matching
local function removeExtension(str)
   return str:match("(.+)%..+")
end

-- sprite handling.

-- Hides all layers and sub-layers inside a group, returning a list with all
-- initial states of each layer's visibility.
local function hideLayers(sprite)
   local data = {} -- Save visibility status of each layer here.
   for i,layer in ipairs(sprite.layers) do
      if layer.isGroup then
         -- Recursive for groups.
         data[i] = hideLayers(layer)
      else
         data[i] = layer.isVisible
         layer.isVisible = false
      end
   end
   return data
end

-- Restore layers visibility.
local function restoreLayersVisibility(sprite, data)
   for i,layer in ipairs(sprite.layers) do
      if layer.isGroup then
         -- Recursive for groups.
         restoreLayersVisibility(layer, data[i])
      else
         layer.isVisible = data[i]
      end
   end
end

-- Dialog
local function msgDialog(title, msg)
   local dialog = Dialog(title)
   dialog:label{
      id = "msg",
      text = msg
   }
   dialog:newrow()
   dialog:button{id = "close", text = "Close", onclick = function() dialog:close() end }
   return dialog
end

-- Function to calculate the bounding box of the non-transparent pixels in a layer
local function calculateBoundingBox(layer)
    local minX, minY, maxX, maxY = nil, nil, nil, nil
    for _, cel in ipairs(layer.cels) do
        local image = cel.image
        local position = cel.position

        for y = 0, image.height - 1 do
            for x = 0, image.width - 1 do
                if image:getPixel(x, y) ~= 0 then -- Non-transparent pixel
                    local pixelX = position.x + x
                    local pixelY = position.y + y
                    if not minX or pixelX < minX then minX = pixelX end
                    if not minY or pixelY < minY then minY = pixelY end
                    if not maxX or pixelX > maxX then maxX = pixelX end
                    if not maxY or pixelY > maxY then maxY = pixelY end
                end
            end
        end
    end
    return Rectangle(minX, minY, maxX - minX + 1, maxY - minY + 1)
end

-- Variable to keep track of the number of layers exported.
local n_layers = 0

-- Exports every layer individually.
local function exportLayers(sprite, root_layer, filename, group_sep, data)
    for _, layer in ipairs(root_layer.layers) do
        local prefix = data.exclusion_prefix or "_"
        -- Skip layer with specified prefix and prefix is not empty
        if data.exclude_prefix and prefix ~= "" and string.sub(layer.name, 1, #prefix) == prefix then
            goto continue
        end
        local filename = filename
        if layer.isGroup then
            -- Recursive for groups.
            local previousVisibility = layer.isVisible
            layer.isVisible = true
            filename = filename:gsub("{layergroups}",
                                     layer.name .. group_sep .. "{layergroups}")
            exportLayers(sprite, layer, filename, group_sep, data)
            layer.isVisible = previousVisibility
        else
            -- Ignore reference layers.
            if layer.isReference then
                goto continue
            end

            -- Individual layer. Export it.
            layer.isVisible = true
            filename = filename:gsub("{layergroups}", "")
            filename = filename:gsub("{layername}", layer.name)
            os.execute("mkdir \"" .. dirName(filename) .. "\"")
            if data.spritesheet then
                local sheettype=SpriteSheetType.HORIZONTAL
                if (data.tagsplit == "To Rows") then
                    sheettype=SpriteSheetType.ROWS
                elseif (data.tagsplit == "To Columns") then
                    sheettype=SpriteSheetType.COLUMNS
                end
                app.command.ExportSpriteSheet{
                    ui=false,
                    askOverwrite=false,
                    type=sheettype,
                    columns=0,
                    rows=0,
                    width=0,
                    height=0,
                    bestFit=false,
                    textureFilename=filename,
                    dataFilename="",
                    dataFormat=SpriteSheetDataFormat.JSON_HASH,
                    borderPadding=0,
                    shapePadding=0,
                    innerPadding=0,
                    trimSprite=data.trimSprite,
                    trim=data.trimCells,
                    trimByGrid=data.trimByGrid,
                    mergeDuplicates=data.mergeDuplicates,
                    extrude=false,
                    openGenerated=false,
                    layer="",
                    tag="",
                    splitLayers=false,
                    splitTags=(data.tagsplit ~= "No"),
                    listLayers=layer,
                    listTags=true,
                    listSlices=true,
                }
            elseif data.trim then -- Trim the layer
                local boundingRect = calculateBoundingBox(layer)
                -- make a selection on the active layer
                app.activeLayer = layer;
                sprite.selection = Selection(boundingRect);

                -- create a new sprite from that selection
                app.command.NewSpriteFromSelection()

                -- save it as png
                app.command.SaveFile {
                    ui=false,
                    filename=filename
                }
                app.command.CloseFile()

                app.activeSprite = layer.sprite  -- Set the active sprite to the current layer's sprite
                sprite.selection = Selection();
            else
                sprite:saveCopyAs(filename)
            end
            layer.isVisible = false
            n_layers = n_layers + 1
        end
        ::continue::
    end
end

-- Current sprite.
local sprite = app.activeSprite
if sprite == nil then
   -- Show error, no sprite active.
   local dialog = msgDialog("Error", "No sprite is currently active. Please, open a sprite first and run again.")
   dialog:show()
   return 1
end

-- Identify operative system.
Sep = string.sub(sprite.filename, 1, 1) == "/" and "/" or "\\"

if dirName(sprite.filename) == nil then
   -- Error, can't identify OS when the sprite isn't saved somewhere.
   local dialog = msgDialog("Error", "Current sprite is not associated to a file. Please, save your sprite and run again.")
   dialog:show()
   return 1
end

-- Open main dialog.
local dialog = Dialog("Export layers")
dialog:file{
    id = "directory",
    label = "Output directory:",
    filename = sprite.filename,
    open = false
}
dialog:entry{
    id = "filename",
    label = "File name format:",
    text = "{layergroups}{layername}"
}
dialog:combobox{
    id = 'format',
    label = 'Export Format:',
    option = 'webp',
    options = {'webp', 'png', 'gif', 'jpg'}
}
dialog:combobox{
    id = 'group_sep',
    label = 'Group separator:',
    option = Sep,
    options = {Sep, '-', '_'}
}
dialog:slider{id = 'scale', label = 'Export Scale:', min = 1, max = 10, value = 1}
dialog:check{
    id = "spritesheet",
    label = "Export as spritesheet:",
    selected = true,
    onclick = function()
        -- Hide these options when spritesheet is checked.
        dialog:modify{
            id = "trim",
            visible = not dialog.data.spritesheet
        }
        -- Show these options when spritesheet is checked.
        dialog:modify{
            id = "trimSprite",
            visible = dialog.data.spritesheet
        }
        dialog:modify{
            id = "trimCells",
            visible = dialog.data.spritesheet
        }
        dialog:modify{
            id = "mergeDuplicates",
            visible = dialog.data.spritesheet
        }
        dialog:modify{
            id = "tagsplit",
            visible = dialog.data.spritesheet
        }
    end
}
dialog:check{
    id = "trim",
    label = "Trim:",
    visible = true,
    selected = false
}
dialog:check{
    id = "trimSprite",
    label = "  Trim sprite:",
    selected = false,
    visible = true,
    onclick = function()
        dialog:modify{
            id = "trimByGrid",
            visible = dialog.data.trimSprite or dialog.data.trimCells,
        }
    end
}
dialog:check{
    id = "trimCells",
    label = "  Trim Cells:",
    selected = false,
    visible = true,
    onclick = function()
        dialog:modify{
            id = "trimByGrid",
            visible = dialog.data.trimSprite or dialog.data.trimCells,
        }
    end
}
dialog:check{
    id = "trimByGrid",
    label = "  Trim Grid:",
    selected = false,
    visible = true
}
dialog:combobox{ -- Spritesheet export only option
    id = "tagsplit",
    label = "  Split Tags:",
    visible = true,
    option = 'To Rows',
    options = {'No', 'To Rows', 'To Columns'}
}
dialog:check{ -- Spritesheet export only option
    id = "mergeDuplicates",
    label = "  Merge duplicates:",
    selected = false,
    visible = false
}
dialog:check{
    id = "exclude_prefix",
    label = "Exclude layers with prefix",
    selected = false,
    onclick = function()
        dialog:modify{
            id = "exclusion_prefix",
            visible = dialog.data.exclude_prefix
        }
    end
}
dialog:entry{
    id = "exclusion_prefix",
    label = "  Prefix:",
    text = "_",
    visible = false
}
dialog:check{id = "save", label = "Save sprite:", selected = false}
dialog:button{id = "ok", text = "Export"}
dialog:button{id = "cancel", text = "Cancel"}
dialog:show()

if not dialog.data.ok then return 0 end

-- Get path and filename
local output_path = dirName(dialog.data.directory)
local filename = dialog.data.filename .. "." .. dialog.data.format

if output_path == nil then
    local dialog = msgDialog("Error", "No output directory was specified.")
    dialog:show()
    return 1
end

-- Switch to RGB if necessary
local colorMode = sprite.colorMode
if sprite.colorMode == ColorMode.INDEXED then
    app.command.ChangePixelFormat {
        format="rgb"
    }
end

local group_sep = dialog.data.group_sep
filename = filename:gsub("{spritename}",
                         removeExtension(baseName(sprite.filename)))
filename = filename:gsub("{groupseparator}", group_sep)

-- Finally, perform everything.
sprite:resize(sprite.width * dialog.data.scale, sprite.height * dialog.data.scale)
local layers_visibility_data = hideLayers(sprite)
exportLayers(sprite, sprite, output_path .. filename, group_sep, dialog.data)
restoreLayersVisibility(sprite, layers_visibility_data)
sprite:resize(sprite.width / dialog.data.scale, sprite.height / dialog.data.scale)

-- Switch back to INDEXED if necessary
if sprite.colorMode ~= colorMode then
    app.command.ChangePixelFormat {
        format="indexed"
    }
end

-- Save the original file if specified
if dialog.data.save then sprite:saveAs(dialog.data.directory) end

-- Success dialog.
local dialog = msgDialog("Success!", "Exported " .. n_layers .. " layers.")
dialog:show()

return 0
