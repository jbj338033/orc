import SwiftUI

struct SidebarView: View {
    @Bindable var store: AppStore

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(spacing: 2) {
                    ForEach(store.workspaces) { workspace in
                        WorkspaceRow(
                            workspace: workspace,
                            isSelected: workspace.id == store.selectedWorkspaceId
                        )
                        .onTapGesture {
                            store.selectWorkspace(workspace.id)
                        }
                    }
                }
                .padding(.horizontal, 8)
                .padding(.top, 8)
            }

            Spacer()

            Button(action: store.addWorkspace) {
                Image(systemName: "plus")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 10)
            }
            .buttonStyle(.plain)
        }
        .frame(width: 200)
        .background(Color(nsColor: NSColor(red: 0.14, green: 0.14, blue: 0.14, alpha: 1)))
    }
}
