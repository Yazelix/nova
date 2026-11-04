local M = {}

local cwd = ya.sync(function() return tostring(cx.active.current.cwd) end)

function M:entry(job)
	local role = os.getenv("YZX_YAZI_ROLE")
	local zoxide = job and job.args.source == "zoxide"
	if role ~= "startup-picker" and role ~= "workspace-popup" then
		if zoxide then return ya.emit("plugin", { "zoxide" }) end
		return ya.emit("spot", {})
	end
	if zoxide then return end

	local current = cwd()
	local history, history_err = Command("zoxide")
		:arg({ "query", "--list", "--exclude", current })
		:output()
	if not history then
		return ya.notify({ title = "Quick search", content = tostring(history_err), timeout = 5, level = "error" })
	end
	if not history.status.success then
		return ya.notify({ title = "Quick search", content = history.stderr, timeout = 5, level = "error" })
	end
	if history.stdout == "" then return end

	local permit = ui.hide()
	local popup = role == "workspace-popup"
	local footer = (popup and "Enter Set tab folder" or "Enter Open in editor") .. " · Tab Browse from here · Esc Close"
	local options = "--exact --no-sort --cycle --keep-right --info=inline --layout=reverse --height=100% --border=none --tabstop=1 --exit-0 "
		.. "--expect=tab --bind=enter:accept-non-empty,alt-enter:ignore,ctrl-z:ignore,btab:up --prompt='Go to folder > ' "
		.. "--footer='" .. footer .. "' --footer-border=none --color=footer:-1"
	local child, err = Command("fzf")
		:env("FZF_DEFAULT_OPTS", options)
		:env("FZF_DEFAULT_OPTS_FILE", "")
		:stdin(Command.PIPED)
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()
	if child then
		child:write_all(history.stdout)
		child:flush()
	end
	local output, wait_err = child and child:wait_with_output()
	permit:drop()
	if not output then
		return ya.notify({ title = "Quick search", content = tostring(err or wait_err), timeout = 5, level = "error" })
	end
	if output.status.code == 130 then
		if not popup then return ya.emit("quit", { code = 130, ["no-cwd-file"] = true }) end
		local hidden, hide_err = Command(os.getenv("YZX_YAZI_RETURN")):output()
		if hidden and hidden.status.success then return end
		return ya.notify({ title = "Quick search", content = hidden and hidden.stderr or tostring(hide_err), timeout = 5, level = "error" })
	end
	-- Tab with no matching result leaves Yazi where it is.
	if output.status.code == 1 then return end
	if not output.status.success then
		return ya.notify({ title = "Quick search", content = output.stderr, timeout = 5, level = "error" })
	end

	local key, target = output.stdout:match("^([^\n]*)\n(.*)")
	target = target and target:gsub("\n$", "")
	if not target or target == "" then return end
	if key == "tab" then return ya.emit("cd", { target, raw = true }) end
	require("tab-workspace").open(target)
end

return M
