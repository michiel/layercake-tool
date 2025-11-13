import { lazy, Suspense, ComponentType } from 'react';
import { Spinner } from '../ui/spinner';

// Lazy load heavy visualization components to reduce initial bundle size
// Use absolute imports with @ alias to ensure consistent module resolution across platforms
const MermaidPreviewDialogLazy = lazy(() =>
  import('@/components/visualization/MermaidPreviewDialog').then((module) => ({
    default: module.MermaidPreviewDialog,
  }))
);

const DotPreviewDialogLazy = lazy(() =>
  import('@/components/visualization/DotPreviewDialog').then((module) => ({
    default: module.DotPreviewDialog,
  }))
);

const GraphPreviewDialogLazy = lazy(() =>
  import('@/components/visualization/GraphPreviewDialog').then((module) => ({
    default: module.GraphPreviewDialog,
  }))
);

const DataPreviewDialogLazy = lazy(() =>
  import('@/components/visualization/DataPreviewDialog').then((module) => ({
    default: module.DataPreviewDialog,
  }))
);

// Loading fallback component
const LoadingFallback = () => (
  <div className="flex items-center justify-center p-8">
    <Spinner size="lg" />
    <span className="ml-3 text-sm text-muted-foreground">Loading preview...</span>
  </div>
);

// Wrapper to provide Suspense boundary
function withSuspense<P extends object>(Component: ComponentType<P>) {
  return (props: P) => (
    <Suspense fallback={<LoadingFallback />}>
      <Component {...props} />
    </Suspense>
  );
}

// Export lazy-loaded components with Suspense boundaries
export const MermaidPreviewDialog = withSuspense(MermaidPreviewDialogLazy);
export const DotPreviewDialog = withSuspense(DotPreviewDialogLazy);
export const GraphPreviewDialog = withSuspense(GraphPreviewDialogLazy);
export const DataPreviewDialog = withSuspense(DataPreviewDialogLazy);

// Re-export other visualization components that don't need lazy loading
export { DataPreview } from './DataPreview';
export { GraphPreview } from './GraphPreview';
