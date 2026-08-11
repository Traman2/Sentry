import type { ComponentType } from "react";
import Welcome from "../tab-pages/Welcome";
import TestPage from "../tab-pages/TestPage";
import ResourceMonitor from "../tab-pages/ResourceMonitor";
import ChatSpace from "../tab-pages/ChatSpace";

export interface TabConfig {
  type: "welcome" | "test" | "resource-monitor" | "chat-space";
  defaultId: string;
  defaultTitle: string;
  /** Every tab page receives its owning tab's id, so multi-instance tab types
   * (like chat-space, where several tabs share the same `type`) can tell which
   * instance they're rendering. Singleton tab pages (Welcome, ResourceMonitor,
   * TestPage) simply ignore the prop. */
  component: ComponentType<{ tabId: string }>;
  /** Whether this tab type should appear in the Navbar's "View" menu, which opens
   * it at a single fixed `defaultId`. Multi-instance types like chat-space are
   * opened dynamically instead (see primary-sidebars/Chatbot.tsx) and would be
   * misleading there, so they opt out. */
  showInViewMenu: boolean;
}

export const TABS: TabConfig[] = [
  {
    type: "welcome",
    defaultId: "welcome",
    defaultTitle: "Welcome",
    component: Welcome,
    showInViewMenu: true,
  },
  {
    type: "resource-monitor",
    defaultId: "resource-monitor",
    defaultTitle: "Resource Monitor",
    component: ResourceMonitor,
    showInViewMenu: true,
  },
  {
    type: "test",
    defaultId: "test",
    defaultTitle: "Test Page",
    component: TestPage,
    showInViewMenu: true,
  },
  {
    type: "chat-space",
    defaultId: "chat-space",
    defaultTitle: "New Chat",
    component: ChatSpace,
    showInViewMenu: false,
  },
];

export type TabType = TabConfig["type"];