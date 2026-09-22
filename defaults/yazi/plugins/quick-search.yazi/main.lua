local M = {}

local cwd = ya.sync(function() return tostring(cx.active.current.cwd) end)

function M:entry(job)
	local role = os.getenv("YZX_YAZI_ROLE")
	if role ~= "startup-picker" and role ~= "workspace-popup" then
		if job and job.args.source == "zoxide" then return ya.emit("plugin", { "zoxide" }) end
		return ya.emit("spot", {})
	end

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
	local accept = popup and "--expect=alt-enter " or "--expect=ctrl-o --bind=alt-enter:ignore "
	local footer = popup
		and "Enter Go here · Alt+Enter Set tab folder · Tab/Esc Browse Yazi"
		or "Enter Go here · Ctrl+O Open in editor · Tab/Esc Browse Yazi"
	local options = "--exact --no-sort --cycle --keep-right --info=inline --layout=reverse --height=100% --border=none --tabstop=1 --exit-0 "
		.. accept .. "--bind=enter:accept-non-empty,ctrl-z:ignore,tab:abort,btab:up --prompt='Go to folder > ' "
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
	if output.status.code == 130 then return end
	if not output.status.success then
		return ya.notify({ title = "Quick search", content = output.stderr, timeout = 5, level = "error" })
	end

	local key, target = output.stdout:match("^([^\n]*)\n(.*)")
	target = target and target:gsub("\n$", "")
	if not target or target == "" then return end
	if key == "alt-enter" then return require("tab-workspace").open(target) end
	if key ~= "ctrl-o" then return ya.emit("cd", { target, raw = true }) end

	local yzx_open = os.getenv("YZX_OPEN")
	if not yzx_open or yzx_open == "" then
		return ya.notify({ title = "Open in editor", content = "YZX_OPEN is not set", timeout = 5, level = "error" })
	end

	local editor, editor_err = Command(yzx_open):arg({ target }):output()
	if not editor then
		return ya.notify({ title = "Open in editor", content = tostring(editor_err), timeout = 5, level = "error" })
	end
	if not editor.status.success then
		return ya.notify({ title = "Open in editor", content = editor.stderr, timeout = 5, level = "error" })
	end
end

return M
