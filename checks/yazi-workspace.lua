local role = "workspace-popup"
local cwd = "/current"
local notifications = {}
local calls = {}
local succeeds = true

cx = { active = { current = { cwd = cwd } } }
ya = {
	sync = function(fn) return fn end,
	notify = function(message) notifications[#notifications + 1] = message end,
}
os.getenv = function(name)
	if name == "YZX_OPEN" then return "/yzx-open" end
	if name == "YZX_YAZI_ROLE" then return role end
end

Command = setmetatable({ PIPED = "piped" }, {
	__call = function(_, program)
		assert(program == "/yzx-open")
		local command = {}
		function command:arg(args)
			calls[#calls + 1] = args
			return self
		end
		function command:stdout() return self end
		function command:stderr() return self end
		function command:spawn() return self end
		function command:wait_with_output()
			return { status = { success = succeeds }, stderr = "failed" }
		end
		return command
	end,
})

local plugin = assert(dofile(assert(arg[1])))
plugin:entry()
assert(calls[1][1] == "--set-workspace" and calls[1][2] == cwd)
assert(notifications[1].content == "Set to " .. cwd)

role = "startup-picker"
plugin:entry()
assert(calls[2][1] == "--retarget-workspace" and calls[2][2] == cwd)
assert(#notifications == 1)

role = "workspace-popup"
succeeds = false
plugin:entry()
assert(notifications[2].level == "error" and notifications[2].content == "failed")
