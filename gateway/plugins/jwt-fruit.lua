local jwt = require "resty.jwt"
local utils = require "kong.tools.utils"

local JwtFruitHandler = {}

JwtFruitHandler.VERSION = "1.0.0"
JwtFruitHandler.PRIORITY = 1000

function JwtFruitHandler:access(conf)
    local token = kong.request.get_header("Authorization")

    if not token then
        return kong.response.exit(401, { message = "Missing JWT token" })
    end

    -- Remove "Bearer " prefix
    token = token:gsub("Bearer ", "")

    -- Verify JWT token
    local jwt_obj = jwt:verify(conf.secret, token)

    if not jwt_obj or not jwt_obj.valid then
        return kong.response.exit(401, { message = "Invalid JWT token" })
    end

    -- Add user info to headers for downstream services
    if jwt_obj.payload.sub then
        kong.service.request.set_header("X-User-ID", jwt_obj.payload.sub)
    end

    if jwt_obj.payload.role then
        kong.service.request.set_header("X-User-Role", jwt_obj.payload.role)
    end
end

return JwtFruitHandler