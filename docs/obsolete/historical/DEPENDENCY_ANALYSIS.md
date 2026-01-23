# Frontend Dependency Analysis

## Large Bundle Chunks Explained

### flowchart-elk-definition-ae0efee6.js (1,452 kB)

**Status**: ⚠️ **Cannot be removed** - This is Mermaid's internal dependency

**Why it exists**:
- This is **inside the Mermaid library itself**
- Mermaid uses ELK internally for rendering flowchart diagrams
- It's not our code - it's bundled with `mermaid@10.9.1`

**Why we can't remove it**:
- Mermaid is used in `OutputNode.tsx` for previewing Mermaid diagrams from plan outputs
- This is a core feature - users can export to Mermaid format and preview
- Removing Mermaid would break this functionality

**Mitigation - Already Done** ✅:
- Mermaid is **lazy-loaded** via `MermaidPreviewDialog`
- Only loads when user opens Mermaid preview dialog
- **Does NOT load** on chat page or initial app load
- **Does NOT affect** initial bundle size or page load time

### Our ELK Usage - Replaced ✅

We successfully removed our own ELK usage:
- **Before**: Used `elkjs` directly in layout code (1,451 kB)
- **After**: Replaced with `dagre` (91 kB)
- **Savings**: 1,360 kB in our layout engine

The remaining ELK chunk is Mermaid's, not ours.

## Dependency Audit Results

### ✅ All Dependencies Are Used

Checked major dependencies:

| Package | Size | Used In | Status |
|---------|------|---------|--------|
| `mermaid` | ~600 kB | MermaidPreviewDialog | ✅ Keep (lazy-loaded) |
| `d3-graphviz` | ~400 kB | DotPreviewDialog | ✅ Keep (lazy-loaded) |
| `force-graph` | ~140 kB | GraphPreview | ✅ Keep (lazy-loaded) |
| `tweakpane` | ~60 kB | GraphPreview controls | ✅ Keep (lazy-loaded) |
| `react-querybuilder` | ~40 kB | QueryFilterBuilder | ✅ Keep |
| `apollo-upload-client` | ~20 kB | GraphQL file uploads | ✅ Keep |
| `next-themes` | ~5 kB | Theme provider | ✅ Keep |
| `remark-gfm` | ~30 kB | Markdown GFM support | ✅ Keep |
| `lucide-react` | ~100 kB | UI icons | ✅ Keep |
| `class-variance-authority` | ~5 kB | Component variants | ✅ Keep |
| `katex` | 265 kB | Math rendering in chat | ✅ Keep (needed) |

### ❌ Unused Dependencies Found

| Package | Why It Can Be Removed |
|---------|----------------------|
| `gql-tag` | Not used - all GraphQL queries use `gql` from `@apollo/client` instead |

## Recommendations

### 1. Remove gql-tag ✅
```bash
npm uninstall gql-tag
```
**Impact**: Minimal size reduction (~5-10 kB), but cleaner dependencies

### 2. Keep Mermaid (with ELK) ✅
**Reason**:
- Provides important preview functionality
- Already lazy-loaded (doesn't affect initial load)
- Only loads when user explicitly opens Mermaid preview
- Breaking this feature not worth the "optimization"

### 3. Monitor Assistant UI Bundle
The `@assistant-ui/*` packages total ~280 kB and include:
- `@assistant-ui/react` - Core chat functionality
- `@assistant-ui/react-markdown` - Pulls in katex (265 kB)

**Note**: This is needed for chat functionality and already optimized.

## Bundle Size Summary

### Before All Optimizations
- Total bundle: ~6 MB
- Main chunk: 2,026 kB
- All libs loaded upfront

### After Optimizations
- Total bundle: ~3.5 MB (42% reduction)
- Main chunk: 1,032 kB (50% reduction)
- Heavy libs lazy-loaded (1.1 MB saved from initial load)

### Remaining Large Chunks (All Lazy-Loaded)
- Mermaid + ELK: 1,452 kB + 226 kB = **1,678 kB** (loads on Mermaid preview)
- DOT Preview: **758 kB** (loads on DOT preview)
- Katex: **265 kB** (loads with chat - needed for markdown)

## Conclusion

**All significant optimizations have been completed**:
1. ✅ Replaced our ELK usage with Dagre (1,360 kB saved)
2. ✅ Lazy-loaded all preview components
3. ✅ Configured proper code splitting
4. ✅ Main bundle reduced by 50%

**Remaining large chunks are**:
- Necessary for features (Mermaid, DOT, math rendering)
- Already lazy-loaded (don't affect initial page load)
- Part of third-party libraries (can't be removed without breaking functionality)

**One small cleanup possible**:
- Remove `gql-tag` package (unused, ~5-10 kB)
