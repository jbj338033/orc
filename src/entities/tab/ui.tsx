import { useState } from "react";

interface TabItemProps {
  name: string;
  isSelected: boolean;
  canClose: boolean;
  onSelect: () => void;
  onClose: () => void;
}

export function TabItem({ name, isSelected, canClose, onSelect, onClose }: TabItemProps) {
  const [hovering, setHovering] = useState(false);

  return (
    <button
      onClick={onSelect}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
      className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-[12px] transition-all duration-150 ${
        isSelected
          ? "bg-white/[0.1] text-white"
          : "text-neutral-500 hover:bg-white/[0.06] hover:text-neutral-300"
      }`}
    >
      <span className="truncate">{name}</span>
      {canClose && (isSelected || hovering) && (
        <span
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          className="flex h-4 w-4 items-center justify-center rounded text-[9px] text-neutral-500 transition-colors hover:bg-white/10 hover:text-neutral-300"
        >
          ✕
        </span>
      )}
    </button>
  );
}
