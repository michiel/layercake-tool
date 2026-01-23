# Story Sequences Consolidation Plan

## Goals
- Consolidate story sequences into a single editor at `/project/:projectId/stories/:storyId?tab=sequences`.
- Add a “Details” tab for story name/description/tags; move existing fields there.
- Rework the sequence editor UI:
  - Each sequence is a collapsible “section” with a delete icon.
  - Clicking a section/segment marks it active with clear styling.
  - Clicking an edge adds it to the active section.
  - Diagram preview renders the active section only.
  - Plus icon adds a new section (sequence).

## Technical Notes
- Routing: remove per-sequence sub-routes; sequences live under the story route with `tab=sequences` param. Guard redirects from old URLs to the new tab if possible.
- Story page layout: add `Tabs` with “Details” and “Sequences”. Move form fields (name, description, tags) into Details tab; Sequence tab hosts the editor.
- State management:
  - Maintain `activeSequenceId` in the sequences tab state.
  - Collapse/expand sequences locally; ensure only one active at a time.
  - Edge click handler adds edge to `activeSequenceId`; update local state + mutation.
  - Diagram preview component consumes `activeSequenceId` to render subset.
- API/GraphQL: reuse existing sequence CRUD; add bulk updates as needed for batch edge assignments. Ensure delete sequence mutation is wired to the delete icon.
- UI components:
  - Sequence list: accordion-like list with header (title, counts) + delete icon.
  - Active styling: highlight header + preview; set when header or segment clicked.
  - Add button: plus icon appends a new sequence with default name; opens it active.
  - Edge interactions: hook into existing edge selection handlers to push to active sequence.
  - Preview: filter data to active sequence; fallback message when none active.
- Data loading: preload all sequences for the story when tab is sequences; keep SWR/cache keys scoped to story.

## Risks & Mitigations
- **Navigation regressions**: old sequence sub-routes may break bookmarks. Mitigate with redirects to `?tab=sequences` when detecting legacy params.
- **State sync**: active sequence might be stale after delete. Reset to next available or none.
- **Edge assignment UX**: accidental edge clicks could add to wrong sequence. Add confirm toast/undo or subtle notification; highlight active section clearly.
- **Performance**: rendering many sequences in one view. Use collapsibles and virtualize if needed; lazy-load previews.

## Implementation Steps
1) Routing & Tabs ✅
   - Updated story route to use `tab` param; added “Details” & “Sequences” tabs.
   - Legacy per-sequence route now redirects to `?tab=sequences`.
2) Details Tab ✅
   - Moved name/description/tags form into the Details tab; save flow unchanged.
3) Sequences Tab Layout ✅
   - Accordion-style list of sequences with active state, delete icon, and plus button.
   - Delete wired to sequence delete mutation; active pointer resets.
4) Active Sequence Handling ✅
   - Track `activeSequenceId`; set on section click; clear/reset on delete.
   - Edge click appends to active sequence via update mutation.
5) Diagram Preview ✅
   - Preview button opens diagram for the active sequence.
6) Add New Sequence ✅
   - Plus icon creates a default-named sequence, expands, and activates it.
7) Testing
   - Manual: create/delete sequences, toggle active, add edges, preview, and save; verify Details tab save.
   - Regression: legacy `/sequences/:id` route redirects to sequences tab.

## TODOs / Follow-ups
- Consider undo/toast for accidental edge assignment.
- Add analytics/logging for new UX if desired.
