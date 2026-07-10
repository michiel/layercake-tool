# Plan DAG Editor — Disappearing Nodes & Position Switching

**Date:** 2026-07-10
**Scope:** Root-cause investigation of two reported symptoms in `PlanVisualEditor`:
1. **Disappearing nodes** — deleting one node makes *other* (unrelated) nodes vanish.
2. **Positions switch unexpectedly** — nodes jump/revert to old positions on their own.
**Files:** `frontend/src/components/editors/PlanVisualEditor/{PlanVisualEditor.tsx,hooks/usePlanDagCQRS.ts}`, `frontend/src/adapters/ReactFlowAdapter.ts`, `frontend/src/hooks/useStableReference.ts`.

## Summary

These are **not** the timing bugs fixed in the Horizon 1 pass (that removed the 500 ms echo window and made drag-gating completion-driven). They are **reconciliation / identity** bugs in the layer that merges the plan-DAG state into the ReactFlow node array. There are **five** distinct causes; several compound each other. The two symptoms are each explained by more than one of them, which is why they've been hard to pin down.

The state flow is:

```
planDag (state) ──useMemo──▶ stablePlanDag ──ReactFlowAdapter (cached)──▶ reactFlowData
                                                                              │
                                          useExternalDataChangeDetector ◀─────┤
                                                                              ▼
                                         sync effect: merge reactFlowData into `nodes`
```

Every stage in this flow has a defect.

## Findings (ranked by how directly they cause the symptoms)

### F1 — CRITICAL: Adapter conversion cache key is truncated → returns stale nodes/positions
`ReactFlowAdapter.ts:18-52`. The memoization cache key is:
```
`plandag-${version}-${nodes.length}-${edges.length}-${positionHash}-${edgeHash}`
```
where `positionHash` and `edgeHash` are **`.substring(0, 50)`** of the joined per-node/per-edge strings. Node ids are 36-char UUIDs (`node_371caa7f…`), so 50 chars captures **only the first node's id + position**. Moving any node *other than the first* — with `version`/counts unchanged — produces the **same cache key** → the adapter returns the **stale cached conversion**.

Demonstrated collision (three real node ids, only node 2 & 3 moved):
```
before: "node_371c…badcb:100,200|node"
after : "node_371c…badcb:100,200|node"   ← identical → stale cache hit
```

- **Position switching:** a moved node's new position is silently discarded (stale positions served).
- **Disappearing nodes:** after add/delete, a stale `reactFlowData.nodes` (missing a node, or with an old set) is served into the merge (F2), which then drops live nodes.

This is the single highest-leverage bug: it poisons everything downstream.

### F2 — CRITICAL: Merge drops any live node absent from the (possibly stale/transient) snapshot
`usePlanDagCQRS.ts:370-400`. The merge maps over `currentNodes` and `return null` (→ `filter(Boolean)`) for any node whose id isn't in `newNodesMap` (built from `reactFlowData.nodes`):
```js
const newNode = newNodesMap.get(currentNode.id)
if (!newNode) return null   // node removed from canvas
```
`reactFlowData` can transiently *lack* a legitimately-present node — because of F1 (stale cache), an optimistic add not yet in `stablePlanDag`, or a lagging delta. When the sync fires (and F3/F5 make it fire spuriously), the merge **deletes that node from the canvas even though the user never touched it**. This is the literal "delete one → others disappear" mechanism: the delete triggers a `refreshData()`, whose snapshot momentarily misses another node → that node is dropped.

### F3 — HIGH: Keyboard Delete/Backspace removes a node visually but never persists it
`PlanVisualEditor.tsx:397-438` vs `713-763`, `deleteKeyCode` at `:1981`. `deleteKeyCode={["Delete","Backspace"]}` is enabled, so pressing it emits `remove` node changes into `handleNodesChange`, which calls `onNodesChange(changes)` (visual removal) and then only handles `change.type === 'select'`. **There is no `remove` branch for nodes** — unlike `handleEdgesChange`, which explicitly handles `remove` (`:728`) and calls `mutations.deleteEdge` + `updatePlanDagOptimistically`.

Consequences:
- Keyboard-deleting a node removes it from the canvas but leaves it in `planDag` and the backend.
- The next sync re-adds it (merge "add new nodes" branch, `:403-407`) → **node reappears / flickers**.
- Multi-selecting several nodes + Delete removes them all visually, persists none.

