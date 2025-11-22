# ファイル仕訳け君 (File Shiwake-kun)

Automatic file organizer based on metadata (EXIF, filesystem attributes).

## Tech Stack

- **Backend**: Rust (Tauri 2.x)
- **Frontend**: React + TypeScript
- **Build Tool**: Vite
- **Dependencies**:
  - `toml` - Configuration file management
  - `kamadak-exif` - EXIF metadata extraction
  - `tracing` - Logging
  - `proptest` - Property-based testing

## Prerequisites

### Windows
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ development tools
- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/)

### macOS
- Xcode Command Line Tools: `xcode-select --install`
- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/)

### Linux
- Build essentials: `sudo apt install build-essential libssl-dev pkg-config`
- GTK3 development libraries: `sudo apt install libgtk-3-dev`
- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/)

## Project Structure

```
.
├── src/                    # React frontend
│   ├── components/         # React components (to be implemented)
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── models/         # Data models
│   │   │   ├── rule.rs
│   │   │   ├── config.rs
│   │   │   ├── metadata.rs
│   │   │   └── process_result.rs
│   │   ├── services/       # Business logic
│   │   │   ├── metadata_extractor.rs
│   │   │   ├── file_operations.rs
│   │   │   ├── rule_engine.rs
│   │   │   ├── file_processor.rs
│   │   │   └── config_manager.rs
│   │   ├── lib.rs
│   │   └── main.rs
│   └── Cargo.toml
└── package.json
```

## Getting Started

1. Install dependencies:
```bash
npm install
```

2. Run in development mode:
```bash
npm run tauri dev
```

3. Build for production:
```bash
npm run tauri build
```

## Development

- Frontend development: `npm run dev`
- Rust checks: `cargo check --manifest-path src-tauri/Cargo.toml`
- Run tests: `cargo test --manifest-path src-tauri/Cargo.toml`

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
