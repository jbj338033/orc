import SwiftUI

struct PanelView: View {
    let panel: Panel
    @Bindable var store: AppStore

    var body: some View {
        switch panel.node {
        case let .leaf(surface):
            LeafPanelView(panel: panel, surface: surface, store: store)

        case let .split(direction, children):
            switch direction {
            case .horizontal:
                HSplitView {
                    ForEach(children) { child in
                        PanelView(panel: child, store: store)
                    }
                }
            case .vertical:
                VSplitView {
                    ForEach(children) { child in
                        PanelView(panel: child, store: store)
                    }
                }
            }
        }
    }
}

private struct LeafPanelView: View {
    let panel: Panel
    let surface: Surface
    @Bindable var store: AppStore

    var body: some View {
        VStack(spacing: 0) {
            PanelHeaderView(
                surface: surface,
                onSplitH: { store.splitPanel(panel.id, direction: .horizontal) },
                onSplitV: { store.splitPanel(panel.id, direction: .vertical) }
            )

            Group {
                switch surface.type {
                case .agent:
                    AgentSurfaceView()
                case .terminal:
                    TerminalSurfaceView()
                case .browser:
                    BrowserSurfaceView()
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

private struct PanelHeaderView: View {
    let surface: Surface
    let onSplitH: () -> Void
    let onSplitV: () -> Void

    var body: some View {
        HStack {
            Image(systemName: surfaceIcon)
                .font(.system(size: 10))
                .foregroundStyle(.secondary)

            Text(surfaceLabel)
                .font(.system(size: 11))
                .foregroundStyle(.secondary)

            Spacer()

            Button(action: onSplitH) {
                Image(systemName: "rectangle.split.1x2")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)

            Button(action: onSplitV) {
                Image(systemName: "rectangle.split.2x1")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(Color(nsColor: NSColor(red: 0.12, green: 0.12, blue: 0.12, alpha: 1)))
    }

    private var surfaceIcon: String {
        switch surface.type {
        case .terminal: "terminal"
        case .agent: "bubble.left"
        case .browser: "globe"
        }
    }

    private var surfaceLabel: String {
        switch surface.type {
        case .terminal: "Terminal"
        case .agent: "Agent"
        case .browser: "Browser"
        }
    }
}
