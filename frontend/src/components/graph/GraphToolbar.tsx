import React, { useState } from 'react';
import { Button } from '../ui/Button';
import { 
  Settings, 
  ZoomIn, 
  ZoomOut, 
  RotateCcw, 
  Download, 
  Upload, 
  Maximize2, 
  Minimize2,
  Play,
  Pause,
  Square,
  Filter,
  Eye,
  EyeOff,
  Grid,
  Search,
  Info
} from 'lucide-react';

export interface GraphToolbarProps {
  // Zoom controls
  zoomLevel: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetZoom: () => void;
  onFitToScreen: () => void;

  // Layout controls
  isLayoutRunning: boolean;
  onStartLayout: () => void;
  onStopLayout: () => void;
  onResetLayout: () => void;

  // View controls
  isFullscreen: boolean;
  onToggleFullscreen: () => void;
  showGrid: boolean;
  onToggleGrid: () => void;
  showMinimap: boolean;
  onToggleMinimap: () => void;

  // Selection and search
  selectedNodeCount: number;
  selectedEdgeCount: number;
  onClearSelection: () => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  searchResults: number;

  // Settings and tools
  onOpenSettings: () => void;
  onOpenFilters: () => void;
  onExportGraph: () => void;
  onImportGraph: () => void;
  onShowInfo: () => void;

  // Statistics
  nodeCount: number;
  edgeCount: number;
  layerCount: number;
  
  // Layout algorithm
  currentLayout: string;
  availableLayouts: string[];
  onLayoutChange: (layout: string) => void;
}

