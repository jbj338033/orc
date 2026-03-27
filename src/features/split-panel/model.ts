import type { Panel, SplitDirection, SurfaceType } from "@/entities/panel/model";
import { createLeafPanel } from "@/entities/panel/model";

export function splitPanel(
  root: Panel,
  targetId: string,
  direction: SplitDirection,
  newSurfaceType: SurfaceType = "terminal",
): Panel {
  if (root.id === targetId && root.node.kind === "leaf") {
    return {
      id: crypto.randomUUID(),
      node: {
        kind: "split",
        direction,
        children: [
          { id: crypto.randomUUID(), node: root.node },
          createLeafPanel(newSurfaceType),
        ],
        sizes: [50, 50],
      },
    };
  }
  if (root.node.kind === "split") {
    return {
      ...root,
      node: {
        ...root.node,
        children: root.node.children.map((child) =>
          splitPanel(child, targetId, direction, newSurfaceType),
        ),
      },
    };
  }
  return root;
}

export function closePanel(root: Panel, targetId: string): Panel | null {
  if (root.id === targetId) return null;

  if (root.node.kind === "split") {
    const updated = root.node.children.map((child) => closePanel(child, targetId));

    // child가 제거됨 → 형제가 부모 자리를 대체
    const remaining = updated.filter((c): c is Panel => c !== null);
    if (remaining.length === 0) return null;
    if (remaining.length === 1) return remaining[0];

    // 2개 이상 남음 → sizes 재계산
    const oldIndices = updated
      .map((c, i) => (c !== null ? i : -1))
      .filter((i) => i !== -1);
    const remainingSizes = oldIndices.map((i) => root.node.kind === "split" ? root.node.sizes[i] : 50);
    const total = remainingSizes.reduce((a, b) => a + b, 0);
    const normalizedSizes = remainingSizes.map((s) => (s / total) * 100);

    return {
      ...root,
      node: {
        ...root.node,
        children: remaining,
        sizes: normalizedSizes,
      },
    };
  }
  return root;
}
