import SwiftUI

struct ContentView: View {
    @Bindable var store: AppStore

    var body: some View {
        HStack(spacing: 0) {
            SidebarView(store: store)

            Divider().opacity(0.3)

            VStack(spacing: 0) {
                TabBarView(store: store)

                Divider().opacity(0.3)

                if let ws = store.selectedWorkspace,
                   let tab = ws.tabs.first(where: { $0.id == store.selectedTabId }) {
                    PanelView(panel: tab.rootPanel, store: store)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    emptyState
                }
            }
        }
        .background(Color(nsColor: NSColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1)))
        .preferredColorScheme(.dark)
    }

    private var emptyState: some View {
        ZStack {
            Color(nsColor: NSColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1))
            Text("No workspace selected")
                .font(.system(size: 14))
                .foregroundStyle(.secondary.opacity(0.5))
        }
    }
}
