import SwiftUI

struct AgentInputView: View {
    @Binding var text: String
    var onSend: () -> Void

    var body: some View {
        HStack(alignment: .bottom, spacing: 8) {
            TextEditor(text: $text)
                .font(.system(size: 13))
                .scrollContentBackground(.hidden)
                .frame(minHeight: 20, maxHeight: 120)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .background(Color(nsColor: NSColor(red: 0.15, green: 0.15, blue: 0.15, alpha: 1)))
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .onKeyPress(phases: .down) { press in
                    guard press.key == .return && !press.modifiers.contains(.shift) else {
                        return .ignored
                    }
                    if !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                        onSend()
                    }
                    return .handled
                }

            Button(action: {
                if !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    onSend()
                }
            }) {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.system(size: 24))
                    .foregroundStyle(
                        text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                            ? Color.secondary : Color.blue
                    )
            }
            .buttonStyle(.plain)
            .padding(.bottom, 4)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}
