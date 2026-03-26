import SwiftUI

@Observable
final class AppStore {
    var workspaces: [Workspace] = []
    var selectedWorkspaceId: UUID?
    var selectedTabId: UUID?

    var selectedWorkspace: Workspace? {
        get { workspaces.first { $0.id == selectedWorkspaceId } }
        set {
            guard let newValue, let idx = workspaces.firstIndex(where: { $0.id == newValue.id }) else { return }
            workspaces[idx] = newValue
        }
    }

    var selectedTab: Tab? {
        guard let ws = selectedWorkspace else { return nil }
        return ws.tabs.first { $0.id == selectedTabId }
    }

    init() {
        let ws = Workspace(name: "Workspace 1")
        workspaces = [ws]
        selectedWorkspaceId = ws.id
        selectedTabId = ws.tabs.first?.id
    }

    func addWorkspace() {
        let ws = Workspace(name: "Workspace \(workspaces.count + 1)")
        workspaces.append(ws)
        selectedWorkspaceId = ws.id
        selectedTabId = ws.tabs.first?.id
    }

    func removeWorkspace(_ id: UUID) {
        workspaces.removeAll { $0.id == id }
        if selectedWorkspaceId == id {
            selectedWorkspaceId = workspaces.first?.id
            selectedTabId = workspaces.first?.tabs.first?.id
        }
    }

    func selectWorkspace(_ id: UUID) {
        selectedWorkspaceId = id
        if let ws = workspaces.first(where: { $0.id == id }) {
            selectedTabId = ws.tabs.first?.id
        }
    }

    func addTab() {
        guard let idx = workspaces.firstIndex(where: { $0.id == selectedWorkspaceId }) else { return }
        let tab = Tab(name: "Tab \(workspaces[idx].tabs.count + 1)")
        workspaces[idx].tabs.append(tab)
        selectedTabId = tab.id
    }

    func removeTab(_ id: UUID) {
        guard let wsIdx = workspaces.firstIndex(where: { $0.id == selectedWorkspaceId }) else { return }
        workspaces[wsIdx].tabs.removeAll { $0.id == id }
        if selectedTabId == id {
            selectedTabId = workspaces[wsIdx].tabs.first?.id
        }
    }

    func selectTab(_ id: UUID) {
        selectedTabId = id
    }

    func splitPanel(_ panelId: UUID, direction: SplitDirection, newSurfaceType: SurfaceType = .terminal) {
        guard let wsIdx = workspaces.firstIndex(where: { $0.id == selectedWorkspaceId }),
              let tabIdx = workspaces[wsIdx].tabs.firstIndex(where: { $0.id == selectedTabId }) else { return }
        workspaces[wsIdx].tabs[tabIdx].rootPanel = splitPanelRecursive(
            workspaces[wsIdx].tabs[tabIdx].rootPanel,
            targetId: panelId,
            direction: direction,
            newSurfaceType: newSurfaceType
        )
    }

    private func splitPanelRecursive(_ panel: Panel, targetId: UUID, direction: SplitDirection, newSurfaceType: SurfaceType) -> Panel {
        if panel.id == targetId {
            if case let .leaf(surface) = panel.node {
                let newSurface = Surface(type: newSurfaceType)
                let left = Panel(node: .leaf(surface))
                let right = Panel(node: .leaf(newSurface))
                return Panel(node: .split(direction, [left, right]))
            }
        }
        if case let .split(dir, children) = panel.node {
            let updated = children.map { splitPanelRecursive($0, targetId: targetId, direction: direction, newSurfaceType: newSurfaceType) }
            var p = panel
            p.node = .split(dir, updated)
            return p
        }
        return panel
    }
}
