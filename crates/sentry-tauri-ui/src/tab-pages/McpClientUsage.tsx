import { listen } from "@tauri-apps/api/event";
import { PlugZap, Terminal } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { formatRelativeTime } from "@/lib/relativeTime";
import { useMcpUsageStore, type McpClientDetail, type McpToolCall } from "@/store/mcpUsage";

/** Badge label for a client's best-effort `kind` classification. Falls back to the raw
 * value for anything `identify()` on the Rust side didn't recognise, so a new/unclassified
 * client is still legible rather than showing a blank badge. */
const KIND_LABELS: Record<string, string> = {
  "claude-code": "Claude Code",
  "claude-desktop": "Claude Desktop",
  "python-agent": "Python Agent",
  chatgpt: "ChatGPT",
  grok: "Grok",
  unknown: "Unknown",
};

function statusBadge(call: McpToolCall) {
  if (call.status === "protocol_error") {
    return (
      <Badge variant="destructive" title={call.error_message ?? undefined}>
        Protocol error
      </Badge>
    );
  }
  return <Badge variant="outline">Ok</Badge>;
}

function McpClientUsage({ tabId }: { tabId: string }) {
  const clientId = Number(tabId);
  const [detail, setDetail] = useState<McpClientDetail | null>(null);
  const [notFound, setNotFound] = useState(false);
  const getClientUsage = useMcpUsageStore((state) => state.getClientUsage);

  const refresh = useCallback(async () => {
    const next = await getClientUsage(clientId);
    setDetail(next);
    setNotFound(next === null);
  }, [clientId, getClientUsage]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // The MCP server writes calls from another async task as they happen, so it emits an event
  // rather than this tab guessing when to poll — same mechanism as chat's `EVENT_CHAT_UPDATED`.
  useEffect(() => {
    const unlisten = listen<number>("mcp://usage-updated", (event) => {
      if (event.payload === clientId) void refresh();
    });
    return () => {
      void unlisten.then((off) => off());
    };
  }, [clientId, refresh]);

  if (notFound) {
    return (
      <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
        <Empty className="h-full border-none">
          <EmptyHeader>
            <EmptyMedia variant="icon" className="size-10 rounded-xl bg-destructive/10 text-destructive">
              <PlugZap className="size-5" />
            </EmptyMedia>
            <EmptyTitle className="text-lg">Client unavailable</EmptyTitle>
            <EmptyDescription>
              This MCP client no longer exists. It may have been forgotten from the client list.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </div>
    );
  }

  if (detail === null) {
    return (
      <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm" />
    );
  }

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-teal px-4 py-3">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-teal/40 text-navy">
            <PlugZap className="size-4" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h2 className="truncate font-heading text-sm font-semibold text-navy">
                {detail.display_name}
              </h2>
              <Badge variant="secondary">{KIND_LABELS[detail.kind] ?? detail.kind}</Badge>
            </div>
            <p className="truncate text-[11px] text-muted-foreground">
              {detail.protocol_name && `${detail.protocol_name}${detail.protocol_version ? ` v${detail.protocol_version}` : ""} · `}
              {detail.last_process_name
                ? `${detail.last_process_name}${detail.last_pid ? ` (pid ${detail.last_pid})` : ""}`
                : "process unknown"}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-4 text-[11px] text-muted-foreground">
          <span>
            {detail.call_count} {detail.call_count === 1 ? "call" : "calls"}
          </span>
          <span>Last seen {formatRelativeTime(detail.last_seen_ms)}</span>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-4 py-3">
        {detail.calls.length === 0 ? (
          <Empty className="h-full border-none">
            <EmptyHeader>
              <EmptyMedia variant="icon" className="size-10 rounded-xl bg-teal/25 text-navy">
                <Terminal className="size-5" />
              </EmptyMedia>
              <EmptyTitle className="text-lg">No tool calls yet</EmptyTitle>
              <EmptyDescription>
                Recorded the moment this client calls a tool on the MCP server.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Tool</TableHead>
                <TableHead>Process</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Duration</TableHead>
                <TableHead className="text-right">When</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {detail.calls.map((call) => (
                <TableRow key={call.id}>
                  <TableCell className="font-medium text-navy">{call.tool_name}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {call.process_name ? `${call.process_name}${call.pid ? ` (${call.pid})` : ""}` : "—"}
                  </TableCell>
                  <TableCell>{statusBadge(call)}</TableCell>
                  <TableCell className="text-right tabular-nums">{call.duration_ms}ms</TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {formatRelativeTime(call.created_at_ms)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
}

export default McpClientUsage;
