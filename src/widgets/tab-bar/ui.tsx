import { useAppStore } from "@/features/workspace-manage/model";
import { TabItem } from "@/entities/tab/ui";
import { useDragContext } from "@/shared/drag-context";
import type { SurfaceType } from "@/entities/panel/model";

const DRAG_ITEMS: { type: SurfaceType; icon: string; label: string }[] = [
  { type: "agent", icon: "◆", label: "Agent" },
  { type: "terminal", icon: "⌘", label: "Terminal" },
  { type: "browser", icon: "◎", label: "Browser" },
];

export function TabBar() {
  const workspaces = useAppStore((s) => s.workspaces);
  const selectedWsId = useAppStore((s) => s.selectedWorkspaceId);
  const selectedTabId = useAppStore((s) => s.selectedTabId);
  const selectTab = useAppStore((s) => s.selectTab);
  const addTab = useAppStore((s) => s.addTab);
  const removeTab = useAppStore((s) => s.removeTab);
  const { startDrag, endDrag } = useDragContext();

  const ws = workspaces.find((w) => w.id === selectedWsId);
  if (!ws) return null;

  return (
    <div data-tauri-drag-region className="flex h-12 shrink-0 items-end justify-between px-3 pb-1.5">
      <div className="flex items-center gap-1">
        {ws.tabs.map((tab) => (
          <TabItem
            key={tab.id}
            name={tab.name}
            isSelected={tab.id === selectedTabId}
            canClose={ws.tabs.length > 1}
            onSelect={() => selectTab(tab.id)}
            onClose={() => removeTab(tab.id)}
          />
        ))}
        <button
          onClick={addTab}
          className="flex h-7 w-7 items-center justify-center rounded-md text-[12px] text-neutral-600 transition-colors hover:bg-white/5 hover:text-neutral-400"
        >
          +
        </button>
      </div>

      <div className="flex items-center gap-0.5">
        {DRAG_ITEMS.map((item) => (
          <div
            key={item.type}
            draggable
            onDragStart={(e) => {
              e.dataTransfer.effectAllowed = "move";
              e.dataTransfer.setData("text/plain", item.type);
              startDrag(item.type);
            }}
            onDragEnd={endDrag}
            title={`Drag to split: ${item.label}`}
            className="flex h-7 w-7 cursor-grab items-center justify-center rounded-md text-[11px] text-neutral-600 transition-colors hover:bg-white/5 hover:text-neutral-400 active:cursor-grabbing"
          >
            {item.icon}
          </div>
        ))}
      </div>
    </div>
  );
}