The only correct node-delete path is the in-node trash icon (`deleteHandlerRef`). Keyboard delete is a broken parallel path.

### F4 — HIGH: Optimistic move + `refreshData()` race resets positions to stale server values
`PlanVisualEditor.tsx:handleNodeDragStop` + `usePlanDagCQRS.ts:566-578, 388-399`. Drag-stop optimistically writes the new position into `nodes` and `planDag`, fires `moveNode`, and `.finally(() => setDragging(false))`. `setDragging(false)` with `pendingRefresh` calls `refreshData()`, which does `setPlanDag(serverObject)` — **overwriting** the optimistic planDag (the `stable`/`previous` guard is bypassed on explicit refresh). If the move hasn't round-tripped into that snapshot, `reactFlowData` carries the **old** position, and the merge's `positionChanged` branch writes it back → the node **snaps to its pre-drag position**.

### F5 — MEDIUM: Spurious sync trigger from index-based node comparison
`usePlanDagCQRS.ts:274-286`. The change detector compares `reactFlowData.nodes[idx]` to `nodes[idx]` **by array index**, not id. The two arrays are routinely in different orders (the merge appends new nodes to the end; the adapter emits planDag order). So it compares node A's incoming position/id against node B's current values → `posChanged`/`idChanged` fire spuriously → sync runs when nothing relevant changed, giving F2/F4 more chances to misfire.

### F6 — LOW: `Date.now()`-based optimistic edge ids can collide/duplicate
`PlanVisualEditor.tsx:818` (`id: edge-${Date.now()}`), `:621`. Millisecond resolution; two edges in the same tick collide. The temp edge is never reconciled to the backend id, so `edges` can hold both the temp and backend-id edge for one connection.

## Recommendation

Fix in this order (each is independent; F1–F3 remove the bulk of the symptoms):

1. **F1 — remove the truncation.** Either drop the cache entirely (conversion is cheap and already gated by `stablePlanDag`), or hash the *full* position/edge strings. This alone stops most stale-position / stale-set serving.
2. **F2 — don't drop live nodes on a non-authoritative merge.** Only remove a node when the incoming planDag genuinely no longer contains it; never drop a node that exists locally but is merely absent from a transient/optimistic snapshot. Concretely: keep a node if it's optimistic/newer than the snapshot, and only delete on an authoritative full refresh.
3. **F3 — make keyboard delete go through the real delete path.** Add a `change.type === 'remove'` branch to `handleNodesChange` that routes to `deleteHandlerRef`/`mutations.deleteNode` + `updatePlanDagOptimistically` (mirroring the edge handler), **or** set `deleteKeyCode={null}` and rely on the trash icon. The former is the better UX.
4. **F5 — id-key the change detector** (`currentNodesMap.get(newNode.id)`) instead of `nodes[idx]`.
5. **F4 — don't overwrite in-flight optimistic positions on refresh** (prefer the local pending position until the server confirms the move).
6. **F6 — use `crypto.randomUUID()` for optimistic edge ids** and reconcile temp→backend id.

F1, F2, and F3 are the highest-confidence, highest-impact fixes and directly address both reported symptoms.

## Fixes applied (2026-07-10)

- **F1 — done.** Removed the truncated conversion cache from `ReactFlowAdapter` entirely (conversion is cheap and already gated by `stablePlanDag`). No more stale-conversion serving.
- **F2 — done.** The merge is now non-destructive: a current node missing from the incoming snapshot is **kept**, not dropped. Real deletions are owned by the explicit delete path. This removes the "delete one → others vanish" mechanism.
- **F3 — done.** `handleNodesChange` now routes keyboard (Delete/Backspace) `remove` changes through `handleNodeDelete` (persist + optimistic remove) instead of a visual-only removal that desyncs and flickers.
- **F5 — done.** The change detector matches nodes by id (`Map`) instead of array index, eliminating spurious sync triggers from array-order drift.
- **F6 — done.** Optimistic edge ids use `crypto.randomUUID()` instead of `Date.now()`.
- **F4 — done.** Added a `pendingMoves` map to the editor sync state. `handleNodeDragStop` registers a node's move as in-flight before firing `moveNode` and clears it when the mutation resolves. While a move is pending, the sync merge keeps the local optimistic position (and the change detector ignores the stale server position), so a refresh that races the move no longer snaps the node back. The auto-layout batch-move paths already `await batchMoveNodes` before re-enabling sync, so their race window is closed.
