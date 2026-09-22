local M = {}

local target_dir = ya.sync(function()
	local current = cx.active.current
	local hovered = current.hovered
	return tostring(hovered and hovered.cha.is_dir and hovered.url or current.cwd)
end)

function M.open(dir)
	if os.getenv("YZX_YAZI_ROLE") == "startup-picker" then return end
	local yzx_open = os.getenv("YZX_OPEN")
	if not yzx_open or yzx_open == "" then
		return ya.notify({ title = "Tab folder", content = "YZX_OPEN is not set", timeout = 5, level = "error" })
	end

	local output, err = Command(yzx_open)
		:arg({ "--set-workspace", dir })
		:output()
	if not output then
		return ya.notify({ title = "Tab folder", content = tostring(err), timeout = 5, level = "error" })
	end
	if not output.status.success then
		return ya.notify({ title = "Tab folder", content = output.stderr, timeout = 5, level = "error" })
	end
	ya.notify({ title = "Tab folder", content = "Set to " .. dir, timeout = 3, level = "info" })
end

function M:entry()
	M.open(target_dir())
end

return M
