import Foundation

enum SplitDirection {
    case horizontal
    case vertical
}

enum PanelNode {
    case leaf(Surface)
    case split(SplitDirection, [Panel])
}

struct Panel: Identifiable {
    let id: UUID
    var node: PanelNode

    init(node: PanelNode) {
        self.id = UUID()
        self.node = node
    }
}
