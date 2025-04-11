local cartridge = require('cartridge')
local api = require('app.api')

local function init(opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end
    api.make_http_endpoints()
    local log = require('log')
    log.info('Meteo API service initialized')

    return true
end

local function stop()
    return true
end

local function validate_config(conf_new, conf_old) -- luacheck: no unused args
    return true
end

local function apply_config(conf, opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    return true
end

return {
    role_name = 'Meteo API',
    init = init,
    stop = stop,
    validate_config = validate_config,
    apply_config = apply_config,
    -- dependencies = {'cartridge.roles.vshard-router'},
}
