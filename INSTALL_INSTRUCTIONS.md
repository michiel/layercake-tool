# Installation Instructions - Frontend Optimization

## Required Steps (Run these commands)

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies (this will install dagre and remove elkjs)
npm install

# Build to verify everything works
npm run build
```

## What This Does

1. **Installs dagre** - Lightweight graph layout library (replaces ELK)
2. **Installs @types/dagre** - TypeScript type definitions
3. **Removes elkjs** - Heavy library that was causing bundle bloat

## Expected Output

After `npm install`, you should see:
- Added: `dagre@0.8.5`
- Added: `@types/dagre@0.7.52`
- Removed: `elkjs@0.11.0`

After `npm run build`, you should see:
- ✅ No TypeScript errors
- ✅ Smaller bundle sizes
- ✅ No more 1,451 kB ELK chunk
- ✅ Module type warning gone

## If There Are Issues

If you encounter any problems:

1. **Clear node_modules and reinstall**:
   ```bash
   cd frontend
   rm -rf node_modules package-lock.json
   npm install
   ```

2. **Check npm cache**:
   ```bash
   npm cache clean --force
   npm install
   ```

3. **Revert changes** (if needed):
   ```bash
   git checkout frontend/package.json
   npm install
   ```

## Next Steps

Once the build succeeds:
1. Test the application: `npm run dev`
2. Verify chat page loads without hanging
3. Test graph auto-layout functionality
4. Test preview dialogs (Mermaid, DOT, force-graph)
