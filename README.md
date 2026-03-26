# Orc

Native macOS agent multiplexer. Run multiple AI agents in parallel with split panels, workspaces, and tabs.

## Highlights

- Workspace → Tab → Panel → Surface hierarchy
- Recursive panel splitting (horizontal / vertical)
- Three surface types: Terminal, Agent, Browser
- Rust core with SwiftUI frontend
- Dark mode

## Requirements

- macOS 14+
- Xcode 16+
- Rust toolchain
- [xcodegen](https://github.com/yonaskolb/XcodeGen)

## Build

```bash
xcodegen generate
xcodebuild -project Orc.xcodeproj -scheme Orc build
```

## License

MIT
