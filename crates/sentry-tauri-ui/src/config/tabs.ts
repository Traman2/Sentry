import type { ComponentType } from "react";
import Welcome from "../tab-pages/Welcome";
import TestPage from "../tab-pages/TestPage";

export interface TabConfig {
  type: "welcome" | "test";
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
    type: "test",
    defaultId: "test",
    defaultTitle: "Test Page",
    component: TestPage,
  },
];

export type TabType = TabConfig["type"];
