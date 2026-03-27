import { useState, useCallback } from "react";
import { useAppStore } from "@/features/workspace-manage/model";
import { Sidebar } from "@/widgets/sidebar/ui";
import { TabBar } from "@/widgets/tab-bar/ui";
import { PanelView } from "@/widgets/panel/ui";
import { useKeyboardShortcuts } from "@/shared/use-keyboard-shortcuts";
import { DragContext } from "@/shared/drag-context";
import type { SurfaceType } from "@/entities/panel/model";

export function App() {
  useKeyboardShortcuts();

  const workspaces = useAppStore((s) => s.workspaces);
  const selectedWsId = useAppStore((s) => s.selectedWorkspaceId);
  const selectedTabId = useAppStore((s) => s.selectedTabId);

  const ws = workspaces.find((w) => w.id === selectedWsId);
  const tab = ws?.tabs.find((t) => t.id === selectedTabId);

  const [drag, setDrag] = useState<{ isDragging: boolean; surfaceType: SurfaceType | null }>({
    isDragging: false,
    surfaceType: null,
  });

  const startDrag = useCallback((type: SurfaceType) => {
    setDrag({ isDragging: true, surfaceType: type });
  }, []);

  const endDrag = useCallback(() => {
    setDrag({ isDragging: false, surfaceType: null });
  }, []);

  return (
    <DragContext.Provider value={{ drag, startDrag, endDrag }}>
      <div className="flex h-screen overflow-hidden rounded-xl bg-[#0c0c0c]">
        <Sidebar />

        <div className="flex min-w-0 flex-1 flex-col border-l border-white/[0.1]">
          <TabBar />

          <div className="min-h-0 flex-1">
            {tab ? (
              <PanelView panel={tab.rootPanel} isRoot />
            ) : (
              <div className="flex h-full items-center justify-center text-[13px] text-neutral-600">
                No workspace selected
              </div>
            )}
          </div>
        </div>
      </div>
    </DragContext.Provider>
  );
}
