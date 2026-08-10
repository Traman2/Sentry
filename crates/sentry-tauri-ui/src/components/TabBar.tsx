import { X } from "lucide-react";
import { useTabStore } from "../store/tabs";

function TabBar() {
  const tabs = useTabStore((state) => state.tabs);
  const activeTabId = useTabStore((state) => state.activeTabId);
  const setActiveTab = useTabStore((state) => state.setActiveTab);
  const closeTab = useTabStore((state) => state.closeTab);

  return (
    <div className="flex h-9 w-full items-center gap-1 px-1 pt-1 select-none">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            role="button"
            tabIndex={0}
            onClick={() => setActiveTab(tab.id)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") setActiveTab(tab.id);
            }}
            className={`flex h-7 cursor-pointer items-center gap-1.5 rounded-md border px-3 text-sm font-medium transition-colors ${
              isActive
                ? "bg-canvas shadow-sm border-teal text-navy"
                : "border-transparent text-navy/70 hover:bg-teal/25"
            }`}
          >
            <span>{tab.title}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                closeTab(tab.id);
              }}
              className={`flex h-4 w-4 cursor-pointer items-center justify-center rounded ${
                isActive ? "hover:bg-navy/10" : "hover:bg-teal/40"
              }`}
            >
              <X className="h-3 w-3" />
            </button>
          </div>
        );
      })}
    </div>
  );
}

export default TabBar;
