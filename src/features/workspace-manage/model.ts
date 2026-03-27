import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Workspace } from "@/entities/workspace/model";
import { createWorkspace } from "@/entities/workspace/model";
import { createTab } from "@/entities/tab/model";
import { splitPanel, closePanel as closePanelHelper } from "@/features/split-panel/model";
import type { Panel, SplitDirection, SurfaceType } from "@/entities/panel/model";

interface AppState {
  workspaces: Workspace[];
  selectedWorkspaceId: string | null;
  selectedTabId: string | null;

  addWorkspace: () => void;
  removeWorkspace: (id: string) => void;
  selectWorkspace: (id: string) => void;

  addTab: () => void;
  removeTab: (id: string) => void;
  selectTab: (id: string) => void;

  splitPanel: (panelId: string, direction: SplitDirection, surfaceType?: SurfaceType) => void;
  closePanel: (panelId: string) => void;
  updatePanelSizes: (panelId: string, sizes: number[]) => void;
  changeSurfaceType: (panelId: string, surfaceType: SurfaceType) => void;
}

function changeSurfaceInPanel(root: Panel, targetId: string, surfaceType: SurfaceType): Panel {
  if (root.id === targetId && root.node.kind === "leaf") {
    return {
      ...root,
      node: { ...root.node, surface: { ...root.node.surface, type: surfaceType } },
    };
  }
  if (root.node.kind === "split") {
    return {
      ...root,
      node: {
        ...root.node,
        children: root.node.children.map((child) => changeSurfaceInPanel(child, targetId, surfaceType)),
      },
    };
  }
  return root;
}

function updateSizesInPanel(root: Panel, targetId: string, sizes: number[]): Panel {
  if (root.id === targetId && root.node.kind === "split") {
    return { ...root, node: { ...root.node, sizes } };
  }
  if (root.node.kind === "split") {
    return {
      ...root,
      node: {
        ...root.node,
        children: root.node.children.map((child) => updateSizesInPanel(child, targetId, sizes)),
      },
    };
  }
  return root;
}

const initialWs = createWorkspace("Workspace 1");

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
  workspaces: [initialWs],
  selectedWorkspaceId: initialWs.id,
  selectedTabId: initialWs.tabs[0].id,

  addWorkspace: () =>
    set((state) => {
      const ws = createWorkspace(`Workspace ${state.workspaces.length + 1}`);
      return {
        workspaces: [...state.workspaces, ws],
        selectedWorkspaceId: ws.id,
        selectedTabId: ws.tabs[0].id,
      };
    }),

  removeWorkspace: (id) =>
    set((state) => {
      const filtered = state.workspaces.filter((ws) => ws.id !== id);
      return {
        workspaces: filtered,
        selectedWorkspaceId: state.selectedWorkspaceId === id ? (filtered[0]?.id ?? null) : state.selectedWorkspaceId,
        selectedTabId: state.selectedWorkspaceId === id ? (filtered[0]?.tabs[0]?.id ?? null) : state.selectedTabId,
      };
    }),

  selectWorkspace: (id) =>
    set((state) => {
      const ws = state.workspaces.find((w) => w.id === id);
      return { selectedWorkspaceId: id, selectedTabId: ws?.tabs[0]?.id ?? null };
    }),

  addTab: () =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const tab = createTab(`Tab ${state.workspaces[wsIdx].tabs.length + 1}`);
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...workspaces[wsIdx], tabs: [...workspaces[wsIdx].tabs, tab] };
      return { workspaces, selectedTabId: tab.id };
    }),

  removeTab: (id) =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const filtered = state.workspaces[wsIdx].tabs.filter((t) => t.id !== id);
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...workspaces[wsIdx], tabs: filtered };
      return {
        workspaces,
        selectedTabId: state.selectedTabId === id ? (filtered[0]?.id ?? null) : state.selectedTabId,
      };
    }),

  selectTab: (id) => set({ selectedTabId: id }),

  splitPanel: (panelId, direction, surfaceType) =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const ws = state.workspaces[wsIdx];
      const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
      if (tabIdx === -1) return state;
      const tab = ws.tabs[tabIdx];
      const newPanel = splitPanel(tab.rootPanel, panelId, direction, surfaceType);
      const tabs = [...ws.tabs];
      tabs[tabIdx] = { ...tab, rootPanel: newPanel };
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...ws, tabs };
      return { workspaces };
    }),

  closePanel: (panelId) =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const ws = state.workspaces[wsIdx];
      const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
      if (tabIdx === -1) return state;
      const tab = ws.tabs[tabIdx];
      const result = closePanelHelper(tab.rootPanel, panelId);
      if (!result) return state; // don't close the last panel
      const tabs = [...ws.tabs];
      tabs[tabIdx] = { ...tab, rootPanel: result };
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...ws, tabs };
      return { workspaces };
    }),

  updatePanelSizes: (panelId, sizes) =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const ws = state.workspaces[wsIdx];
      const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
      if (tabIdx === -1) return state;
      const tab = ws.tabs[tabIdx];
      const newPanel = updateSizesInPanel(tab.rootPanel, panelId, sizes);
      const tabs = [...ws.tabs];
      tabs[tabIdx] = { ...tab, rootPanel: newPanel };
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...ws, tabs };
      return { workspaces };
    }),

  changeSurfaceType: (panelId, surfaceType) =>
    set((state) => {
      const wsIdx = state.workspaces.findIndex((w) => w.id === state.selectedWorkspaceId);
      if (wsIdx === -1) return state;
      const ws = state.workspaces[wsIdx];
      const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
      if (tabIdx === -1) return state;
      const tab = ws.tabs[tabIdx];
      const newPanel = changeSurfaceInPanel(tab.rootPanel, panelId, surfaceType);
      const tabs = [...ws.tabs];
      tabs[tabIdx] = { ...tab, rootPanel: newPanel };
      const workspaces = [...state.workspaces];
      workspaces[wsIdx] = { ...ws, tabs };
      return { workspaces };
    }),
    }),
    {
      name: "orc-app-state",
      version: 1,
      partialize: (state) => ({
        workspaces: state.workspaces,
        selectedWorkspaceId: state.selectedWorkspaceId,
        selectedTabId: state.selectedTabId,
      }),
    },
  ),
);
