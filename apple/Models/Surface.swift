import Foundation

enum SurfaceType {
    case terminal
    case agent
    case browser
}

struct Surface: Identifiable {
    let id: UUID
    var type: SurfaceType

    init(type: SurfaceType) {
        self.id = UUID()
        self.type = type
    }
}
