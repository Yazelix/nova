local M = {}

local cwd = ya.sync(function() return tostring(cx.active.current.cwd) end)

function M:entry()
	local role = os.getenv("YZX_YAZI_ROLE")
	if role ~= "startup-picker" and role ~= "workspace-popup" then return ya.emit("spot", {}) end

	local zoxide = require("zoxide")
	local current = cwd()
	if zoxide.is_empty(current) then return end

	local permit = ui.hide()
	local enter = role == "workspace-popup" and "Enter Browse here" or "Enter Open in Helix"
	local options = "--exact --no-sort --cycle --keep-right --info=inline --layout=reverse --height=100% --border=none --tabstop=1 --exit-0 "
		.. "--bind=enter:accept-non-empty,ctrl-z:ignore,tab:abort,btab:up --prompt='Open folder > ' "
		.. "--footer='" .. enter .. " · Tab/Esc Browse Yazi' --footer-border=none --color=footer:-1"
	local child, err = Command("zoxide")
		:arg({ "query", "-i", "--exclude", current })
		:env("SHELL", "sh")
		:env("CLICOLOR", 1)
		:env("CLICOLOR_FORCE", 1)
		:env("FZF_DEFAULT_OPTS", "")
		:env("FZF_DEFAULT_OPTS_FILE", "")
		:env("_ZO_FZF_OPTS", options)
		:stdin(Command.INHERIT)
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()
	local output, wait_err = child and child:wait_with_output()
	permit:drop()
	if not output then
		return ya.notify({ title = "Quick search", content = tostring(err or wait_err), timeout = 5, level = "error" })
	end
	if output.status.code == 130 then return end
	if not output.status.success then
		return ya.notify({ title = "Quick search", content = output.stderr, timeout = 5, level = "error" })
	end

	local target = output.stdout:gsub("\n$", "")
	if target == "" then return end
	if role == "workspace-popup" then
		ya.emit("cd", { target, raw = true })
	else
		require("tab-workspace").open(target)
	end
end

return M
