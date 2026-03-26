import SwiftUI

struct BrowserSurfaceView: View {
    var body: some View {
        ZStack {
            Color(nsColor: NSColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1))

            VStack(spacing: 8) {
                Image(systemName: "globe")
                    .font(.system(size: 32))
                    .foregroundStyle(.secondary.opacity(0.5))

                Text("Browser")
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary.opacity(0.5))
            }
        }
    }
}
