import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

function toCsv(header: string[], rows: string[][]): string {
  return [header, ...rows]
    .map((row) => row.map((cell) => `"${cell.replace(/"/g, '""')}"`).join(","))
    .join("\n");
}

/**
 * Opens a native save dialog for a CSV file, writes it, and opens the saved
 * file with the OS default handler. No-ops if the user cancels the dialog.
 */
async function saveCsv(defaultPath: string, header: string[], rows: string[][]) {
  const path = await save({
    defaultPath,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!path) return;

  await invoke("write_and_open_file", { path, contents: toCsv(header, rows) });
}

export { toCsv, saveCsv };