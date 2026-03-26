import Foundation

struct Tab: Identifiable {
    let id: UUID
    var name: String
    var rootPanel: Panel

    init(name: String) {
        self.id = UUID()
        self.name = name
        self.rootPanel = Panel(node: .leaf(Surface(type: .agent)))
    }
}
