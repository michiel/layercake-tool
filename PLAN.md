# Tauri Desktop Application Build Plan

## Overview

Create a standalone desktop application using Tauri that embeds both the frontend (React/Vite) and backend (Rust server) into a single executable. Add a Database menu with functionality to re-initialize the database.

## Current State

- ✅ Tauri scaffolding exists in `src-tauri/`
- ✅ Frontend builds to `frontend/dist`
- ✅ Backend has database migrations in `layercake-core`
- ✅ Basic Tauri commands exist (`get_app_info`, `check_server_status`)
- ⚠️ Backend server not embedded - currently expects external server
- ❌ No database management UI/menu

## Goals

1. Embed the backend server into the Tauri application
2. Start the embedded server automatically when the app launches
3. Add a Database menu with re-initialize functionality
4. Configure build process for all target platforms (Linux, macOS, Windows)
5. Handle graceful shutdown of the embedded server

## Architecture

```
┌─────────────────────────────────────┐
│         Tauri Desktop App           │
│                                     │
│  ┌─────────────┐  ┌──────────────┐ │
│  │   Frontend  │  │   Backend    │ │
│  │ (React/Vite)│←→│  (Axum HTTP) │ │
│  │             │  │              │ │
│  │  Port: Web  │  │  Port: 3030  │ │
│  │  View       │  │  (localhost) │ │
│  └─────────────┘  └──────────────┘ │
│         ↓                ↓          │
│  ┌─────────────────────────────┐   │
│  │   Database (SQLite)         │   │
│  │   ~/.layercake/layercake.db │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

## Implementation Stages

### Stage 1: Embed Backend Server ✅ COMPLETED

**Goal**: Integrate the backend server to run inside the Tauri application

**Status**: ✅ Completed

**Tasks**:
1. Add dependencies to `src-tauri/Cargo.toml`:
   - `layercake-core` with server features
   - `axum`, `tower`, `tower-http` from workspace
   - `sea-orm`, `sea-orm-migration` from workspace

2. Create `src-tauri/src/server.rs`:
   - Function to start embedded server on a thread
   - Return server handle for graceful shutdown
   - Use fixed port (e.g., 3030) for localhost-only access
   - Configure CORS to allow only `tauri://` protocol

3. Update `src-tauri/src/main.rs`:
   - Start embedded server during Tauri setup
   - Store server handle in app state
   - Configure webview to load `http://localhost:3030`
   - Handle server shutdown on app close

4. Update `src-tauri/tauri.conf.json`:
   - Change `devUrl` to use embedded server port
   - Update `beforeDevCommand` to not start separate server
   - Configure security settings for localhost access

**Success Criteria**:
- ✅ Dependencies added to src-tauri/Cargo.toml
- ✅ server.rs module created with embedded server logic
- ✅ main.rs updated to start server during Tauri setup
- ✅ tauri.conf.json configured for embedded server
- ✅ Frontend .env.production configured for localhost:3030
- ✅ Project builds successfully

**Completed Changes**:
- Created `src-tauri/src/server.rs` with `start_embedded_server()` function
- Server runs on port 3030 with CORS for tauri://localhost
- Database migrations run automatically on server startup
- Server handle stored in app state for graceful shutdown
- Frontend production build configured to use http://localhost:3030

**Tests** (To be done in integration testing):
- Start app and verify frontend loads
- Open `/graphql` playground and execute query
- Check logs show server started on correct port
- Close app and verify server shuts down gracefully

### Stage 2: Configure Database Path ✅ COMPLETED

**Goal**: Store database in user-specific application data directory

**Status**: ✅ Completed (implemented in Stage 1)

**Tasks**:
1. ~~Add `tauri-plugin-fs` dependency for file system access~~ - Not needed, used built-in path API
2. ✅ Update `src-tauri/src/main.rs`:
   - Get app data directory using Tauri API
   - Create app data directory if not exists
   - Set database path to platform-specific location
   - Pass database path to server initialization
3. ✅ Add database path to app state for menu commands

**Success Criteria**:
- ✅ Database created in correct user directory
- ✅ Database path stored in app state
- ✅ Directory created automatically if it doesn't exist
- ⏳ Database persists between app restarts (to be tested)
- ⏳ Multiple users can run app independently (to be tested)

