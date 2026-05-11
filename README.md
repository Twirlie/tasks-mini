# Tasks Mini

A lightweight personal task management desktop app with Kanban-style interface. Built with Rust, Tauri, and Leptos.

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js (for frontend tooling)
- System build tools (cargo, make, etc.)

### Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install frontend dependencies  
npm install
```

### Development

```bash
# Start Tauri dev server (runs both frontend and backend)
cargo tauri dev

# Or start frontend only (if Tauri server already running)
trunk serve
```

### Build

```bash
# Build for release
cargo tauri build

# Frontend only
trunk build
```

## IDE Setup

VS Code + rust-analyzer + Tauri extension recommended.
