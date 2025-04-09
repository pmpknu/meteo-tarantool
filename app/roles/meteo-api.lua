local cartridge = require('cartridge')
local api = require('app.api')

local function init(opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    local httpd = assert(cartridge.service_get('httpd'), "Failed to get httpd service")
    httpd:route({method = 'GET', path = '/api/weather'}, function(req)
        local lat = tonumber(req:query_param('lat'))
        local lon = tonumber(req:query_param('lon'))
        
        if not lat or not lon then
            return {status = 400, body = 'Missing or invalid lat/lon parameters'}
        end

        local res, err = api.get_weather(lat, lon)
        if err then
          return {status = 400, body = err}
        end
        return {body = res}
    end)

    httpd:route({method = 'GET', path = '/api/geocoding'}, function(req)
        local city = req:query_param('city')
        
        if not city or city == '' then
            return {status = 400, body = 'Missing city parameter'}
        end
        
        local res, err = api.get_geocoding(city)
        if err then
            return {status = 400, body = err}
        end
        return {body = res}
    end)

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