**Completed Changes**:
- Database path configured in main.rs setup using `app.path().app_data_dir()`
- Directory created with `std::fs::create_dir_all()`
- Path stored in AppState for later use
- Platform-specific paths handled automatically by Tauri:
  - Linux: `~/.local/share/com.layercake.app/layercake.db`
  - macOS: `~/Library/Application Support/com.layercake.app/layercake.db`
  - Windows: `%APPDATA%\com.layercake.app\layercake.db`

### Stage 3: Add Database Menu and Commands ⏳ IN PROGRESS

**Goal**: Provide UI for database management operations

**Status**: ⏳ In Progress - Commands implemented, menu UI pending

**Tasks**:
1. Create `src-tauri/src/commands/database.rs`:
   - `reinitialize_database()` command
   - `get_database_path()` command
   - `get_database_info()` command (size, table counts, etc.)

2. Implement `reinitialize_database()`:
   - Stop accepting new requests (set maintenance flag)
   - Close all database connections
   - Delete existing database file
   - Create new database file
   - Run all migrations fresh
   - Restart server with new database
   - Clear maintenance flag

3. Add native menu to `src-tauri/src/main.rs`:
   ```rust
   Menu::new()
     .add_submenu("Database", Submenu::new()
       .add_item("Reinitialize Database", "reinit_db")
       .add_item("Show Database Location", "show_db_path")
       .add_separator()
       .add_item("Database Info", "db_info")
     )
   ```

4. Add menu event handlers:
   - Show confirmation dialog before reinitializing
   - Display database path in native dialog
   - Show database statistics in dialog

5. Update app state to include:
   - Database path
   - Server handle
   - Maintenance mode flag

**Success Criteria**:
- ✅ Database commands module created (`commands/database.rs`)
- ✅ `get_database_path()` command implemented
- ✅ `get_database_info()` command implemented (returns path, size, exists)
- ✅ `reinitialize_database()` command implemented
- ✅ `show_database_location()` command implemented (returns directory path)
- ✅ Commands registered in Tauri invoke_handler
- ⏳ Database menu added to app UI (pending)
- ⏳ Menu items trigger commands with confirmation dialogs (pending)

**Completed Changes**:
- Created `src-tauri/src/commands/database.rs` with 4 commands
- Commands can get database info, path, reinitialize database, and show location
- `reinitialize_database()` gracefully shuts down server, deletes DB, and restarts
- All commands use async/await and proper error handling
- Commands registered in main.rs invoke_handler

**Tests**:
- Create test data in database
- Select "Reinitialize Database"
- Confirm in dialog
- Verify data is cleared and migrations run
- Verify app continues to function
- Check database file timestamp is recent

### Stage 4: Build Configuration

**Goal**: Configure Tauri build process for all platforms

**Tasks**:
1. Update `src-tauri/tauri.conf.json`:
   - Configure bundle identifier
   - Set up icons for all platforms
   - Configure code signing (optional for development)
   - Set bundle resources (include any assets needed)
   - Configure installer options

2. Create platform-specific build scripts:
   - `scripts/build-linux.sh`: Linux AppImage/Deb/RPM
   - `scripts/build-macos.sh`: macOS DMG/App bundle
   - `scripts/build-windows.sh`: Windows MSI/NSIS installer

3. Update `package.json` in project root:
   ```json
   "scripts": {
     "tauri:dev": "cargo tauri dev",
     "tauri:build": "cargo tauri build",
     "tauri:build:linux": "./scripts/build-linux.sh",
     "tauri:build:macos": "./scripts/build-macos.sh",
     "tauri:build:windows": "./scripts/build-windows.sh"
   }
   ```

4. Create `.taurignore` to exclude unnecessary files from bundle

5. Configure GitHub Actions for automated builds (optional):
   - Build for Linux x64
   - Build for macOS x64 and ARM64
   - Build for Windows x64
   - Upload artifacts

**Success Criteria**:
- ✅ `cargo tauri dev` runs development version
- ✅ `cargo tauri build` creates production bundle
- ✅ Built app runs without external dependencies
- ✅ App is correctly signed (on macOS)
- ✅ Installer packages are created

**Platform Outputs**:
- Linux: `.AppImage`, `.deb`, `.rpm` in `src-tauri/target/release/bundle/`
- macOS: `.app`, `.dmg` in `src-tauri/target/release/bundle/`
- Windows: `.msi`, `.exe` in `src-tauri/target/release/bundle/`

### Stage 5: Error Handling and Polish

**Goal**: Handle edge cases and improve user experience

**Tasks**:
1. Add error handling:
   - Server fails to start (port in use)
   - Database initialization fails
   - Migration errors
   - Database corruption

