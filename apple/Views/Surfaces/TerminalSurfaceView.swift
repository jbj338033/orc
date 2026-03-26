import SwiftUI

struct TerminalSurfaceView: View {
    var body: some View {
        ZStack {
            Color(nsColor: NSColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1))

            VStack(spacing: 8) {
                Image(systemName: "terminal")
                    .font(.system(size: 32))
                    .foregroundStyle(.secondary.opacity(0.5))

                Text("Terminal")
                    .font(.system(size: 13, design: .monospaced))
                    .foregroundStyle(.secondary.opacity(0.5))

                Text("libghostty integration pending")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary.opacity(0.3))
            }
        }
    }
}
