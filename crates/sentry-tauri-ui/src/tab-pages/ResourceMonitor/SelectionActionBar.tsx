import { Eye, OctagonX, Radar } from "lucide-react";
import { Button } from "../../components/ui/button";
import type { AppRow } from "./types";

function ChatbotIcon({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15" className={className}>
      <path d="M0 0h15v15H0z" fill="none" />
      <path
        fill="currentColor"
        d="M9 2.5V2zm-3 0V3zm6.856 9.422l-.35-.356l-.205.2l.07.277zM13.5 14.5l-.121.485a.5.5 0 0 0 .606-.606zm-4-1l-.354-.354l-.624.625l.857.214zm.025-.025l.353.354a.5.5 0 0 0-.4-.852zM.5 8H0zM7 0v2.5h1V0zm2 2H6v1h3zm6 6a6 6 0 0 0-6-6v1a5 5 0 0 1 5 5zm-1.794 4.279A5.98 5.98 0 0 0 15 7.999h-1a4.98 4.98 0 0 1-1.495 3.567zm.78 2.1L13.34 11.8l-.97.242l.644 2.578zm-4.607-.394l4 1l.242-.97l-4-1zm-.208-.863l-.025.024l.708.707l.024-.024zM9 14q.29 0 .572-.027l-.094-.996A5 5 0 0 1 9 13zm-3 0h3v-1H6zM0 8a6 6 0 0 0 6 6v-1a5 5 0 0 1-5-5zm6-6a6 6 0 0 0-6 6h1a5 5 0 0 1 5-5zm1.5 6A1.5 1.5 0 0 1 6 6.5H5A2.5 2.5 0 0 0 7.5 9zM9 6.5A1.5 1.5 0 0 1 7.5 8v1A2.5 2.5 0 0 0 10 6.5zM7.5 5A1.5 1.5 0 0 1 9 6.5h1A2.5 2.5 0 0 0 7.5 4zm0-1A2.5 2.5 0 0 0 5 6.5h1A1.5 1.5 0 0 1 7.5 5zm0 8c1.064 0 2.042-.37 2.813-.987l-.626-.78c-.6.48-1.359.767-2.187.767zm-2.813-.987c.77.617 1.75.987 2.813.987v-1a3.48 3.48 0 0 1-2.187-.767z"
      />
    </svg>
  );
}

export function SelectionActionBar({
  process,
  onViewMore,
  onTrack,
  onAskAi,
  onKill,
}: {
  process: AppRow;
  onViewMore: (process: AppRow) => void;
  onTrack: (process: AppRow) => void;
  onAskAi: (process: AppRow) => void;
  onKill: (process: AppRow) => void;
}) {
  return (
    <div className="flex items-center justify-between border-t border-teal bg-teal/10 px-4 py-2">
      <div className="min-w-0 text-xs text-navy">
        <span className="font-medium">{process.name}</span>
        <span className="text-muted-foreground"> · PID {process.pid}</span>
      </div>
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          className="hover:bg-navy hover:text-white dark:hover:bg-navy dark:hover:text-white"
          onClick={() => onViewMore(process)}
        >
          <Eye className="h-3.5 w-3.5" />
          View more
        </Button>
        <Button
          size="sm"
          variant="outline"
          className="hover:bg-navy hover:text-white dark:hover:bg-navy dark:hover:text-white"
          onClick={() => onTrack(process)}
        >
          <Radar className="h-3.5 w-3.5" />
          Track
        </Button>
        <Button
          size="sm"
          variant="outline"
          className="hover:bg-navy hover:text-white dark:hover:bg-navy dark:hover:text-white"
          onClick={() => onAskAi(process)}
        >
          <ChatbotIcon className="h-3.5 w-3.5" />
          Ask AI
        </Button>
        <Button
          size="sm"
          variant="destructive"
          className="bg-destructive text-white hover:bg-destructive/90 dark:bg-destructive dark:text-white dark:hover:bg-destructive/90"
          onClick={() => onKill(process)}
        >
          <OctagonX className="h-3.5 w-3.5" />
          Terminate
        </Button>
      </div>
    </div>
  );
}