export const GraphToolbar: React.FC<GraphToolbarProps> = ({
  zoomLevel,
  onZoomIn,
  onZoomOut,
  onResetZoom,
  onFitToScreen,
  isLayoutRunning,
  onStartLayout,
  onStopLayout,
  onResetLayout,
  isFullscreen,
  onToggleFullscreen,
  showGrid,
  onToggleGrid,
  showMinimap,
  onToggleMinimap,
  selectedNodeCount,
  selectedEdgeCount,
  onClearSelection,
  searchQuery,
  onSearchChange,
  searchResults,
  onOpenSettings,
  onOpenFilters,
  onExportGraph,
  onImportGraph,
  onShowInfo,
  nodeCount,
  edgeCount,
  layerCount,
  currentLayout,
  availableLayouts,
  onLayoutChange,
}) => {
  const [showStats, setShowStats] = useState(false);

  return (
    <div className="bg-white border-b border-gray-200 px-4 py-2">
      <div className="flex items-center justify-between">
        {/* Left section - Main controls */}
        <div className="flex items-center space-x-1">
          {/* Layout controls */}
          <div className="flex items-center space-x-1 border-r border-gray-200 pr-3 mr-3">
            <select
              value={currentLayout}
              onChange={(e) => onLayoutChange(e.target.value)}
              className="text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              {availableLayouts.map((layout) => (
                <option key={layout} value={layout}>
                  {layout}
                </option>
              ))}
            </select>
            
            {isLayoutRunning ? (
              <>
                <Button
                  variant="secondary"
                  size="small"
                  onClick={onStopLayout}
                  className="h-8 w-8 p-0"
                  title="Stop layout"
                >
                  <Pause className="h-4 w-4" />
                </Button>
                <Button
                  variant="secondary"
                  size="small"
                  onClick={onResetLayout}
                  className="h-8 w-8 p-0"
                  title="Reset layout"
                >
                  <Square className="h-4 w-4" />
                </Button>
              </>
            ) : (
              <Button
                variant="secondary"
                size="small"
                onClick={onStartLayout}
                className="h-8 w-8 p-0"
                title="Start layout"
              >
                <Play className="h-4 w-4" />
              </Button>
            )}
          </div>

          {/* Zoom controls */}
          <div className="flex items-center space-x-1 border-r border-gray-200 pr-3 mr-3">
            <Button
              variant="secondary"
              size="small"
              onClick={onZoomOut}
              className="h-8 w-8 p-0"
              title="Zoom out"
            >
              <ZoomOut className="h-4 w-4" />
            </Button>
            
            <span className="text-xs text-gray-600 min-w-[3rem] text-center">
              {Math.round(zoomLevel * 100)}%
            </span>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onZoomIn}
              className="h-8 w-8 p-0"
              title="Zoom in"
            >
              <ZoomIn className="h-4 w-4" />
            </Button>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onResetZoom}
              className="h-8 w-8 p-0"
              title="Reset zoom"
            >
              <RotateCcw className="h-4 w-4" />
            </Button>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onFitToScreen}
              className="h-8 w-8 p-0"
              title="Fit to screen"
            >
              <Maximize2 className="h-4 w-4" />
            </Button>
          </div>

          {/* View controls */}
          <div className="flex items-center space-x-1 border-r border-gray-200 pr-3 mr-3">
            <Button
              variant={showGrid ? "primary" : "secondary"}
              size="small"
              onClick={onToggleGrid}
              className="h-8 w-8 p-0"
              title={showGrid ? "Hide grid" : "Show grid"}
            >
              <Grid className="h-4 w-4" />
            </Button>
            
            <Button
              variant={showMinimap ? "primary" : "secondary"}
              size="small"
              onClick={onToggleMinimap}
              className="h-8 w-8 p-0"
              title={showMinimap ? "Hide minimap" : "Show minimap"}
            >
              {showMinimap ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onToggleFullscreen}
              className="h-8 w-8 p-0"
              title={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
            >
              {isFullscreen ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
            </Button>
          </div>

          {/* Tools */}
          <div className="flex items-center space-x-1">
            <Button
              variant="secondary"
              size="small"
              onClick={onOpenFilters}
              className="h-8 w-8 p-0"
              title="Open filters"
            >
              <Filter className="h-4 w-4" />
            </Button>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onOpenSettings}
              className="h-8 w-8 p-0"
              title="Open settings"
            >
              <Settings className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* Center section - Search */}
        <div className="flex items-center space-x-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search nodes and edges..."
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              className="pl-10 pr-4 py-1.5 text-sm border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 w-64"
            />
            {searchQuery && (
              <span className="absolute right-3 top-1/2 transform -translate-y-1/2 text-xs text-gray-500">
                {searchResults} results
              </span>
            )}
          </div>
        </div>

        {/* Right section - Actions and stats */}
        <div className="flex items-center space-x-1">
          {/* Selection info */}
          {(selectedNodeCount > 0 || selectedEdgeCount > 0) && (
            <div className="flex items-center space-x-2 border-r border-gray-200 pr-3 mr-3">
              <span className="text-sm text-gray-600">
                Selected: {selectedNodeCount} nodes, {selectedEdgeCount} edges
              </span>
              <Button
                variant="secondary"
                size="small"
                onClick={onClearSelection}
                className="text-xs"
              >
                Clear
              </Button>
            </div>
          )}

          {/* Stats */}
          <div className="flex items-center space-x-1 border-r border-gray-200 pr-3 mr-3">
            <Button
              variant={showStats ? "primary" : "secondary"}
              size="small"
              onClick={() => setShowStats(!showStats)}
              className="h-8 w-8 p-0"
              title="Toggle statistics"
            >
              <Info className="h-4 w-4" />
            </Button>
            
            {showStats && (
              <div className="text-xs text-gray-600 space-x-2">
                <span>N: {nodeCount}</span>
                <span>E: {edgeCount}</span>
                <span>L: {layerCount}</span>
              </div>
            )}
          </div>

          {/* Import/Export */}
          <div className="flex items-center space-x-1">
            <Button
              variant="secondary"
              size="small"
              onClick={onImportGraph}
              className="h-8 w-8 p-0"
              title="Import graph"
            >
              <Upload className="h-4 w-4" />
            </Button>
            
            <Button
              variant="secondary"
              size="small"
              onClick={onExportGraph}
              className="h-8 w-8 p-0"
              title="Export graph"
            >
              <Download className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* Extended toolbar for additional controls when needed */}
      {showStats && (
        <div className="mt-2 pt-2 border-t border-gray-100">
          <div className="flex items-center justify-between text-xs text-gray-600">
            <div className="flex items-center space-x-4">
              <span>Nodes: {nodeCount}</span>
              <span>Edges: {edgeCount}</span>
              <span>Layers: {layerCount}</span>
              <span>Zoom: {Math.round(zoomLevel * 100)}%</span>
            </div>
            
            <div className="flex items-center space-x-4">
              <span>Layout: {currentLayout}</span>
              {isLayoutRunning && (
                <span className="text-blue-600 animate-pulse">Layout running...</span>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};