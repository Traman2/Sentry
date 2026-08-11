import { Activity, FileText, FlaskConical, MessageSquare } from "lucide-react";
import { useTabStore, type Tab, type TabType } from "../store/tabs";

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

const MAX_LABEL_LENGTH = 18;

function truncateLabel(label: string) {
  return label.length > MAX_LABEL_LENGTH ? `${label.slice(0, MAX_LABEL_LENGTH)}…` : label;
}

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
      title={label}
      className="flex min-w-0 cursor-pointer items-center gap-1.5 text-xs text-navy/80 hover:text-navy"
    >
      {icon}
      <span className="truncate">{truncateLabel(label)}</span>
    </button>
  );
}

function recentIcon(type: TabType) {
  switch (type) {
    case "resource-monitor":
      return <Activity className="h-3.5 w-3.5 shrink-0 text-teal" />;
    case "test":
      return <FlaskConical className="h-3.5 w-3.5 shrink-0 text-teal" />;
    case "chat-space":
      return <MessageSquare className="h-3.5 w-3.5 shrink-0 text-teal" />;
    case "welcome":
      return <FileText className="h-3.5 w-3.5 shrink-0 text-teal" />;
  }
}

function Welcome() {
  const openTab = useTabStore((state) => state.openTab);
  const recent = useTabStore((state) => state.recent);

  return (
    <div className="flex h-full w-full items-center justify-center overflow-hidden rounded-lg border border-teal bg-canvas p-6 pb-24 shadow-sm sm:p-8 sm:pb-32">
      <div className="mx-20 flex max-w-400 flex-1 flex-col">
        <div>
          <p className="text-3xl font-semibold tracking-tight text-navy">
            Sentry Monitor
          </p>
          <div className="mt-0.5 text-base font-medium text-[#5a9184] sm:text-lg">
            Keeping Tabs
          </div>
        </div>

        <div className="mt-6 flex flex-wrap items-start gap-x-12 gap-y-6 sm:mt-10 sm:gap-x-24">
          <div className="min-w-48 flex-1">
            <h2 className="text-lg font-semibold tracking-tight text-navy/60">
              Get started
            </h2>
            <div className="mt-1 flex flex-col gap-1.5">
              <StartLink
                icon={<Activity className="h-3.5 w-3.5 text-teal" />}
                label="Open Resource Monitor"
                onClick={() => openTab(RESOURCE_MONITOR_TAB)}
              />
              <StartLink
                icon={<FlaskConical className="h-3.5 w-3.5 text-teal" />}
                label="Open Test Page"
                onClick={() => openTab(TEST_PAGE_TAB)}
              />
            </div>
          </div>

          <div className="min-w-48 flex-1">
            <h2 className="text-lg font-semibold tracking-tight text-navy/60">
              Recent
            </h2>
            {recent.length === 0 ? (
              <p className="mt-1 text-sm text-muted-foreground">No recent items</p>
            ) : (
              <div className="mt-1 flex flex-col gap-1.5">
                {recent.map((tab) => (
                  <StartLink
                    key={tab.id}
                    icon={recentIcon(tab.type)}
                    label={tab.title}
                    onClick={() => openTab(tab)}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default Welcome;
