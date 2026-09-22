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

Command = setmetatable({ PIPED = "piped" }, {
	__call = function(_, program)
		assert(program == "/yzx-open" or program == "zoxide" or program == "fzf")
		local command = {}
		function command:arg(args)
			if program == "zoxide" then
				zoxide_calls = zoxide_calls + 1
				assert(args[1] == "query" and args[2] == "--list" and args[3] == "--exclude" and args[4] == cwd)
			elseif program == "/yzx-open" then
				calls[#calls + 1] = args
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
				local stdout = fzf_options:find("--expect=", 1, true)
					and zoxide_key .. "\n" .. zoxide_target .. "\n"
					or zoxide_target .. "\n"
				return { status = { success = true }, stdout = stdout, stderr = "" }
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
assert(#calls == 1 and #notifications == 1)

role = "workspace-popup"
cx.active.current.hovered = { url = "/current/file", cha = { is_dir = false } }
plugin:entry()
assert(calls[2][2] == cwd)

cx.active.current.hovered = nil
plugin:entry()
assert(calls[3][2] == cwd)

succeeds = false
plugin:entry()
assert(notifications[4].level == "error" and notifications[4].content == "failed")

package.preload["tab-workspace"] = function() return plugin end
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
assert(fzf_options:find("Enter Go here · Alt+Enter Set tab folder", 1, true))
assert(not fzf_options:find("Ctrl+O", 1, true))

role = "startup-picker"
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 2 and fzf_calls == 2 and #calls == workspace_calls)
assert(emitted[#emitted][1] == "cd" and emitted[#emitted][2][1] == zoxide_target)
assert(fzf_options:find("--expect=ctrl-o", 1, true))
assert(fzf_options:find("--bind=alt-enter:ignore", 1, true))
assert(not fzf_options:find("--expect=alt-enter", 1, true))
assert(not fzf_options:find("Alt+Enter", 1, true))
assert(fzf_options:find("Enter Go here · Ctrl+O Open in editor · Tab/Esc Browse Yazi", 1, true))

zoxide_key = "ctrl-o"
local moves = #emitted
search:entry({ args = { source = "tab" } })
assert(zoxide_calls == 3 and fzf_calls == 3 and #calls == workspace_calls + 1)
assert(#calls[#calls] == 1 and calls[#calls][1] == zoxide_target)
assert(#emitted == moves)

zoxide_key = "alt-enter"
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
assert(startup_footer == " Tab/Z Search · Shift+Tab Spot")
assert(emitted[1][1] == "plugin" and emitted[1][2][1] == "quick-search")
assert(footer("workspace-popup") == " Alt+Enter Set tab folder · Tab/Z Search · Shift+Tab Spot")
assert(#emitted == 1)
cx.layer = "input"
assert(Status.render() == "")
