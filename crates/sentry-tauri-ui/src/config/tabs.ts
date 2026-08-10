import type { ComponentType } from "react";
import Welcome from "../tab-pages/Welcome";
import TestPage from "../tab-pages/TestPage";
import ResourceMonitor from "../tab-pages/ResourceMonitor";

export interface TabConfig {
  type: "welcome" | "test" | "resource-monitor";
  defaultId: string;
  defaultTitle: string;
  component: ComponentType;
}

export const TABS: TabConfig[] = [
  {
    type: "welcome",
    defaultId: "welcome",
    defaultTitle: "Welcome",
    component: Welcome,
  },
  {
    type: "resource-monitor",
    defaultId: "resource-monitor",
    defaultTitle: "Resource Monitor",
    component: ResourceMonitor,
  },
  {
    type: "test",
    defaultId: "test",
    defaultTitle: "Test Page",
    component: TestPage,
  },
];

export type TabType = TabConfig["type"];