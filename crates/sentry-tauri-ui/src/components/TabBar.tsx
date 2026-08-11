import { X } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTabStore } from "../store/tabs";

function TabBar() {
  const tabs = useTabStore((state) => state.tabs);
  const activeTabId = useTabStore((state) => state.activeTabId);
  const setActiveTab = useTabStore((state) => state.setActiveTab);
  const closeTab = useTabStore((state) => state.closeTab);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Mirrors VS Code's tab strip: vertical mouse-wheel scroll (the common case for a
  // plain mouse) is redirected to horizontal scroll here. Trackpad users already
  // send meaningful deltaX, so leave those untouched.
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;

    const handleWheel = (e: WheelEvent) => {
      if (Math.abs(e.deltaX) >= Math.abs(e.deltaY)) return;
      e.preventDefault();
      el.scrollLeft += e.deltaY;
    };

    el.addEventListener("wheel", handleWheel, { passive: false });
    return () => el.removeEventListener("wheel", handleWheel);
  }, []);

  return (
    <div
      ref={scrollRef}
      className="scrollbar-none flex h-9 w-full items-center gap-1 overflow-x-auto px-1 pt-1 select-none"
    >
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
            className={`flex h-7 max-w-40 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border px-3 text-sm font-medium transition-colors ${
              isActive
                ? "bg-canvas shadow-sm border-teal text-navy"
                : "border-transparent text-navy/70 hover:bg-teal/25"
            }`}
          >
            <span className="min-w-0 truncate">{tab.title}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                closeTab(tab.id);
              }}
              className={`flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center rounded ${
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
