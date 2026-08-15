import type { ComponentType } from "react";
import { TABS } from "../config/tabs";
import { useTabStore, type TabType } from "../store/tabs";

const TAB_PAGES = Object.fromEntries(
  TABS.map((t) => [t.type, t.component]),
) as Record<TabType, ComponentType<{ tabId: string }>>;

function TabContent() {
  const tabs = useTabStore((state) => state.tabs);
  const activeTabId = useTabStore((state) => state.activeTabId);

  return (
    <div className="relative flex-1 w-full overflow-hidden p-1">
      {tabs.map((tab) => {
        const Page = TAB_PAGES[tab.type];
        return (
          <div
            key={tab.id}
            style={{ display: tab.id === activeTabId ? "block" : "none" }}
            className="h-full w-full overflow-hidden"
          >
            <Page tabId={tab.id} />
          </div>
        );
      })}
    </div>
  );
}

export default TabContent;
