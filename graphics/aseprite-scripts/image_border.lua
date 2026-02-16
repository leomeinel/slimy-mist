----------------------------------------------------------------------
--- Finds the layer in the sprite with the given name
--- or creates one at the top of the stack.
---
--- PARAMS
--- sprite (Sprite)
--- name (String)
---
--- RETURNS
--- Layer
----------------------------------------------------------------------
function findOrCreateLayer(sprite, name)
    local tmpGroup = nil
    for k,layer in ipairs(sprite.layers) do
        if layer.name == '_tmp_' and layer.isGroup then
        tmpGroup = layer
        end
    end

    if tmpGroup == nil then
        tmpGroup = sprite:newGroup()
        tmpGroup.name = '_tmp_'
    end

    for k,layer in ipairs(tmpGroup.layers) do
        if layer.name == name then return layer end
    end

    local newLayer = sprite:newLayer()
    newLayer.name = name
    newLayer.parent = tmpGroup
    return newLayer
end

----------------------------------------------------------------------
--- Determines if a pixel is completely transparent
---
--- PARAMS
--- pixel (Integer)
---
--- RETURNS
--- Boolean
----------------------------------------------------------------------
function isTransparentPixel(pixel)
    local sprite = app.activeSprite
    if sprite.colorMode == ColorMode.INDEXED then
        return pixel == sprite.spec.transparentColor
    else
        return app.pixelColor.rgbaA(pixel) == 0
    end
end

----------------------------------------------------------------------
--- Creates a new image which contains an outline of the passed image.
---
--- PARAMS
--- image (Image)
--- color (Color)
---
--- RETURNS
--- Image
----------------------------------------------------------------------
function outlineImage(image, color)
    local outline = Image(image.width, image.height, image.colorMode)

    for x = 0, image.width - 1, 1 do
        for y = 0, image.height - 1, 1 do
        if isTransparentPixel(image:getPixel(x, y)) then
            local draw = false
            if x > 0 and not isTransparentPixel(image:getPixel(x - 1, y)) then draw = true end
            if y > 0 and not isTransparentPixel(image:getPixel(x, y - 1)) then draw = true end
            if x < (image.width - 1) and not isTransparentPixel(image:getPixel(x + 1, y)) then draw = true end
            if y < (image.height - 1) and not isTransparentPixel(image:getPixel(x, y + 1)) then draw = true end
            if draw then outline:drawPixel(x, y, color) end
        end
        end
    end

    return outline
end

----------------------------------------------------------------------
--- Invoke a callback with `settingsCallback()` used as settings.
---
--- PARAMS
--- @param settingsCallback() that returns `Dialog().data`.
--- @param callback(sprite, frame, settings) called for each selected frame.
----------------------------------------------------------------------
function invoke(settingsCallback, callback)
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
        local border = findOrCreateLayer(sprite, "Image Border")

        local cel = sprite:newCel(border, frame.frameNumber)
        local rawImage = Image(sprite.width, sprite.height, sprite.colorMode)

        rawImage:drawSprite(sprite, frame.frameNumber)
        cel.image = outlineImage(rawImage, settings.borderColor)
    end
)

return 0
