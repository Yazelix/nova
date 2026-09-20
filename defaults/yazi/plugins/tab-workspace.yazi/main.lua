local M = {}

local target_dir = ya.sync(function()
	local current = cx.active.current
	local hovered = current.hovered
	return tostring(hovered and hovered.cha.is_dir and hovered.url or current.cwd)
end)

function M.open(dir)
	local yzx_open = os.getenv("YZX_OPEN")
	if not yzx_open or yzx_open == "" then
		return ya.notify({ title = "Tab workspace", content = "YZX_OPEN is not set", timeout = 5, level = "error" })
	end

	local flag = os.getenv("YZX_YAZI_ROLE") == "startup-picker" and "--retarget-workspace" or "--set-workspace"
	local child, err = Command(yzx_open)
		:arg({ flag, dir })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()
	if not child then
		return ya.notify({ title = "Tab workspace", content = tostring(err), timeout = 5, level = "error" })
	end

	local output, wait_err = child:wait_with_output()
	if not output then
		return ya.notify({ title = "Tab workspace", content = tostring(wait_err), timeout = 5, level = "error" })
	end
	if not output.status.success then
		return ya.notify({ title = "Tab workspace", content = output.stderr, timeout = 5, level = "error" })
	end
	if flag == "--set-workspace" then
		ya.notify({ title = "Tab workspace", content = "Set to " .. dir, timeout = 3, level = "info" })
	end
end

function M:entry()
	M.open(target_dir())
end

return M
