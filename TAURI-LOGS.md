## Tauri Log Viewer Plan

### Goals
- Surface backend (Rust) log output directly inside the Tauri desktop UI.
- Stream historical and live log entries into a dedicated “App Logs” page reachable via navigation.
- Keep the standalone web build unchanged (log viewer only activates under Tauri).

### Deliverables & Steps
1. **Plugin wiring**
   - Enable `tauri-plugin-log` on the Rust side with webview output and bridge it into the React app (guarded by `window.__TAURI__`).
   - Provide a front-end helper (hook/service) that subscribes to the plugin stream and buffers messages (timestamp, level, message).
2. **Frontend log component**
   - Build a reusable `AppLogConsole` React component that renders the buffered log list, supports auto-scroll, level colouring, and manual clear/download actions.
   - Ensure the component replays buffered entries (so users see logs emitted before navigation) and listens for new events to append in real time.
3. **Navigation & page**
   - Introduce an “App Logs” route/page in the Tauri desktop navigation (sibling to Database/System settings) that hosts the log console component.
   - Gate the navigation button and page behind the Tauri environment check so the pure web build omits it.
4. **Polish & testing**
   - Add basic search/filter (by level/text) if time permits; otherwise, capture as a follow-up task.
   - Verify behaviour via `npm run tauri:dev`, ensuring existing logs show on load and new entries stream live; confirm standalone `npm run frontend:dev` build is unaffected.

### Tracking
- Update this plan as scope changes or milestones complete.
- Record test commands (tauri, frontend build) once work finishes.

### Progress
- [x] Plugin wiring: added `tauri-plugin-log` backend integration and log stream bootstrap in the React entry-point.
- [x] Frontend log component: implemented a reusable log console with buffering, filtering, clear/download actions, and live updates.
- [x] Navigation & page: exposed an "App Logs" navigation entry (desktop only) and routed page consuming the log stream service.
- [ ] Polish & testing: run a `npm run tauri:dev` smoke test in a full environment; `npm run frontend:build` and `cargo test -p layercake-core --lib` now succeed locally.
