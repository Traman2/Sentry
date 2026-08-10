import sentryLogo from "/sentry-logo.svg";
import { useTabStore } from "../store/tabs";

const QUICK_COMMANDS = [
  { keys: ["Ctrl", "Shift", "M"], label: "Open Activity Monitor" },
  { keys: ["Ctrl", "Shift", "T"], label: "Open Test Page" },
];

function AppLogo() {
  const hasTabs = useTabStore((state) => state.tabs.length > 0);

  if (hasTabs) return null;

  return (
    <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
      <img src={sentryLogo} alt="" className="h-90 w-auto opacity-10" />
      <div className="flex w-full max-w-90 flex-col gap-1.5 px-6">
        {QUICK_COMMANDS.map((command) => (
          <div key={command.label} className="flex items-center justify-between gap-2.5">
            <span className="text-sm font-medium text-navy/70">{command.label}</span>
            <div className="flex items-center gap-1">
              {command.keys.map((key) => (
                <kbd
                  key={key}
                  className="rounded border border-navy/40 bg-navy/5 px-1.5 py-0.5 text-[11px] font-semibold text-navy/80"
                >
                  {key}
                </kbd>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default AppLogo;
