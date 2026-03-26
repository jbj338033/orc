import SwiftUI

struct WorkspaceRow: View {
    let workspace: Workspace
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 8) {
            Circle()
                .fill(statusColor)
                .frame(width: 8, height: 8)

            Text(workspace.name)
                .font(.system(size: 13))
                .foregroundStyle(isSelected ? .white : .secondary)
                .lineLimit(1)

            Spacer()

            if workspace.hasUnread {
                Circle()
                    .fill(.blue)
                    .frame(width: 6, height: 6)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(isSelected ? Color.white.opacity(0.1) : .clear)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private var statusColor: Color {
        switch workspace.agentStatus {
        case .idle: .green
        case .thinking: .blue
        case .waiting: .yellow
        case .error: .red
        }
    }
}
