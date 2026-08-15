import type { ComponentType } from "react";
import activityMonitorIcon from "/prim-sidebar-buttons/activity-monitor.svg";
import chatbotIcon from "/prim-sidebar-buttons/chatbot.svg";
import detailsViewIcon from "/prim-sidebar-buttons/details-view.svg";
import mcpViewIcon from "/prim-sidebar-buttons/mcp-view.svg";
import ActivityMonitor from "../primary-sidebars/ActivityMonitor";
import Chatbot from "../primary-sidebars/Chatbot";
import DetailsView from "../primary-sidebars/DetailsView";
import McpClients from "../primary-sidebars/McpClients";

export interface PanelConfig {
  id: "activity-monitor" | "chatbot" | "details-view" | "mcp-clients";
  icon: string;
  label: string;
  component: ComponentType;
}

export const PANELS: PanelConfig[] = [
  {
    id: "activity-monitor",
    icon: activityMonitorIcon,
    label: "Activity Monitor",
    component: ActivityMonitor,
  },
  { id: "chatbot", icon: chatbotIcon, label: "Chatbot", component: Chatbot },
  {
    id: "details-view",
    icon: detailsViewIcon,
    label: "Details View",
    component: DetailsView,
  },
  {
    id: "mcp-clients",
    icon: mcpViewIcon,
    label: "MCP Clients",
    component: McpClients,
  },
];

export type PanelId = PanelConfig["id"];
