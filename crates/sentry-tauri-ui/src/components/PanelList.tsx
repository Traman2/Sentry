import { Trash2 } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { cn } from "@/lib/utils";

/** A labelled block inside a side panel. Scrolling is the panel's job, not
 * this component's — nesting a second scroll region inside `SidePanel`'s is
 * what makes sidebar lists feel stuck. */
export function PanelSection({
  label,
  bodyClassName,
  children,
}: {
  label: string;
  /** Override the body's spacing — rows sit flush (`gap-px`), cards need air. */
  bodyClassName?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1">
      <p className="px-2 text-[10px] font-semibold tracking-wide text-navy/50 uppercase">
        {label}
      </p>
      <div className={cn("flex flex-col gap-px", bodyClassName)}>{children}</div>
    </div>
  );
}

/** Live/idle indicator for a list row, in place of an icon. */
export function StatusDot({ live }: { live?: boolean }) {
  return (
    <span className="relative flex size-2 shrink-0">
      {live && (
        <span className="absolute inline-flex size-full animate-ping rounded-full bg-success/60" />
      )}
      <span
        className={cn(
          "relative inline-flex size-2 rounded-full",
          live ? "bg-success" : "bg-navy/25",
        )}
      />
    </span>
  );
}

/** One openable row in a side panel list — a tracked session, a chat space.
 *
 * Follows shadcn's own sidebar-menu shape: the row is a ghost `Button`, and
 * the delete action is a *sibling* button absolutely positioned over its right
 * edge. It can't be a child — nesting a `<button>` inside a `<button>` is
 * invalid HTML and browsers drop the inner one. The row reserves `pr-8` so a
 * truncated title never slides under the action. */
export function PanelListItem({
  leading,
  title,
  onOpen,
  onDelete,
  deleteLabel,
}: {
  /** Optional status dot or icon before the title. */
  leading?: ReactNode;
  title: string;
  onOpen: () => void;
  onDelete: () => void;
  deleteLabel: string;
}) {
  return (
    <div className="group/row relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={onOpen}
        title={title}
        className="h-7 w-full justify-start gap-2 rounded-lg px-2 pr-8 font-normal text-navy hover:bg-teal/15"
      >
        {leading && (
          <span className="flex size-3.5 shrink-0 items-center justify-center text-navy/40">
            {leading}
          </span>
        )}
        <span className="min-w-0 flex-1 truncate text-left text-xs">{title}</span>
      </Button>
      <Button
        variant="ghost"
        size="icon-xs"
        aria-label={deleteLabel}
        onClick={onDelete}
        className="absolute top-1/2 right-1 size-5 -translate-y-1/2 text-navy/40 opacity-0 transition-opacity hover:bg-teal/30 hover:text-navy focus-visible:opacity-100 group-hover/row:opacity-100"
      >
        <Trash2 className="h-3 w-3" />
      </Button>
    </div>
  );
}

/** Placeholder for an empty side panel list, sized down to fit the 256px rail. */
export function PanelEmpty({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <Empty className="gap-0 rounded-lg border border-dashed border-teal/50 px-3 py-5">
      <EmptyHeader className="gap-1">
        <EmptyMedia variant="icon" className="mb-1 size-7 text-navy/50">
          {icon}
        </EmptyMedia>
        <EmptyTitle className="text-xs">{title}</EmptyTitle>
        <EmptyDescription className="text-[11px] leading-relaxed">
          {description}
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
