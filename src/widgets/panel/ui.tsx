import { useRef, useCallback, useState, useEffect, type ReactNode } from "react";
import type { Panel, SurfaceType } from "@/entities/panel/model";
import { useAppStore } from "@/features/workspace-manage/model";
import { AgentSurface } from "@/features/agent-chat/ui";
import { TerminalSurface } from "@/features/terminal/ui";
import { useDragContext } from "@/shared/drag-context";

const MIN_SIZE_PCT = 10;

function BrowserPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center text-neutral-700">
      <div className="text-center">
        <div className="text-3xl opacity-40">◎</div>
        <div className="mt-2 text-[12px] opacity-40">Browser</div>
      </div>
    </div>
  );
}

// --- Drop zone overlay shown during drag ---

type DropPosition = "left" | "right" | "top" | "bottom";

function DropZoneOverlay({
  panelId,
  onDrop,
}: {
  panelId: string;
  onDrop: (panelId: string, direction: "horizontal" | "vertical") => void;
}) {
  const [hover, setHover] = useState<DropPosition | null>(null);

  const zones: { pos: DropPosition; direction: "horizontal" | "vertical"; style: string; indicator: string }[] = [
    { pos: "left", direction: "horizontal", style: "left-0 top-0 w-1/4 h-full", indicator: "left-0 top-0 h-full w-1/2" },
    { pos: "right", direction: "horizontal", style: "right-0 top-0 w-1/4 h-full", indicator: "right-0 top-0 h-full w-1/2" },
    { pos: "top", direction: "vertical", style: "left-0 top-0 w-full h-1/4", indicator: "left-0 top-0 w-full h-1/2" },
    { pos: "bottom", direction: "vertical", style: "left-0 bottom-0 w-full h-1/4", indicator: "left-0 bottom-0 w-full h-1/2" },
  ];

  return (
    <div className="absolute inset-0 z-30">
      {/* highlight indicator */}
      {hover && (
        <div
          className={`absolute ${zones.find((z) => z.pos === hover)!.indicator} rounded-sm bg-blue-500/15 border-2 border-blue-500/40 pointer-events-none transition-all duration-75`}
        />
      )}

      {/* invisible drop zones */}
      {zones.map((zone) => (
        <div
          key={zone.pos}
          className={`absolute ${zone.style}`}
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = "move";
            setHover(zone.pos);
          }}
          onDragLeave={() => setHover((prev) => (prev === zone.pos ? null : prev))}
          onDrop={(e) => {
            e.preventDefault();
            setHover(null);
            onDrop(panelId, zone.direction);
          }}
        />
      ))}
    </div>
  );
}

// --- Context menu (simplified: close + change surface type) ---

interface ContextMenuState {
  x: number;
  y: number;
  panelId: string;
  surfaceId: string;
  currentType: SurfaceType;
  isRoot: boolean;
}

function PanelContextMenu({
  menu,
  onClose,
}: {
  menu: ContextMenuState;
  onClose: () => void;
}) {
  const doClose = useAppStore((s) => s.closePanel);
  const changeSurface = useAppStore((s) => s.changeSurfaceType);

  useEffect(() => {
    const handler = () => onClose();
    document.addEventListener("click", handler);
    return () => document.removeEventListener("click", handler);
  }, [onClose]);

  const surfaceTypes: { label: string; type: SurfaceType }[] = [
    { label: "Agent", type: "agent" },
    { label: "Terminal", type: "terminal" },
    { label: "Browser", type: "browser" },
  ];

  return (
    <div
      className="fixed z-50 min-w-[140px] rounded-lg border border-white/[0.1] bg-[#1a1a1a] py-1 shadow-xl"
      style={{ left: menu.x, top: menu.y }}
    >
      {surfaceTypes
        .filter((s) => s.type !== menu.currentType)
        .map((s) => (
          <button
            key={s.type}
            onClick={() => { changeSurface(menu.panelId, s.type); onClose(); }}
            className="flex w-full px-3 py-1.5 text-left text-[12px] text-neutral-300 hover:bg-white/[0.08]"
          >
            Change to {s.label}
          </button>
        ))}
      {!menu.isRoot && (
        <>
          <div className="mx-2 my-1 h-px bg-white/[0.06]" />
          <button
            onClick={() => { doClose(menu.panelId); onClose(); }}
            className="flex w-full px-3 py-1.5 text-left text-[12px] text-red-400/80 hover:bg-white/[0.08]"
          >
            Close Panel
          </button>
        </>
      )}
    </div>
  );
}

