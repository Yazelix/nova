require("auto-layout"):setup()
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZX_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})

local role = os.getenv("YZX_YAZI_ROLE")
if role == "startup-picker" then ya.emit("plugin", { "quick-search" }) end
if role == "startup-picker" or role == "workspace-popup" then
	for id = 1, 6 do
		Status:children_remove(id, id <= 3 and Status.LEFT or Status.RIGHT)
	end
	Status:children_add(function()
		if tostring(cx.layer) ~= "mgr" then return "" end
		if role == "startup-picker" then
			return " Alt+Enter Start here · Tab Search · Shift+Tab Spot"
		end
		return " Alt+Enter Workspace · Tab Search · Shift+Tab Spot"
	end, 1000, Status.LEFT)
end
