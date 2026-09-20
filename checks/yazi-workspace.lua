local role = "workspace-popup"
local cwd = "/current"
local child_dir = "/current/child dir"
local notifications = {}
local calls = {}
local emitted = {}
local succeeds = true

cx = { active = { current = { cwd = cwd, hovered = { url = child_dir, cha = { is_dir = true } } } } }
ya = {
	sync = function(fn) return fn end,
	notify = function(message) notifications[#notifications + 1] = message end,
	emit = function(name, args) emitted[#emitted + 1] = { name, args } end,
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
assert(calls[1][1] == "--set-workspace" and calls[1][2] == child_dir)
assert(notifications[1].content == "Set to " .. child_dir)

role = "startup-picker"
plugin:entry()
assert(calls[2][1] == "--retarget-workspace" and calls[2][2] == child_dir)
assert(#notifications == 1)

role = "workspace-popup"
cx.active.current.hovered = { url = "/current/file", cha = { is_dir = false } }
plugin:entry()
assert(calls[3][2] == cwd)

cx.active.current.hovered = nil
plugin:entry()
assert(calls[4][2] == cwd)

succeeds = false
plugin:entry()
assert(notifications[4].level == "error" and notifications[4].content == "failed")

for _, name in ipairs({ "auto-layout", "git", "starship", "zoxide" }) do
	package.preload[name] = function() return { setup = function() end } end
end
cx.layer = "mgr"
local function footer(name)
	role = name
	Status = { LEFT = 1, RIGHT = 2, removed = 0 }
	function Status:children_remove() self.removed = self.removed + 1 end
	function Status:children_add(render) self.render = render end
	dofile(assert(arg[2]))
	assert(Status.removed == 6)
	return Status.render()
end
assert(footer("startup-picker"):find("Alt+Enter Use this folder", 1, true))
assert(emitted[1][1] == "plugin" and emitted[1][2][1] == "startup-search")
assert(footer("workspace-popup") == " Alt+Enter Workspace · Shift+Z Jump")
assert(#emitted == 1)
cx.layer = "input"
assert(Status.render() == "")
