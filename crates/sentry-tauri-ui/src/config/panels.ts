import activityMonitorIcon from "/prim-sidebar-buttons/activity-monitor.svg";
import chatbotIcon from "/prim-sidebar-buttons/chatbot.svg";
import detailsViewIcon from "/prim-sidebar-buttons/details-view.svg";

export interface PanelConfig {
  id: "activity-monitor" | "chatbot" | "details-view";
  icon: string;
  label: string;
}

export const PANELS: PanelConfig[] = [
  { id: "activity-monitor", icon: activityMonitorIcon, label: "Activity Monitor" },
  { id: "chatbot", icon: chatbotIcon, label: "Chatbot" },
  { id: "details-view", icon: detailsViewIcon, label: "Details View" },
];

export type PanelId = PanelConfig["id"];