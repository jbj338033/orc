import type { Workspace } from "./model";

const statusColors: Record<string, string> = {
  idle: "bg-emerald-500",
  thinking: "bg-blue-500",
  waiting: "bg-amber-400",
  error: "bg-red-500",
};

interface WorkspaceRowProps {
  workspace: Workspace;
  isSelected: boolean;
  onSelect: () => void;
  onDelete: () => void;
}

export function WorkspaceRow({ workspace, isSelected, onSelect, onDelete }: WorkspaceRowProps) {
  return (
    <button
      onClick={onSelect}
      onContextMenu={(e) => {
        e.preventDefault();
        onDelete();
      }}
      className={`group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors ${
        isSelected
          ? "bg-white/[0.1] text-white"
          : "text-neutral-400 hover:bg-white/[0.06] hover:text-neutral-200"
      }`}
    >
      <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${statusColors[workspace.agentStatus]}`} />
      <span className="truncate text-[12px] font-medium">{workspace.name}</span>
      {workspace.hasUnread && <span className="ml-auto h-1.5 w-1.5 shrink-0 rounded-full bg-blue-500" />}
    </button>
  );
}
