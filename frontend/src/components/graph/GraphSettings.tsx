import React, { useState, useEffect } from 'react';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';

export interface GraphLayoutSettings {
  // Force-directed layout settings
  forceStrength: number;
  linkDistance: number;
  linkStrength: number;
  chargeStrength: number;
  centerStrength: number;
  
  // Hierarchy layout settings
  nodeSize: number;
  levelSeparation: number;
  nodeSeparation: number;
  
  // Visual settings
  enableLabels: boolean;
  labelFontSize: number;
  nodeOpacity: number;
  edgeOpacity: number;
  enableAnimations: boolean;
  animationDuration: number;
  
  // Collision detection
  enableCollision: boolean;
  collisionRadius: number;
}

export interface GraphDisplaySettings {
  // Zoom and pan
  enableZoom: boolean;
  enablePan: boolean;
  zoomExtent: [number, number];
  
  // Node rendering
  showNodeLabels: boolean;
  showNodeIcons: boolean;
  nodeScale: number;
  nodeColorScheme: 'layer' | 'weight' | 'custom';
  
  // Edge rendering
  showEdgeLabels: boolean;
  showEdgeWeights: boolean;
  edgeScale: number;
  edgeColorScheme: 'layer' | 'weight' | 'custom';
  
  // Layer visualization
  showLayers: boolean;
  layerOpacity: number;
  groupByLayers: boolean;
  
  // Performance settings
  enableWebGL: boolean;
  maxVisibleNodes: number;
  levelOfDetail: boolean;
}

export interface GraphFilterSettings {
  // Node filters
  minNodeWeight: number;
  maxNodeWeight: number;
  selectedLayers: string[];
  nodeTypes: string[];
  
  // Edge filters
  minEdgeWeight: number;
  maxEdgeWeight: number;
  showSelfLoops: boolean;
  showMultiEdges: boolean;
  
  // Layout filters
  showIsolatedNodes: boolean;
  minDegree: number;
  maxDegree: number;
}

export interface GraphSettingsProps {
  layoutSettings: GraphLayoutSettings;
  displaySettings: GraphDisplaySettings;
  filterSettings: GraphFilterSettings;
  onLayoutChange: (settings: GraphLayoutSettings) => void;
  onDisplayChange: (settings: GraphDisplaySettings) => void;
  onFilterChange: (settings: GraphFilterSettings) => void;
  onReset: () => void;
  onExport: () => void;
  availableLayers: string[];
  isVisible: boolean;
  onClose: () => void;
}

