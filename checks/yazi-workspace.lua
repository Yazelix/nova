local role = "workspace-popup"
local cwd = "/current"
local child_dir = "/current/child dir"
local notifications = {}
local calls = {}
local emitted = {}
local succeeds = true
local zoxide_calls = 0
local fzf_calls = 0
local zoxide_target = "/recent folder"
local zoxide_key = ""
local fzf_options

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

Command = setmetatable({ INHERIT = "inherit", PIPED = "piped" }, {
	__call = function(_, program)
		assert(program == "/yzx-open" or program == "zoxide" or program == "fzf")
		local command = {}
		function command:arg(args)
			if program == "zoxide" then
				zoxide_calls = zoxide_calls + 1
				assert(args[1] == "query" and args[2] == "--list")
			else
				if program == "/yzx-open" then calls[#calls + 1] = args end
			end
			return self
		end
		function command:env(name, value)
			if program == "fzf" and name == "FZF_DEFAULT_OPTS" then fzf_options = value end
			return self
		end
		function command:stdin() return self end
		function command:stdout() return self end
		function command:stderr() return self end
		function command:spawn() return self end
		function command:write_all(input) assert(input == zoxide_target .. "\n") end
		function command:flush() end
		function command:output() return self:wait_with_output() end
		function command:wait_with_output()
			if program == "zoxide" then
				return { status = { success = true }, stdout = zoxide_target .. "\n", stderr = "" }
			end
			if program == "fzf" then
				fzf_calls = fzf_calls + 1
				return { status = { success = true }, stdout = zoxide_key .. "\n" .. zoxide_target .. "\n", stderr = "" }
			end
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
assert(calls[2][1] == "--set-workspace" and calls[2][2] == child_dir)
assert(notifications[2].content == "Set to " .. child_dir)

role = "workspace-popup"
cx.active.current.hovered = { url = "/current/file", cha = { is_dir = false } }
plugin:entry()
assert(calls[3][2] == cwd)

cx.active.current.hovered = nil
plugin:entry()
assert(calls[4][2] == cwd)

succeeds = false
plugin:entry()
assert(notifications[5].level == "error" and notifications[5].content == "failed")

package.preload["tab-workspace"] = function() return plugin end
package.preload.zoxide = function()
	return {
		setup = function() end,
		is_empty = function(current)
			assert(current == cwd)
			return false
		end,
	}
end
ui = { hide = function() return { drop = function() end } end }
succeeds = true
local search = assert(dofile(assert(arg[3])))
local workspace_calls = #calls
role = "workspace-popup"
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 1 and #calls == workspace_calls)
assert(fzf_calls == 1)
assert(emitted[#emitted][1] == "cd" and emitted[#emitted][2][1] == zoxide_target)
assert(emitted[#emitted][2].raw == true)
assert(fzf_options:find("--expect=alt-enter", 1, true))
assert(fzf_options:find("Enter Browse here · Alt+Enter Set tab folder", 1, true))

role = "startup-picker"
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 2 and fzf_calls == 2 and #calls == workspace_calls)
assert(emitted[#emitted][1] == "cd" and emitted[#emitted][2][1] == zoxide_target)

zoxide_key = "alt-enter"
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 3 and fzf_calls == 3 and calls[#calls][1] == "--set-workspace" and calls[#calls][2] == zoxide_target)

role = "workspace-popup"
search:entry({ args = { source = "zoxide" } })
assert(zoxide_calls == 4 and fzf_calls == 4 and calls[#calls][1] == "--set-workspace" and calls[#calls][2] == zoxide_target)

role = nil
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 4 and emitted[#emitted][1] == "spot" and type(emitted[#emitted][2]) == "table")
search:entry({ args = { source = "zoxide" } })
assert(emitted[#emitted][1] == "plugin" and emitted[#emitted][2][1] == "zoxide")

for _, name in ipairs({ "auto-layout", "git", "starship", "zoxide" }) do
	package.preload[name] = function() return { setup = function() end } end
end
emitted = {}
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
local startup_footer = footer("startup-picker")
assert(startup_footer == " Alt+Enter Set tab folder · Tab/Z Search · Shift+Tab Spot")
assert(emitted[1][1] == "plugin" and emitted[1][2][1] == "quick-search")
assert(footer("workspace-popup") == " Alt+Enter Set tab folder · Tab/Z Search · Shift+Tab Spot")
assert(#emitted == 1)
cx.layer = "input"
assert(Status.render() == "")
