import SwiftUI

struct MessageBubble: View {
    let message: Message

    var body: some View {
        HStack {
            if message.role == .user { Spacer(minLength: 60) }

            Text(message.content)
                .font(.system(size: 13))
                .foregroundStyle(.white)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(bubbleColor)
                .clipShape(RoundedRectangle(cornerRadius: 12))
                .textSelection(.enabled)

            if message.role == .agent { Spacer(minLength: 60) }
        }
    }

    private var bubbleColor: Color {
        switch message.role {
        case .user: .blue.opacity(0.6)
        case .agent: Color(nsColor: NSColor(red: 0.2, green: 0.2, blue: 0.2, alpha: 1))
        }
    }
}
