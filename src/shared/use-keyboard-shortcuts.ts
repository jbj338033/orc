import { useEffect } from "react";
import { useAppStore } from "@/features/workspace-manage/model";
import type { Panel } from "@/entities/panel/model";

function findFirstLeafId(panel: Panel): string | null {
  if (panel.node.kind === "leaf") return panel.id;
  if (panel.node.kind === "split" && panel.node.children.length > 0) {
    return findFirstLeafId(panel.node.children[0]);
  }
  return null;
}

function collectLeafIds(panel: Panel, acc: string[] = []): string[] {
  if (panel.node.kind === "leaf") {
    acc.push(panel.id);
  } else if (panel.node.kind === "split") {
    for (const child of panel.node.children) {
      collectLeafIds(child, acc);
    }
  }
  return acc;
}

export function useKeyboardShortcuts() {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const meta = e.metaKey;
      if (!meta) return;

      const state = useAppStore.getState();
      const ws = state.workspaces.find((w) => w.id === state.selectedWorkspaceId);
      const tab = ws?.tabs.find((t) => t.id === state.selectedTabId);

      switch (e.key.toLowerCase()) {
        // Cmd+N — new workspace
        case "n": {
          e.preventDefault();
          state.addWorkspace();
          break;
        }

        // Cmd+T — new tab
        case "t": {
          e.preventDefault();
          state.addTab();
          break;
        }

        // Cmd+D — split right (terminal)
        case "d": {
          e.preventDefault();
          if (!tab) break;
          const leafId = findFirstLeafId(tab.rootPanel);
          if (leafId) {
            const direction = e.shiftKey ? "vertical" : "horizontal";
            state.splitPanel(leafId, direction, "terminal");
          }
          break;
        }

        // Cmd+W — close pane, or close tab if single pane
        case "w": {
          e.preventDefault();
          if (!tab || !ws) break;

          if (tab.rootPanel.node.kind === "leaf") {
            if (ws.tabs.length > 1) {
              state.removeTab(tab.id);
            }
          } else {
            const leaves = collectLeafIds(tab.rootPanel);
            if (leaves.length > 0) {
              state.closePanel(leaves[leaves.length - 1]);
            }
          }
          break;
        }

        // Cmd+Shift+[ / ] — switch tabs
        case "[": {
          e.preventDefault();
          if (!ws) break;
          const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
          if (tabIdx > 0) state.selectTab(ws.tabs[tabIdx - 1].id);
          break;
        }
        case "]": {
          e.preventDefault();
          if (!ws) break;
          const tabIdx = ws.tabs.findIndex((t) => t.id === state.selectedTabId);
          if (tabIdx < ws.tabs.length - 1) state.selectTab(ws.tabs[tabIdx + 1].id);
          break;
        }

        // Cmd+1~9 — switch to nth tab
        default: {
          const num = parseInt(e.key);
          if (num >= 1 && num <= 9 && ws) {
            e.preventDefault();
            const idx = num - 1;
            if (idx < ws.tabs.length) state.selectTab(ws.tabs[idx].id);
          }
          break;
        }
      }
    };

    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);
}