2. Add loading screen:
   - Show splash screen while server initializes
   - Display progress messages
   - Show error if startup fails

3. Add logging:
   - Configure `tracing` to log to file in app data directory
   - Add "Show Logs" menu item
   - Implement log rotation

4. Add tray icon (optional):
   - System tray icon with menu
   - "Show/Hide Window" option
   - "Quit" option

5. Add update checking (optional):
   - Check for updates on startup
   - Notify user if update available
   - Link to download page

**Success Criteria**:
- ✅ Meaningful error messages shown to user
- ✅ App doesn't crash on startup errors
- ✅ Loading screen shows during initialization
- ✅ Logs are saved and accessible
- ✅ User can gracefully handle errors

## Testing Plan

### Manual Testing

1. **Fresh Install Test**:
   - Install app on clean system
   - Verify database is created
   - Verify app functions correctly

2. **Upgrade Test**:
   - Install older version
   - Create test data
   - Install new version
   - Verify data persists
   - Verify migrations run

3. **Database Reinitialize Test**:
   - Create test data
   - Use "Reinitialize Database" menu
   - Verify data is cleared
   - Verify app continues to work

4. **Error Handling Test**:
   - Corrupt database file
   - Use database from incompatible version
   - Verify error messages are clear
   - Verify app recovers or fails gracefully

### Automated Testing

1. Add integration tests:
   - Test server starts and responds
   - Test database initialization
   - Test GraphQL queries
   - Test menu commands

2. Add build tests:
   - Verify builds complete on all platforms
   - Verify bundle sizes are reasonable
   - Verify all assets are included

## Dependencies to Add

### `src-tauri/Cargo.toml`

```toml
[dependencies]
layercake-core = { path = "../layercake-core", features = ["server", "graphql"] }
tauri = { workspace = true, features = ["dialog", "fs", "shell"] }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }

# Server dependencies
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }

# Database dependencies
sea-orm = { workspace = true }
sea-orm-migration = { workspace = true }
```

## File Structure After Implementation

```
layercake-tool/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # App entry, menu setup
│   │   ├── server.rs         # Embedded server logic
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── database.rs   # Database commands
│   │   │   └── app.rs        # App info commands
│   │   └── state.rs          # App state management
│   ├── Cargo.toml
│   └── tauri.conf.json       # Updated config
├── frontend/                  # Unchanged
├── layercake-core/           # Unchanged
├── scripts/
│   ├── build-linux.sh
│   ├── build-macos.sh
│   └── build-windows.sh
└── PLAN.md                    # This file
```

## Known Issues and Considerations

1. **Port Conflicts**: If port 3030 is in use, server won't start
   - Solution: Try multiple ports or allow user to configure

2. **Database Locking**: SQLite has write concurrency limitations
   - Current configuration limits connections to 20
   - Should be sufficient for single-user desktop app

3. **File Permissions**: App needs write access to data directory
   - Most platforms handle this automatically
   - May need elevated permissions on first run (Windows)

4. **Code Signing**: Required for macOS distribution
   - Requires Apple Developer account ($99/year)
   - Can skip for development/testing

5. **Update Mechanism**: Tauri updater requires signed releases
   - Consider using GitHub releases
   - Implement auto-update in Stage 5

## Development Commands

```bash
# Run in development mode
cd /home/michiel/dev/layercake-tool
cargo tauri dev

# Build for production
cargo tauri build

# Build specific platform (if on that platform)
./scripts/build-linux.sh
./scripts/build-macos.sh
./scripts/build-windows.sh

# Test embedded server separately
cargo run -p layercake-core -- server --port 3030

# Run migrations manually
cargo run -p layercake-core -- migrate up --database layercake.db
```

## Rollout Strategy

1. **Development**: Implement stages 1-3
2. **Internal Testing**: Build and test on all platforms (Stage 4)
3. **Alpha Release**: Share with select users for feedback
4. **Beta Release**: Public testing with update mechanism
5. **Production Release**: Full release with Stage 5 complete

## Success Metrics

- App launches in < 3 seconds on modern hardware
- Database operations complete in < 100ms
- Bundle size < 50MB per platform
- Zero critical startup failures in testing
- All menu commands work as expected
- Clean shutdown with no data loss

## Next Steps

1. Review and approve this plan
2. Create GitHub issues for each stage
3. Begin implementation with Stage 1
4. Test after each stage before proceeding
5. Update this plan as implementation reveals new requirements
