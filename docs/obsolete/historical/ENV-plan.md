## Runtime Environment Configuration Plan

### Goals
- Allow updating environment-derived settings at runtime via console and GraphQL (excluding MCP).
- Persist configuration changes so they survive restarts and rehydrate startup subsystems (chat providers, URLs, timeouts).
- Provide a frontend page that surfaces current settings and lets operators edit them through GraphQL.

### Deliverables & Steps
1. **Backend configuration inventory & storage**
   - Catalog all environment variables that should be runtime-editable (chat provider, model IDs, API keys, timeouts, MCP URL, etc.).
   - Introduce a `system_settings` persistence layer (SeaORM entity + migration) seeded from `.env` defaults.
   - Build a `SystemSettingsService` that loads settings, validates updates, and produces typed configs (`ChatConfig`, etc.).
2. **Runtime refresh + API surfaces**
   - Update startup wiring so services read from `SystemSettingsService`, with watchers or refresh hooks when values change.
   - Extend console commands with a `settings` namespace supporting list/show/set using the new service.
   - Expose GraphQL queries/mutations for listing and updating settings, enforcing validation and refreshing dependent caches.
3. **Frontend management UI**
   - Add GraphQL hooks in the React app for the new schema.
   - Create a System Settings page that lists editable settings, masks sensitive values, and lets users submit updates with optimistic UI/error states.
   - Integrate the page into existing navigation and document usage.

### Tracking
- Update this plan as steps complete or scope shifts.
- Note testing commands (cargo/npm) and schema/UI changes here as work progresses.

### Progress
- [x] Backend storage/service: added `system_settings` table, `SystemSettingsService`, and migrated chat config loaders + credential store to use it.
- [x] Runtime refresh & APIs: wired service into App/GraphQL/console, added console `settings` commands plus GraphQL queries/mutation.
- [x] Frontend UI: shipped GraphQL client ops, dedicated System Settings page, navigation entry, and build verification.
