import type { Table } from "@tanstack/react-table";
import { Columns3, Search, Upload } from "lucide-react";
import type { ReactNode } from "react";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../../components/ui/dropdown-menu";
import type { ProcessRow } from "./types";

function ToolbarButton({
  icon,
  label,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm text-navy/70 hover:bg-teal/25"
    >
      {icon}
      {label}
    </button>
  );
}

export function Toolbar({ table }: { table: Table<ProcessRow> }) {
  const nameColumn = table.getColumn("name");

  return (
    <div className="flex items-center justify-between border-b border-teal px-4 py-2.5">
      <div className="flex items-center gap-2 rounded-md border border-teal bg-canvas px-2 py-0.5">
        <Search className="h-3.5 w-3.5 text-navy/50" />
        <input
          value={(nameColumn?.getFilterValue() as string) ?? ""}
          onChange={(e) => nameColumn?.setFilterValue(e.target.value)}
          placeholder="Search processes…"
          className="w-48 bg-transparent text-sm text-navy outline-none placeholder:text-navy/40"
        />
      </div>
      <div className="flex items-center gap-1">
        <DropdownMenu>
          <DropdownMenuTrigger className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm text-navy/70 hover:bg-teal/25">
            <Columns3 className="h-3.5 w-3.5" />
            Columns
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuGroup>
              <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {table.getAllLeafColumns().map((column) => (
                <DropdownMenuCheckboxItem
                  key={column.id}
                  checked={column.getIsVisible()}
                  onCheckedChange={(checked) => column.toggleVisibility(!!checked)}
                >
                  {String(column.columnDef.header)}
                </DropdownMenuCheckboxItem>
              ))}
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <ToolbarButton icon={<Upload className="h-3.5 w-3.5" />} label="Export" />
      </div>
    </div>
  );
}
