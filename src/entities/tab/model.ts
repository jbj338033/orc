import { type Panel, createLeafPanel } from "@/entities/panel/model";

export interface Tab {
  id: string;
  name: string;
  rootPanel: Panel;
}

export function createTab(name: string): Tab {
  return { id: crypto.randomUUID(), name, rootPanel: createLeafPanel("agent") };
}
