import { Eye, Radar } from "lucide-react";
import { Button } from "../../components/ui/button";
import type { ProcessRow } from "./types";

export function SelectionActionBar({
  process,
  onViewMore,
  onTrack,
}: {
  process: ProcessRow;
  onViewMore: (process: ProcessRow) => void;
  onTrack: (process: ProcessRow) => void;
}) {
  return (
    <div className="flex items-center justify-between border-t border-teal bg-teal/10 px-4 py-2">
      <div className="min-w-0 text-xs text-navy">
        <span className="font-medium">{process.name}</span>
        <span className="text-muted-foreground"> · PID {process.pid}</span>
      </div>
      <div className="flex items-center gap-2">
        <Button size="sm" variant="outline" onClick={() => onViewMore(process)}>
          <Eye className="h-3.5 w-3.5" />
          View more
        </Button>
        <Button size="sm" variant="default" onClick={() => onTrack(process)}>
          <Radar className="h-3.5 w-3.5" />
          Track
        </Button>
      </div>
    </div>
  );
}
