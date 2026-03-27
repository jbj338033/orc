import { type Tab, createTab } from "@/entities/tab/model";

export type AgentStatus = "idle" | "thinking" | "waiting" | "error";

export interface Workspace {
  id: string;
  name: string;
  tabs: Tab[];
  agentStatus: AgentStatus;
  hasUnread: boolean;
}

export function createWorkspace(name: string): Workspace {
  return {
    id: crypto.randomUUID(),
    name,
    tabs: [createTab("Tab 1")],
    agentStatus: "idle",
    hasUnread: false,
  };
}
