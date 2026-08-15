import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export interface StatSpec {
  label: string;
  value: string;
  /** Color key dot tying this figure to its line on the chart. Only set where
   * the chart actually has more than one line to tell apart. */
  accent?: string;
}

/** The row of summary figures above each metric's chart. */
export function StatCards({ stats }: { stats: StatSpec[] }) {
  return (
    <div className="grid shrink-0 gap-3 sm:grid-cols-3">
      {stats.map((stat) => (
        <Card key={stat.label} size="sm" className="rounded-lg ring-teal/50">
          <CardHeader>
            <CardTitle className="flex items-center gap-1.5 text-[10px] font-semibold tracking-wide text-navy/60 uppercase">
              {stat.accent && (
                <span
                  className="h-2 w-2 shrink-0 rounded-full"
                  style={{ backgroundColor: stat.accent }}
                />
              )}
              {stat.label}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="font-heading text-xl font-semibold text-navy tabular-nums">
              {stat.value}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
