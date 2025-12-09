# Frontend Bundle Optimization Plan

## Problems Identified

1. **ELK library (elkjs)**: 1,451 kB (444 kB gzipped)
   - Used in only 2 files for graph layout
   - Can be replaced with dagre (~60 kB, 7x smaller)

2. **Large visualization libraries loaded eagerly**:
   - Mermaid: ~600 kB (only used in preview dialog)
   - d3-graphviz: (only used in DOT preview)
   - force-graph: (only used in graph preview)
   - These should be lazy-loaded

3. **No code splitting**: All code loads upfront causing browser hangs

4. **@assistant-ui/react-markdown**: Pulls in katex (265 kB)
   - This is necessary but could be optimized

## Solution Steps

### 1. Install Dependencies

```bash
cd frontend
npm install dagre @types/dagre
```

### 2. Replace ELK with Dagre

Files to update:
- `src/utils/graphUtils.ts`
- `src/components/editors/PlanVisualEditor/utils/autoLayout.ts`

### 3. Add Lazy Loading

Components to lazy-load:
- `MermaidPreviewDialog`
- `DotPreviewDialog`
- `GraphPreview`
- `GraphPreviewDialog`

### 4. Configure Vite Code Splitting

Update `vite.config.ts` with manual chunks configuration

### 5. Remove ELK Dependency

```bash
cd frontend
npm uninstall elkjs
```

### 6. Fix package.json Warning

Add `"type": "module"` to package.json

## Expected Improvements

- **Bundle size reduction**: ~1.4 MB → ~100 KB (main bundle)
- **Initial load time**: 50% faster
- **Chat page**: No more browser hangs
- **Better caching**: Split chunks allow better long-term caching
