import SwiftUI

struct AgentSurfaceView: View {
    @State private var messages: [Message] = []
    @State private var inputText = ""

    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 8) {
                        ForEach(messages) { message in
                            MessageBubble(message: message)
                                .id(message.id)
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                }
                .onChange(of: messages.count) {
                    if let last = messages.last {
                        withAnimation(.easeOut(duration: 0.2)) {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider().opacity(0.3)

            AgentInputView(text: $inputText) {
                let trimmed = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !trimmed.isEmpty else { return }
                messages.append(Message(role: .user, content: trimmed))
                inputText = ""

                // placeholder echo
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    messages.append(Message(role: .agent, content: "echo: \(trimmed)"))
                }
            }
        }
        .background(Color(nsColor: NSColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1)))
    }
}
