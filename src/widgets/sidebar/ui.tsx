import { useAppStore } from "@/features/workspace-manage/model";
import { WorkspaceRow } from "@/entities/workspace/ui";

export function Sidebar() {
  const workspaces = useAppStore((s) => s.workspaces);
  const selectedId = useAppStore((s) => s.selectedWorkspaceId);
  const selectWorkspace = useAppStore((s) => s.selectWorkspace);
  const addWorkspace = useAppStore((s) => s.addWorkspace);
  const removeWorkspace = useAppStore((s) => s.removeWorkspace);

  return (
    <div className="flex h-full w-48 shrink-0 flex-col bg-[#161616]">
      <div data-tauri-drag-region className="h-11 shrink-0" />

      <div className="flex-1 overflow-y-auto px-2">
        <div className="flex flex-col gap-px">
          {workspaces.map((ws) => (
            <WorkspaceRow
              key={ws.id}
              workspace={ws}
              isSelected={ws.id === selectedId}
              onSelect={() => selectWorkspace(ws.id)}
              onDelete={() => removeWorkspace(ws.id)}
            />
          ))}
        </div>
      </div>

      <div className="px-2 pb-2.5">
        <button
          onClick={addWorkspace}
          className="flex w-full items-center justify-center rounded-md py-1.5 text-[11px] text-neutral-600 transition-colors hover:bg-white/[0.04] hover:text-neutral-400"
        >
          +
        </button>
      </div>
    </div>
  );
}
