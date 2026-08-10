import type { ComponentType } from "react";
import activityMonitorIcon from "/prim-sidebar-buttons/activity-monitor.svg";
import chatbotIcon from "/prim-sidebar-buttons/chatbot.svg";
import detailsViewIcon from "/prim-sidebar-buttons/details-view.svg";
import ActivityMonitor from "../primary-sidebars/ActivityMonitor";
import Chatbot from "../primary-sidebars/Chatbot";
import DetailsView from "../primary-sidebars/DetailsView";

export interface PanelConfig {
  id: "activity-monitor" | "chatbot" | "details-view";
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
];

export type PanelId = PanelConfig["id"];
