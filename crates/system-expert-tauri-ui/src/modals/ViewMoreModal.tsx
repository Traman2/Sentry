import { Activity, ArrowDown, ArrowUp, Cpu, MemoryStick, X } from "lucide-react";
import { DialogClose, DialogTitle } from "@/components/ui/dialog";
import type { AppRow } from "@/tab-pages/ResourceMonitor/types";

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-navy/60">{label}</span>
      <span className="font-medium text-navy">{value}</span>
    </div>
  );
}

function ViewMoreModal({ row }: { row: AppRow }) {
  const isApp = row.pid_count > 1;

  const stats = [
    {
      icon: <Cpu className="h-3.5 w-3.5 text-navy/50" />,
      label: "CPU usage",
      value: row.cpu_usage_display,
    },
    {
      icon: <MemoryStick className="h-3.5 w-3.5 text-navy/50" />,
      label: "Memory",
      value: `${row.memory_display} (${row.memory_percent_display})`,
    },
    {
      icon: <ArrowDown className="h-3.5 w-3.5 text-navy/50" />,
      label: "Disk read",
      value: row.disk_read_display,
    },
    {
      icon: <ArrowUp className="h-3.5 w-3.5 text-navy/50" />,
      label: "Disk write",
      value: row.disk_write_display,
    },
  ];

  return (
    <div className="flex max-h-[80vh] w-160 flex-col overflow-hidden rounded-xl border border-teal bg-canvas shadow-xl">
      <div className="flex shrink-0 items-start justify-between border-b border-teal/50 px-6 py-6">
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-teal/40 text-lg font-semibold text-navy">
            {row.name.slice(0, 1).toUpperCase()}
          </div>
          <div>
            <DialogTitle className="text-xl">{row.name}</DialogTitle>
            <p className="mt-0.5 text-sm text-muted-foreground">
              {isApp ? `App · ${row.pid_count} processes` : `Process · PID ${row.pid}`}
            </p>
          </div>
        </div>
        <DialogClose className="cursor-pointer rounded-md p-1.5 text-navy/50 hover:bg-teal/25 hover:text-navy">
          <X className="h-4 w-4" />
        </DialogClose>
      </div>

      {/* This wrapper only provides the left/right inset (aligned with the
          header's px-6) so the inner scrollbar renders at that inset edge
          instead of flush against the card's outer border. The inner div's
          -mr-2.5 (negative margin matching the 10px custom scrollbar width
          from index.css) lets the scrollbar bleed into that reserved
          padding instead of eating into the content width, so the bordered
          boxes below still line up flush with the header's right edge. */}
      <div className="flex min-h-0 flex-1 flex-col px-6">
        <div className="-mr-2.5 min-h-0 flex-1 overflow-y-auto py-6 scrollbar-gutter-stable">
          <div className="overflow-hidden rounded-lg border border-teal/50">
            <table className="w-full text-left text-sm">
              <tbody>
                {stats.map((stat, i) => (
                  <tr
                    key={stat.label}
                    className={i < stats.length - 1 ? "border-b border-teal/30" : ""}
                  >
                    <td className="w-40 px-3 py-2 text-navy/60">
                      <span className="flex items-center gap-1.5">
                        {stat.icon}
                        {stat.label}
                      </span>
                    </td>
                    <td className="px-3 py-2 font-medium text-navy">{stat.value}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="mt-4 flex flex-col gap-2.5 rounded-lg border border-teal/50 bg-teal/5 p-4 text-sm">
            <InfoRow label="Status" value={row.status} />
            <InfoRow label={isApp ? "Primary PID" : "PID"} value={String(row.pid)} />
          </div>

          {isApp && row.subRows && (
            <div className="mt-4">
              <div className="mb-1.5 text-xs font-medium uppercase tracking-wide text-navy/60">
                Processes
              </div>
              {/* No `overflow-hidden` on the wrapper: it would become the
                  scrollport for the sticky header below and pin it to this box
                  instead of the modal's scroll region. The header cells round
                  their own top corners in its place. */}
              <div className="rounded-lg border border-teal/50">
                <table className="w-full border-separate border-spacing-0 text-left text-sm">
                  <thead>
                    <tr>
                      <th className="sticky top-0 z-10 border-b border-teal/50 bg-muted px-3 py-2 text-xs font-medium text-navy/60 first:rounded-tl-lg last:rounded-tr-lg">
                        PID
                      </th>
                      <th className="sticky top-0 z-10 border-b border-teal/50 bg-muted px-3 py-2 text-xs font-medium text-navy/60 first:rounded-tl-lg last:rounded-tr-lg">
                        CPU
                      </th>
                      <th className="sticky top-0 z-10 border-b border-teal/50 bg-muted px-3 py-2 text-xs font-medium text-navy/60 first:rounded-tl-lg last:rounded-tr-lg">
                        Memory
                      </th>
                    </tr>
                  </thead>
                  <tbody className="[&_tr:last-child>td]:border-b-0">
                    {row.subRows.map((sub) => (
                      <tr key={sub.pid}>
                        <td className="border-b border-teal/30 px-3 py-1.5 text-navy">{sub.pid}</td>
                        <td className="border-b border-teal/30 px-3 py-1.5 text-navy">
                          {sub.cpu_usage_display}
                        </td>
                        <td className="border-b border-teal/30 px-3 py-1.5 text-navy">
                          {sub.memory_display}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          <div className="mt-4">
            <div className="mb-1.5 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-navy/60">
              <Activity className="h-3 w-3" />
              Executable path
            </div>
            <div
              className="truncate rounded-md border border-teal/50 bg-navy/5 px-3 py-2 font-mono text-xs text-navy"
              title={row.executable_path ?? undefined}
            >
              {row.executable_path ?? "—"}
            </div>
          </div>
        </div>
      </div>

      <div className="flex shrink-0 justify-end border-t border-teal/50 p-6">
        <DialogClose className="cursor-pointer rounded-md px-4 py-2 text-sm font-medium text-navy/70 hover:bg-teal/25">
          Close
        </DialogClose>
      </div>
    </div>
  );
}

export default ViewMoreModal;
