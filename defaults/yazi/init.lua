require("auto-layout"):setup()
require("git"):setup()
require("starship"):setup({
	config_file = os.getenv("YZX_YAZI_STARSHIP_CONFIG"),
})
require("zoxide"):setup({
	update_db = true,
})

local role = os.getenv("YZX_YAZI_ROLE")
if role == "startup-picker" or role == "workspace-popup" then
	for id = 1, 6 do
		Status:children_remove(id, id <= 3 and Status.LEFT or Status.RIGHT)
	end
	Status:children_add(function()
		if tostring(cx.layer) ~= "mgr" then return "" end
		if role == "startup-picker" then
			return " Enter Open · Alt+Enter Use this folder · Tab Search · Esc Cancel · F1 Help"
		end
		return " Alt+Enter Workspace · Shift+Z Jump"
	end, 1000, Status.LEFT)
end
