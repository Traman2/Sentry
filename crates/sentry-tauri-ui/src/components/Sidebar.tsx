import { PANELS } from "../config/panels";
import { usePanelStore } from "../store/panel";

function Sidebar() {
  const activePanel = usePanelStore((state) => state.activePanel);
  const togglePanel = usePanelStore((state) => state.togglePanel);

  return (
    <div className="flex h-full w-10 flex-col items-center gap-0.5 bg-canvas border-r border-teal p-1 select-none">
      {PANELS.map(({ id, icon, label }) => {
        const isActive = activePanel === id;
        return (
          <button
            key={id}
            onClick={() => togglePanel(id)}
            className={`flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-navy ${
              isActive ? "bg-teal" : "hover:bg-teal/25 active:bg-teal/40"
            }`}
          >
            <img src={icon} alt={label} className="h-5 w-5" />
          </button>
        );
      })}
    </div>
  );
}

export default Sidebar;