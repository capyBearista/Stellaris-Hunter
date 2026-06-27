# Contributing to Stellaris Hunter

Thank you for your interest in contributing to Stellaris Hunter! This project is an open-source desktop application designed to help Stellaris players plan achievement-hunting playthroughs.

## Project Structure

The project is a Tauri v2 application with a Rust backend and a React + TypeScript frontend.

- `app/src-tauri/`: Rust backend crate. Handles local discovery, save parsing, launcher SQLite reading, Steam integration, and IPC commands.
- `app/src/`: React + TypeScript + Vite frontend. Provides the desktop UI.
- `catalog/`: The curated achievement catalog JSON and schema.
- `tools/`: Validation scripts and utilities.

## Development Setup

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/).
- **Node.js**: v20.19+ or v22.12+ (required for Vite 8).
- **Tauri CLI**: Installed globally or via `npm run tauri`.
- **Windows**: The primary target platform. Steam integration and local discovery are optimized for Windows.
- **WSL**: Backend development can be done in WSL, but UI and Steam integration testing require Windows.

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/Stellaris-Hunter.git
   cd Stellaris-Hunter
   ```

2. Install frontend dependencies:
   ```bash
   cd app
   npm install
   ```

3. Run the development server:
   ```bash
   npm run tauri:dev
   ```

## Core Guidelines

### 1. Read-Only Constraint (CRITICAL)
This app must **never mutate** external Stellaris/Steam/Documents files.
- Do not write to game installation files, save files, Steam userdata, or mods.
- Open SQLite databases (like `launcher-v2.sqlite`) with the `SQLITE_OPEN_READ_ONLY` flag.
- Steam mutating APIs (`SetAchievement`, `ClearAchievement`, etc.) are strictly forbidden.

### 2. Code Quality
- Use Rust 2021 edition with explicit error handling (`Result<T, Error>`).
- Avoid `unwrap()` in production code paths.
- Keep functions focused and extract complex parsing logic.
- Run `cargo fmt --check` and `cargo clippy -- -D warnings` before submitting a PR.

### 3. Testing
- Include tests for all new parsing/discovery features using synthetic in-repo fixtures.
- Never use live game files or real saves in tests.
- Run the full validation suite before opening a PR:
  ```bash
  bash tools/validate.sh
  ```

## Submitting Changes

1. Fork the repository and create a feature branch (`feature/description`).
2. Ensure your code follows the guidelines and passes all tests.
3. Open a Pull Request with a clear description of the changes.
4. Use Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).

## Catalog Contributions

The achievement catalog (`catalog/latest.json`) is curated. If you find inaccuracies in achievement conditions or want to add missing DLC achievements, please open an issue or PR specifically targeting the catalog JSON.

## License

By contributing, you agree that your contributions will be licensed under the MPL-2.0 License.
