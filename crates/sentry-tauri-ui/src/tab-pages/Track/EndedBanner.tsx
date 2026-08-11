import { AlertTriangle } from "lucide-react";

export function EndedBanner({ reason }: { reason: "died" | "manual" | null }) {
  return (
    <div className="flex shrink-0 items-center gap-2 border-b border-teal bg-amber-50 px-4 py-2 text-xs text-amber-800">
      <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
      {reason === "died" ? "Tracking ended — process is no longer active." : "Tracking ended."}
    </div>
  );
}
