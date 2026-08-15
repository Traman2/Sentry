import { listen } from "@tauri-apps/api/event";
import { Plug } from "lucide-react";
import { useEffect, useMemo } from "react";
import { PanelEmpty, PanelListItem, PanelSection } from "@/components/PanelList";
import { groupByRecency } from "@/lib/relativeTime";
import { useMcpUsageStore, type McpClient } from "@/store/mcpUsage";
import { useTabStore } from "@/store/tabs";

function McpClients() {
  const clients = useMcpUsageStore((state) => state.clients);
  const loaded = useMcpUsageStore((state) => state.loaded);
  const refresh = useMcpUsageStore((state) => state.refresh);
  const deleteClient = useMcpUsageStore((state) => state.deleteClient);
  const openTab = useTabStore((state) => state.openTab);
  const discardTab = useTabStore((state) => state.discardTab);

  useEffect(() => {
    if (!loaded) refresh();
  }, [loaded, refresh]);

  // Refreshed live whenever a tool call is recorded, so a newly connected client (or a new
  // call from one already open in a tab) appears without waiting on a poll — see
  // `EVENT_MCP_USAGE_UPDATED` in `crates/system-expert-tauri-ui/src-tauri/src/mcp/mod.rs`.
  useEffect(() => {
    const unlisten = listen("mcp://usage-updated", () => void refresh());
    return () => {
      void unlisten.then((off) => off());
    };
  }, [refresh]);

  const groups = useMemo(
    () => groupByRecency(clients, (client) => client.last_seen_ms),
    [clients],
  );

  const openClient = (client: McpClient) => {
    openTab({ id: String(client.id), type: "mcp-client", title: client.display_name });
  };

  const handleDelete = async (client: McpClient) => {
    await deleteClient(client.id);
    discardTab(String(client.id));
  };

  if (clients.length === 0) {
    return (
      <PanelEmpty
        icon={<Plug />}
        title="No MCP clients yet"
        description="Nothing has called a tool on this server's MCP endpoint yet — clients show up here the moment they do."
      />
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {groups.map((group) => (
        <PanelSection key={group.label} label={group.label}>
          {group.items.map((client) => (
            <PanelListItem
              key={client.id}
              title={`${client.display_name} · ${client.call_count}`}
              onOpen={() => openClient(client)}
              onDelete={() => handleDelete(client)}
              deleteLabel={`Forget MCP client "${client.display_name}"`}
            />
          ))}
        </PanelSection>
      ))}
    </div>
  );
}

export default McpClients;