// --- Resize handle ---

interface ResizeHandleProps {
  direction: "horizontal" | "vertical";
  panelId: string;
  index: number;
  sizes: number[];
}

function ResizeHandle({ direction, panelId, index, sizes }: ResizeHandleProps) {
  const updatePanelSizes = useAppStore((s) => s.updatePanelSizes);
  const handleRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      const startPos = direction === "horizontal" ? e.clientX : e.clientY;
      const startSizes = [...sizes];

      const container = handleRef.current?.parentElement;
      if (!container) return;

      const totalPx =
        direction === "horizontal" ? container.offsetWidth : container.offsetHeight;

      const onMouseMove = (ev: MouseEvent) => {
        const currentPos = direction === "horizontal" ? ev.clientX : ev.clientY;
        const deltaPct = ((currentPos - startPos) / totalPx) * 100;

        let left = startSizes[index] + deltaPct;
        let right = startSizes[index + 1] - deltaPct;

        if (left < MIN_SIZE_PCT) {
          right -= MIN_SIZE_PCT - left;
          left = MIN_SIZE_PCT;
        }
        if (right < MIN_SIZE_PCT) {
          left -= MIN_SIZE_PCT - right;
          right = MIN_SIZE_PCT;
        }

        const next = [...startSizes];
        next[index] = left;
        next[index + 1] = right;
        updatePanelSizes(panelId, next);
      };

      const onMouseUp = () => {
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };

      document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
      document.body.style.userSelect = "none";
      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    },
    [direction, panelId, index, sizes, updatePanelSizes],
  );

  return (
    <div
      ref={handleRef}
      onMouseDown={handleMouseDown}
      className={`group relative z-10 shrink-0 ${
        direction === "horizontal"
          ? "w-1 cursor-col-resize"
          : "h-1 cursor-row-resize"
      }`}
    >
      <div
        className={`absolute ${
          direction === "horizontal"
            ? "inset-y-0 left-1/2 w-px -translate-x-1/2 group-hover:w-0.5"
            : "inset-x-0 top-1/2 h-px -translate-y-1/2 group-hover:h-0.5"
        } bg-white/[0.06] transition-all duration-100 group-hover:bg-blue-500/60`}
      />
    </div>
  );
}

// --- Main PanelView ---

interface PanelViewProps {
  panel: Panel;
  isRoot?: boolean;
}

export function PanelView({ panel, isRoot = false }: PanelViewProps) {
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const { drag, endDrag } = useDragContext();
  const doSplit = useAppStore((s) => s.splitPanel);

  const handleDrop = useCallback(
    (panelId: string, direction: "horizontal" | "vertical") => {
      if (drag.surfaceType) {
        doSplit(panelId, direction, drag.surfaceType);
      }
      endDrag();
    },
    [drag.surfaceType, doSplit, endDrag],
  );

  if (panel.node.kind === "leaf") {
    const { surface } = panel.node;
    return (
      <div
        className="relative h-full min-h-0 min-w-0 flex-1"
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
          setContextMenu({
            x: e.clientX,
            y: e.clientY,
            panelId: panel.id,
            surfaceId: surface.id,
            currentType: surface.type,
            isRoot,
          });
        }}
      >
        {surface.type === "agent" && <AgentSurface surfaceId={surface.id} />}
        {surface.type === "terminal" && <TerminalSurface surfaceId={surface.id} />}
        {surface.type === "browser" && <BrowserPlaceholder />}

        {drag.isDragging && (
          <DropZoneOverlay panelId={panel.id} onDrop={handleDrop} />
        )}

        {contextMenu && (
          <PanelContextMenu menu={contextMenu} onClose={() => setContextMenu(null)} />
        )}
      </div>
    );
  }

  const { direction, children, sizes } = panel.node;

  const items: ReactNode[] = [];
  for (let i = 0; i < children.length; i++) {
    if (i > 0) {
      items.push(
        <ResizeHandle
          key={`handle-${children[i].id}`}
          direction={direction}
          panelId={panel.id}
          index={i - 1}
          sizes={sizes}
        />,
      );
    }
    items.push(
      <div
        key={children[i].id}
        className="min-h-0 min-w-0 overflow-hidden"
        style={{ flex: `${sizes[i]} 1 0%` }}
      >
        <PanelView panel={children[i]} />
      </div>,
    );
  }

  return (
    <div
      className={`flex h-full min-h-0 min-w-0 flex-1 ${
        direction === "horizontal" ? "flex-row" : "flex-col"
      }`}
    >
      {items}
    </div>
  );
}
