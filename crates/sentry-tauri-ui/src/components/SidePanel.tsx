import { PANELS } from "../config/panels";
import { usePanelStore } from "../store/panel";

function SidePanel() {
  const activePanel = usePanelStore((state) => state.activePanel);

  if (!activePanel) return null;

  const panel = PANELS.find((p) => p.id === activePanel)!;

  return (
    <div className="flex h-full w-64 flex-none p-1 pr-0">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-lg bg-canvas border border-teal shadow-sm">
        <div className="flex h-9 items-center border-b border-teal px-3 text-sm font-semibold text-navy">
          {panel.label}
        </div>
        <div className="flex-1 overflow-auto p-3 text-sm text-muted-foreground">
          Nothing here yet.
        </div>
      </div>
    </div>
  );
}

export default SidePanel;