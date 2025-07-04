import { useState } from 'react';
import { ZoomIn, ZoomOut, RotateCcw, Play, Pause, Settings } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/utils';

interface GraphControlsProps {
  onZoomIn: () => void;
  onZoomOut: () => void;
  onReset: () => void;
  onToggleSimulation: () => void;
  isSimulationRunning: boolean;
  onOpenSettings?: () => void;
  className?: string;
}

export function GraphControls({
  onZoomIn,
  onZoomOut,
  onReset,
  onToggleSimulation,
  isSimulationRunning,
  onOpenSettings,
  className,
}: GraphControlsProps) {
  return (
    <div className={cn('flex flex-col gap-2 p-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700', className)}>
      {/* Zoom Controls */}
      <div className="flex flex-col gap-1">
        <span className="text-xs font-medium text-gray-600 dark:text-gray-400">Zoom</span>
        <Button
          variant="ghost"
          size="sm"
          onClick={onZoomIn}
          className="w-8 h-8 p-0"
          title="Zoom In"
        >
          <ZoomIn className="w-4 h-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onZoomOut}
          className="w-8 h-8 p-0"
          title="Zoom Out"
        >
          <ZoomOut className="w-4 h-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onReset}
          className="w-8 h-8 p-0"
          title="Reset View"
        >
          <RotateCcw className="w-4 h-4" />
        </Button>
      </div>

      {/* Simulation Controls */}
      <div className="flex flex-col gap-1 border-t border-gray-200 dark:border-gray-700 pt-2">
        <span className="text-xs font-medium text-gray-600 dark:text-gray-400">Simulation</span>
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggleSimulation}
          className="w-8 h-8 p-0"
          title={isSimulationRunning ? "Pause Simulation" : "Start Simulation"}
        >
          {isSimulationRunning ? (
            <Pause className="w-4 h-4" />
          ) : (
            <Play className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* Settings */}
      {onOpenSettings && (
        <div className="flex flex-col gap-1 border-t border-gray-200 dark:border-gray-700 pt-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onOpenSettings}
            className="w-8 h-8 p-0"
            title="Graph Settings"
          >
            <Settings className="w-4 h-4" />
          </Button>
        </div>
      )}
    </div>
  );
}

interface GraphStatsProps {
  nodeCount: number;
  edgeCount: number;
  layerCount: number;
  selectedNode?: string | null;
  className?: string;
}

export function GraphStats({
  nodeCount,
  edgeCount,
  layerCount,
  selectedNode,
  className,
}: GraphStatsProps) {
  return (
    <div className={cn('p-3 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700', className)}>
      <div className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
        Graph Statistics
      </div>
      <div className="space-y-1 text-xs text-gray-600 dark:text-gray-400">
        <div className="flex justify-between">
          <span>Nodes:</span>
          <span className="font-medium">{nodeCount}</span>
        </div>
        <div className="flex justify-between">
          <span>Edges:</span>
          <span className="font-medium">{edgeCount}</span>
        </div>
        <div className="flex justify-between">
          <span>Layers:</span>
          <span className="font-medium">{layerCount}</span>
        </div>
        {selectedNode && (
          <div className="pt-1 border-t border-gray-200 dark:border-gray-700">
            <div className="text-xs font-medium text-primary-600 dark:text-primary-400">
              Selected: {selectedNode}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface GraphToolbarProps {
  onExport?: () => void;
  onImport?: () => void;
  onLayout?: (layout: string) => void;
  className?: string;
}

export function GraphToolbar({
  onExport,
  onImport,
  onLayout,
  className,
}: GraphToolbarProps) {
  const [selectedLayout, setSelectedLayout] = useState('force');

  const layouts = [
    { id: 'force', name: 'Force Directed' },
    { id: 'circular', name: 'Circular' },
    { id: 'hierarchical', name: 'Hierarchical' },
    { id: 'grid', name: 'Grid' },
  ];

  const handleLayoutChange = (layout: string) => {
    setSelectedLayout(layout);
    onLayout?.(layout);
  };

  return (
    <div className={cn('flex items-center gap-2 p-2 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700', className)}>
      {/* Layout Selection */}
      <div className="flex items-center gap-2">
        <span className="text-xs font-medium text-gray-600 dark:text-gray-400">Layout:</span>
        <select
          value={selectedLayout}
          onChange={(e) => handleLayoutChange(e.target.value)}
          className="text-xs px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300"
        >
          {layouts.map(layout => (
            <option key={layout.id} value={layout.id}>
              {layout.name}
            </option>
          ))}
        </select>
      </div>

      {/* Import/Export */}
      <div className="flex items-center gap-1 ml-auto">
        {onImport && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onImport}
            className="text-xs"
          >
            Import
          </Button>
        )}
        {onExport && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onExport}
            className="text-xs"
          >
            Export
          </Button>
        )}
      </div>
    </div>
  );
}