import SwiftUI

struct TabBarView: View {
    @Bindable var store: AppStore

    var body: some View {
        HStack(spacing: 0) {
            if let ws = store.selectedWorkspace {
                ForEach(ws.tabs) { tab in
                    TabItem(
                        tab: tab,
                        isSelected: tab.id == store.selectedTabId,
                        onSelect: { store.selectTab(tab.id) },
                        onClose: { store.removeTab(tab.id) },
                        canClose: ws.tabs.count > 1
                    )
                }
            }

            Button(action: store.addTab) {
                Image(systemName: "plus")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
            }
            .buttonStyle(.plain)

            Spacer()
        }
        .padding(.horizontal, 4)
        .frame(height: 34)
        .background(Color(nsColor: NSColor(red: 0.12, green: 0.12, blue: 0.12, alpha: 1)))
    }
}

private struct TabItem: View {
    let tab: Tab
    let isSelected: Bool
    let onSelect: () -> Void
    let onClose: () -> Void
    let canClose: Bool

    @State private var isHovering = false

    var body: some View {
        HStack(spacing: 6) {
            Text(tab.name)
                .font(.system(size: 12))
                .foregroundStyle(isSelected ? .white : .secondary)
                .lineLimit(1)

            if canClose && (isSelected || isHovering) {
                Button(action: onClose) {
                    Image(systemName: "xmark")
                        .font(.system(size: 8, weight: .bold))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(isSelected ? Color.white.opacity(0.08) : .clear)
        .clipShape(RoundedRectangle(cornerRadius: 5))
        .onTapGesture(perform: onSelect)
        .onHover { isHovering = $0 }
    }
}
