import { Activity, FlaskConical } from "lucide-react";
import { useTabStore, type Tab } from "../store/tabs";

const RESOURCE_MONITOR_TAB: Tab = {
  id: "resource-monitor",
  type: "resource-monitor",
  title: "Resource Monitor",
};

const TEST_PAGE_TAB: Tab = {
  id: "test",
  type: "test",
  title: "Test Page",
};

function StartLink({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex cursor-pointer items-center gap-2 text-sm text-navy/80 hover:text-navy"
    >
      {icon}
      {label}
    </button>
  );
}

function Welcome() {
  const openTab = useTabStore((state) => state.openTab);

  return (
    <div className="flex h-full w-full items-center justify-center overflow-auto rounded-lg border border-teal bg-canvas shadow-sm">
      <div className="flex h-125 max-h-full w-340 m-20  max-w-full flex-col p-8">
        <div>
          <div className="text-4xl font-semibold tracking-tight text-navy">
            Sentry Monitor
          </div>
          <div className="mt-0.5 text-lg font-medium text-[#5a9184]">
            Keeping Tabs
          </div>
        </div>

        <div className="mt-10 flex items-start gap-24">
          <div className="flex-1">
            <h2 className="text-lg font-semibold tracking-tight text-navy/60">
              Get started
            </h2>
            <div className="mt-1 flex flex-col gap-2.5">
              <StartLink
                icon={<Activity className="h-4 w-4 text-teal" />}
                label="Open Resource Monitor"
                onClick={() => openTab(RESOURCE_MONITOR_TAB)}
              />
              <StartLink
                icon={<FlaskConical className="h-4 w-4 text-teal" />}
                label="Open Test Page"
                onClick={() => openTab(TEST_PAGE_TAB)}
              />
            </div>
          </div>

          <div className="flex-1">
            <h2 className="text-lg font-semibold tracking-tight text-navy/60">
              Recent
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">No recent items</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Welcome;
