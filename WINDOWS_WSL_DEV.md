# Windows + WSL Development

This project targets Windows, but WSL is still useful for most backend work.

Use WSL for:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
npm --prefix app run typecheck
npm --prefix app run test
npm --prefix app run build
```

Use Windows PowerShell for Tauri/Windows runtime checks:

```powershell
cd D:\Projects\Stellaris-Hunter\app
npm run tauri -- --version
npm run tauri:dev
```

Why: inside WSL, Tauri compiles as a Linux desktop app and needs Linux GTK/WebKit dev libraries. That is separate from validating the intended Windows desktop app.
