import Foundation

enum MessageRole {
    case user
    case agent
}

struct Message: Identifiable {
    let id: UUID
    var role: MessageRole
    var content: String
    var timestamp: Date

    init(role: MessageRole, content: String) {
        self.id = UUID()
        self.role = role
        self.content = content
        self.timestamp = .now
    }
}
