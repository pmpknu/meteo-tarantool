local rust_funcs = {
  "get_geocoding",
  "get_weather",
}

local api = {}

-- @return boolean?, table?
api.init = function()
    local module = box.lib.load("libmeteo_handler") -- luacheck: globals box

    for _, func_name in ipairs(rust_funcs) do
        local func = module:load(func_name)
        api[func_name] = function(...)
            local is_err, res = pcall(func, ...)

            if is_err == false then
                return nil, res
            end

            return res
        end
    end
    return true, nil
end

return api

