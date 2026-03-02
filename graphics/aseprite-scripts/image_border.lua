--- Name of the floating top-level layer group.
local floating_name = "floating"

--- Toggle exclusive visibility for the floating top-level layer group.
---
--- @param sprite (Sprite) The relevant sprite.
--- @param is_visible (bool) The `.isVisible` value for group layer matching `floating_name`.
local function showOnlyFloating(sprite, is_visible)
    for i,layer in ipairs(sprite.layers) do
        if layer.isGroup then
            if layer.name == floating_name then
                layer.isVisible = is_visible
            else
                layer.isVisible = not is_visible
            end
        end
    end
end

--- Toggle visibility on for all top-level layer groups.
---
--- @param sprite (Sprite) The relevant sprite.
local function showAll(sprite)
    for i,layer in ipairs(sprite.layers) do
        if layer.isGroup then
            layer.isVisible = true
        end
    end
end

--- The pixel layer containing the outline.
---
--- @param sprite (Sprite) The relevant Sprite.
--- @param name (string) The `.name` for parent layer group.
--- @param index (integer) The `.stackIndex` for parent layer group.
---
--- @return Layer The outline pixel layer.
local function outlineLayer(sprite, name, index)
    local group = sprite:newGroup()
    group.name = name
    group.stackIndex = index
    local layer = sprite:newLayer()
    layer.name = "_outline_"
    layer.parent = group

    return layer
end

--- Creates a fixed outline layer and optionally a floating outline layer.
---
--- @param sprite (Sprite) The relevant sprite.
---
--- @return (table) A table with `fixed` and `floating` layer references.
---   - `fixed` (Layer): The fixed layer group.
---   - `floating` (Layer|nil): The optional floating layer group that is
---     created if a group layer matching `floating_name` is found.
local function createLayers(sprite)
    local fixed_fx_name = "_fixed_"
    local floating_fx_name = "_" .. floating_name .. "_"
    local floating_stack_index = 0
    for _,layer in ipairs(sprite.layers) do
        if layer.isGroup then
            if layer.name == floating_name then
                floating_stack_index = layer.stackIndex
            elseif layer.name == fixed_fx_name or layer.name == floating_fx_name then
                sprite:deleteLayer(layer)
            end
        end
    end

    local fixed_layer = nil
    local floating_layer = nil
    if floating_stack_index < 1 then
        fixed_layer = outlineLayer(sprite, fixed_fx_name, #sprite.layers + 1)
    else
        fixed_layer = outlineLayer(sprite, fixed_fx_name, floating_stack_index)
        floating_layer = outlineLayer(sprite, floating_fx_name, floating_stack_index + 2)
    end

    return { fixed = fixed_layer, floating = floating_layer }
end

--- Determines if a pixel is completely transparent.
---
--- This is meant to work for both `ColorMode.INDEXED` and `ColorMode.RGB`.
---
--- @param pixel_color (integer) The relevant pixel color value.
---
--- @return bool Whether `pixel_color` is transparent.
local function isTransparent(sprite, pixel_color)
    if sprite.colorMode == ColorMode.INDEXED then
        return pixel_color == sprite.spec.transparentColor
    else
        return app.pixelColor.rgbaA(pixel_color) == 0
    end
end

--- Creates a new image which contains an outline of the passed image.
---
--- @param image (Image) The image to be processed.
--- @param color (Color) The outline color.
---
--- @return Image The created outline of `image`.
local function outlineImage(sprite, image, color)
    local outline = Image(image.width, image.height, image.colorMode)

    for x = 0, image.width - 1, 1 do
        for y = 0, image.height - 1, 1 do
            if isTransparent(sprite, image:getPixel(x, y)) then
                local draw = false
                if x > 0 and not isTransparent(sprite, image:getPixel(x - 1, y)) then draw = true end
                if y > 0 and not isTransparent(sprite, image:getPixel(x, y - 1)) then draw = true end
                if x < (image.width - 1) and not isTransparent(sprite, image:getPixel(x + 1, y)) then draw = true end
                if y < (image.height - 1) and not isTransparent(sprite, image:getPixel(x, y + 1)) then draw = true end
                if draw then outline:drawPixel(x, y, color) end
            end
        end
    end

    return outline
end

--- Creates an outline in `layer` at `frame`.
---
--- @param sprite (Sprite) The relevant sprite.
--- @param frame (Frame) The relevant target frame.
--- @param color (Color) The outline color.
--- @param layer (Layer) The relevant target pixel layer.
local function drawOutlineImage(sprite, frame, color, layer)
    local cel = sprite:newCel(layer, frame.frameNumber)
    local rawImage = Image(sprite.width, sprite.height, sprite.colorMode)
    rawImage:drawSprite(sprite, frame.frameNumber)
    cel.image = outlineImage(sprite, rawImage, color)
end

--- Invoke a callback with `settingsCallback()` used as settings.
---
--- @param settingsCallback() that returns `Dialog().data`.
--- @param callback(sprite, frame, settings) called for each selected frame.
local function invoke(settingsCallback, callback)
    if app.apiVersion < 3 then
        return app.alert("ERROR: This script requires API version 3.")
    end

    local sprite = app.activeSprite
    if sprite == nil then
        return app.alert("ERROR: Active Sprite does not exist.")
    end

    if app.range.type ~= RangeType.FRAMES then
        return app.alert("ERROR: No frames selected.")
    end

    local settings = settingsCallback()
    if not settings.ok then return 0 end

    app.transaction(
        function()
            for i,frame in ipairs(app.range.frames) do
                callback(sprite, frame, settings)
            end
        end
    )
end

-- Run script
invoke(
    function()
        local dialog = Dialog()
        local sprite = app.activeSprite
        if sprite.colorMode == ColorMode.INDEXED then
        dialog:color({ id = "borderColor", label = "Border Color", color = Color{ index = 1 } })
        else
        dialog:color({ id = "borderColor", label = "Border Color", color = Color{ r = 0, g = 0, b = 0, a = 255 } })
        end
        dialog:button({ id = "cancel", text = "Cancel" })
        dialog:button({ id = "ok", text = "OK" })
        dialog:show()

        return dialog.data
    end,

    function(sprite, frame, settings)
        local layers = createLayers(sprite)

        showOnlyFloating(sprite, false)
        drawOutlineImage(sprite, frame, settings.borderColor, layers.fixed)

        if layers.floating ~= nil then
            showOnlyFloating(sprite, true)
            drawOutlineImage(sprite, frame, settings.borderColor, layers.floating)
        end

        showAll(sprite)
    end
)

return 0
