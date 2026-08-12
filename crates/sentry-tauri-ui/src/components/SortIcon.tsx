import { ArrowDown, ArrowUp, ChevronsUpDown } from "lucide-react";

/** Sort affordance for a tanstack column header — feed it
 * `column.getIsSorted()`. Shows the active direction when sorted, and a muted
 * up/down hint when the column is merely sortable. */
export function SortIcon({ direction }: { direction: false | "asc" | "desc" }) {
  if (direction === "asc") return <ArrowUp className="h-3 w-3 text-navy" />;
  if (direction === "desc") return <ArrowDown className="h-3 w-3 text-navy" />;
  return <ChevronsUpDown className="h-3 w-3 text-navy/30" />;
}

export default SortIcon;