export const GraphSettings: React.FC<GraphSettingsProps> = ({
  layoutSettings,
  displaySettings,
  filterSettings,
  onLayoutChange,
  onDisplayChange,
  onFilterChange,
  onReset,
  onExport,
  availableLayers,
  isVisible,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<'layout' | 'display' | 'filters'>('layout');
  const [localLayoutSettings, setLocalLayoutSettings] = useState(layoutSettings);
  const [localDisplaySettings, setLocalDisplaySettings] = useState(displaySettings);
  const [localFilterSettings, setLocalFilterSettings] = useState(filterSettings);

  useEffect(() => {
    setLocalLayoutSettings(layoutSettings);
    setLocalDisplaySettings(displaySettings);
    setLocalFilterSettings(filterSettings);
  }, [layoutSettings, displaySettings, filterSettings]);

  const handleApply = () => {
    onLayoutChange(localLayoutSettings);
    onDisplayChange(localDisplaySettings);
    onFilterChange(localFilterSettings);
    onClose();
  };

  const handleReset = () => {
    onReset();
    onClose();
  };

  const renderLayoutSettings = () => (
    <div className="space-y-6">
      {/* Force-directed layout */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Force-Directed Layout</h4>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Force Strength
            </label>
            <Input
              type="range"
              min="-100"
              max="100"
              value={localLayoutSettings.forceStrength}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                forceStrength: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.forceStrength}</span>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Link Distance
            </label>
            <Input
              type="range"
              min="10"
              max="200"
              value={localLayoutSettings.linkDistance}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                linkDistance: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.linkDistance}px</span>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Link Strength
            </label>
            <Input
              type="range"
              min="0"
              max="2"
              step="0.1"
              value={localLayoutSettings.linkStrength}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                linkStrength: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.linkStrength}</span>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Charge Strength
            </label>
            <Input
              type="range"
              min="-500"
              max="0"
              value={localLayoutSettings.chargeStrength}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                chargeStrength: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.chargeStrength}</span>
          </div>
        </div>
      </div>

      {/* Hierarchy layout */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Hierarchy Layout</h4>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Node Size
            </label>
            <Input
              type="range"
              min="5"
              max="50"
              value={localLayoutSettings.nodeSize}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                nodeSize: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.nodeSize}px</span>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Level Separation
            </label>
            <Input
              type="range"
              min="50"
              max="300"
              value={localLayoutSettings.levelSeparation}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                levelSeparation: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localLayoutSettings.levelSeparation}px</span>
          </div>
        </div>
      </div>

      {/* Animation settings */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Animation</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localLayoutSettings.enableAnimations}
              onChange={(e) => setLocalLayoutSettings({
                ...localLayoutSettings,
                enableAnimations: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Enable animations</span>
          </label>
          
          {localLayoutSettings.enableAnimations && (
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Animation Duration
              </label>
              <Input
                type="range"
                min="100"
                max="2000"
                step="100"
                value={localLayoutSettings.animationDuration}
                onChange={(e) => setLocalLayoutSettings({
                  ...localLayoutSettings,
                  animationDuration: Number(e.target.value)
                })}
                className="w-full"
              />
              <span className="text-xs text-gray-500">{localLayoutSettings.animationDuration}ms</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );

  const renderDisplaySettings = () => (
    <div className="space-y-6">
      {/* Node rendering */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Node Rendering</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.showNodeLabels}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                showNodeLabels: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show node labels</span>
          </label>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.showNodeIcons}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                showNodeIcons: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show node icons</span>
          </label>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Node Scale
            </label>
            <Input
              type="range"
              min="0.5"
              max="3"
              step="0.1"
              value={localDisplaySettings.nodeScale}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                nodeScale: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localDisplaySettings.nodeScale}x</span>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Node Color Scheme
            </label>
            <select
              value={localDisplaySettings.nodeColorScheme}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                nodeColorScheme: e.target.value as 'layer' | 'weight' | 'custom'
              })}
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
            >
              <option value="layer">By Layer</option>
              <option value="weight">By Weight</option>
              <option value="custom">Custom</option>
            </select>
          </div>
        </div>
      </div>

      {/* Edge rendering */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Edge Rendering</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.showEdgeLabels}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                showEdgeLabels: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show edge labels</span>
          </label>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.showEdgeWeights}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                showEdgeWeights: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show edge weights</span>
          </label>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Edge Scale
            </label>
            <Input
              type="range"
              min="0.5"
              max="3"
              step="0.1"
              value={localDisplaySettings.edgeScale}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                edgeScale: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localDisplaySettings.edgeScale}x</span>
          </div>
        </div>
      </div>

      {/* Layer visualization */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Layer Visualization</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.showLayers}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                showLayers: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show layer backgrounds</span>
          </label>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.groupByLayers}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                groupByLayers: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Group nodes by layers</span>
          </label>
          
          {localDisplaySettings.showLayers && (
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Layer Opacity
              </label>
              <Input
                type="range"
                min="0.1"
                max="1"
                step="0.1"
                value={localDisplaySettings.layerOpacity}
                onChange={(e) => setLocalDisplaySettings({
                  ...localDisplaySettings,
                  layerOpacity: Number(e.target.value)
                })}
                className="w-full"
              />
              <span className="text-xs text-gray-500">{Math.round(localDisplaySettings.layerOpacity * 100)}%</span>
            </div>
          )}
        </div>
      </div>

      {/* Performance settings */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Performance</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.enableWebGL}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                enableWebGL: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Enable WebGL rendering</span>
          </label>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localDisplaySettings.levelOfDetail}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                levelOfDetail: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Level of detail optimization</span>
          </label>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Max Visible Nodes
            </label>
            <Input
              type="range"
              min="100"
              max="10000"
              step="100"
              value={localDisplaySettings.maxVisibleNodes}
              onChange={(e) => setLocalDisplaySettings({
                ...localDisplaySettings,
                maxVisibleNodes: Number(e.target.value)
              })}
              className="w-full"
            />
            <span className="text-xs text-gray-500">{localDisplaySettings.maxVisibleNodes}</span>
          </div>
        </div>
      </div>
    </div>
  );

  const renderFilterSettings = () => (
    <div className="space-y-6">
      {/* Node filters */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Node Filters</h4>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Min Weight
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.minNodeWeight}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  minNodeWeight: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
            
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Max Weight
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.maxNodeWeight}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  maxNodeWeight: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
          </div>
          
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-2">
              Visible Layers
            </label>
            <div className="space-y-1 max-h-32 overflow-y-auto border rounded-md p-2">
              {availableLayers.map((layer) => (
                <label key={layer} className="flex items-center">
                  <input
                    type="checkbox"
                    checked={localFilterSettings.selectedLayers.includes(layer)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setLocalFilterSettings({
                          ...localFilterSettings,
                          selectedLayers: [...localFilterSettings.selectedLayers, layer]
                        });
                      } else {
                        setLocalFilterSettings({
                          ...localFilterSettings,
                          selectedLayers: localFilterSettings.selectedLayers.filter(l => l !== layer)
                        });
                      }
                    }}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="ml-2 text-sm text-gray-700">{layer}</span>
                </label>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Edge filters */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Edge Filters</h4>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Min Weight
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.minEdgeWeight}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  minEdgeWeight: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
            
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Max Weight
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.maxEdgeWeight}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  maxEdgeWeight: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
          </div>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localFilterSettings.showSelfLoops}
              onChange={(e) => setLocalFilterSettings({
                ...localFilterSettings,
                showSelfLoops: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show self-loops</span>
          </label>
          
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localFilterSettings.showMultiEdges}
              onChange={(e) => setLocalFilterSettings({
                ...localFilterSettings,
                showMultiEdges: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show multiple edges</span>
          </label>
        </div>
      </div>

      {/* Layout filters */}
      <div>
        <h4 className="text-sm font-medium text-gray-900 mb-3">Layout Filters</h4>
        <div className="space-y-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              checked={localFilterSettings.showIsolatedNodes}
              onChange={(e) => setLocalFilterSettings({
                ...localFilterSettings,
                showIsolatedNodes: e.target.checked
              })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="ml-2 text-sm text-gray-700">Show isolated nodes</span>
          </label>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Min Degree
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.minDegree}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  minDegree: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
            
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Max Degree
              </label>
              <Input
                type="number"
                min="0"
                value={localFilterSettings.maxDegree}
                onChange={(e) => setLocalFilterSettings({
                  ...localFilterSettings,
                  maxDegree: Number(e.target.value)
                })}
                className="w-full"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  if (!isVisible) return null;

  return (
    <Modal
      isOpen={isVisible}
      onClose={onClose}
      title="Graph Settings"
      size="large"
    >
      <div className="flex flex-col h-full">
        {/* Tab navigation */}
        <div className="flex border-b border-gray-200 mb-6">
          <button
            className={`px-4 py-2 text-sm font-medium border-b-2 ${
              activeTab === 'layout'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
            onClick={() => setActiveTab('layout')}
          >
            Layout
          </button>
          <button
            className={`px-4 py-2 text-sm font-medium border-b-2 ${
              activeTab === 'display'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
            onClick={() => setActiveTab('display')}
          >
            Display
          </button>
          <button
            className={`px-4 py-2 text-sm font-medium border-b-2 ${
              activeTab === 'filters'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
            onClick={() => setActiveTab('filters')}
          >
            Filters
          </button>
        </div>

        {/* Tab content */}
        <div className="flex-1 overflow-y-auto">
          {activeTab === 'layout' && renderLayoutSettings()}
          {activeTab === 'display' && renderDisplaySettings()}
          {activeTab === 'filters' && renderFilterSettings()}
        </div>

        {/* Actions */}
        <div className="flex justify-between pt-6 border-t border-gray-200">
          <div className="flex space-x-2">
            <Button variant="secondary" onClick={handleReset}>
              Reset to Defaults
            </Button>
            <Button variant="secondary" onClick={onExport}>
              Export Settings
            </Button>
          </div>
          <div className="flex space-x-2">
            <Button variant="secondary" onClick={onClose}>
              Cancel
            </Button>
            <Button variant="primary" onClick={handleApply}>
              Apply Changes
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
};