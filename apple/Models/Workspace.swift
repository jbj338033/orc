import Foundation

enum AgentStatus {
    case idle
    case thinking
    case waiting
    case error
}

struct Workspace: Identifiable {
    let id: UUID
    var name: String
    var tabs: [Tab]
    var agentStatus: AgentStatus
    var hasUnread: Bool

    init(name: String) {
        self.id = UUID()
        self.name = name
        self.tabs = [Tab(name: "Tab 1")]
        self.agentStatus = .idle
        self.hasUnread = false
    }
}
