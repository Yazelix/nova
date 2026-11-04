local M = {}

local target_dir = ya.sync(function()
	local current = cx.active.current
	local hovered = current.hovered
	return tostring(hovered and hovered.cha.is_dir and hovered.url or current.cwd)
end)

-- Picks dir: the startup picker opens it in the editor as the tab folder, the popup sets it as
-- the tab folder.
function M.open(dir)
	local startup = os.getenv("YZX_YAZI_ROLE") == "startup-picker"
	local title = startup and "Open in editor" or "Tab folder"
	local yzx_open = os.getenv("YZX_OPEN")
	if not yzx_open or yzx_open == "" then
		return ya.notify({ title = title, content = "YZX_OPEN is not set", timeout = 5, level = "error" })
	end

	local output, err = Command(yzx_open)
		:arg(startup and { dir } or { "--set-workspace", dir })
		:output()
	if not output then
		return ya.notify({ title = title, content = tostring(err), timeout = 5, level = "error" })
	end
	if not output.status.success then
		return ya.notify({ title = title, content = output.stderr, timeout = 5, level = "error" })
	end
	if not startup then
		ya.notify({ title = "Tab folder", content = "Set to " .. dir, timeout = 3, level = "info" })
	end
end

function M:entry()
	M.open(target_dir())
end

return M
