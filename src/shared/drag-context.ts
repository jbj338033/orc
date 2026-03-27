import { createContext, useContext } from "react";
import type { SurfaceType } from "@/entities/panel/model";

interface DragState {
  isDragging: boolean;
  surfaceType: SurfaceType | null;
}

export const DragContext = createContext<{
  drag: DragState;
  startDrag: (type: SurfaceType) => void;
  endDrag: () => void;
}>({
  drag: { isDragging: false, surfaceType: null },
  startDrag: () => {},
  endDrag: () => {},
});

export function useDragContext() {
  return useContext(DragContext);
}
