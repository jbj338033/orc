export type SurfaceType = "terminal" | "agent" | "browser";

export type SplitDirection = "horizontal" | "vertical";

export interface Surface {
  id: string;
  type: SurfaceType;
}

export type PanelNode =
  | { kind: "leaf"; surface: Surface }
  | { kind: "split"; direction: SplitDirection; children: Panel[]; sizes: number[] };

export interface Panel {
  id: string;
  node: PanelNode;
}

export function createSurface(type: SurfaceType): Surface {
  return { id: crypto.randomUUID(), type };
}

export function createLeafPanel(type: SurfaceType): Panel {
  return { id: crypto.randomUUID(), node: { kind: "leaf", surface: createSurface(type) } };
}
