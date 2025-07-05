import React, { useState } from 'react';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { 
  X, 
  Edit3, 
  Trash2, 
  Copy, 
  ExternalLink, 
  ChevronDown, 
  ChevronRight,
  Eye,
  EyeOff
} from 'lucide-react';

export interface NodeData {
  id: string;
  label: string;
  layer: string;
  isPartition: boolean;
  belongsTo?: string;
  weight: number;
  comment?: string;
  x: number;
  y: number;
  degree: number;
  inDegree: number;
  outDegree: number;
  [key: string]: any; // Allow additional properties
}

export interface EdgeData {
  id: string;
  source: string;
  target: string;
  label: string;
  layer: string;
  weight: number;
  comment?: string;
  [key: string]: any; // Allow additional properties
}

export interface LayerData {
  id: string;
  label: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
  nodeCount: number;
  edgeCount: number;
  visible: boolean;
}

export interface GraphInspectorProps {
  // Selection data
  selectedNodes: NodeData[];
  selectedEdges: EdgeData[];
  selectedLayers: LayerData[];
  
  // Actions
  onNodeUpdate: (nodeId: string, updates: Partial<NodeData>) => void;
  onEdgeUpdate: (edgeId: string, updates: Partial<EdgeData>) => void;
  onLayerUpdate: (layerId: string, updates: Partial<LayerData>) => void;
  onNodeDelete: (nodeId: string) => void;
  onEdgeDelete: (edgeId: string) => void;
  onLayerToggle: (layerId: string) => void;
  onFocusElement: (type: 'node' | 'edge', id: string) => void;
  onClearSelection: () => void;

  // Visibility
  isVisible: boolean;
  onClose: () => void;
  position?: 'left' | 'right';
}

export const GraphInspector: React.FC<GraphInspectorProps> = ({
  selectedNodes,
  selectedEdges,
  selectedLayers,
  onNodeUpdate,
  onEdgeUpdate,
  onLayerUpdate,
  onNodeDelete,
  onEdgeDelete,
  onLayerToggle,
  onFocusElement,
  onClearSelection,
  isVisible,
  onClose,
  position = 'right',
}) => {
  const [activeTab, setActiveTab] = useState<'nodes' | 'edges' | 'layers'>('nodes');
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['properties']));
  const [editingNode, setEditingNode] = useState<string | null>(null);
  const [editingEdge, setEditingEdge] = useState<string | null>(null);

  if (!isVisible) return null;

  const hasSelection = selectedNodes.length > 0 || selectedEdges.length > 0;

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const renderNodeInspector = (node: NodeData) => (
    <div key={node.id} className="border-b border-gray-200 pb-4 mb-4 last:border-b-0">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <h4 className="font-medium text-gray-900">{node.label}</h4>
          <span className="text-xs text-gray-500">#{node.id}</span>
        </div>
        <div className="flex items-center space-x-1">
          <Button
            variant="secondary"
            size="small"
            onClick={() => onFocusElement('node', node.id)}
            className="h-6 w-6 p-0"
            title="Focus node"
          >
            <ExternalLink className="h-3 w-3" />
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={() => setEditingNode(editingNode === node.id ? null : node.id)}
            className="h-6 w-6 p-0"
            title="Edit node"
          >
            <Edit3 className="h-3 w-3" />
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={() => onNodeDelete(node.id)}
            className="h-6 w-6 p-0 text-red-600 hover:text-red-700"
            title="Delete node"
          >
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      </div>

      {/* Properties */}
      <div className="space-y-2">
        {editingNode === node.id ? (
          <NodeEditForm
            node={node}
            onSave={(updates) => {
              onNodeUpdate(node.id, updates);
              setEditingNode(null);
            }}
            onCancel={() => setEditingNode(null)}
          />
        ) : (
          <NodePropertiesView node={node} />
        )}
      </div>
    </div>
  );

  const renderEdgeInspector = (edge: EdgeData) => (
    <div key={edge.id} className="border-b border-gray-200 pb-4 mb-4 last:border-b-0">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <h4 className="font-medium text-gray-900">
            {edge.label || `${edge.source} → ${edge.target}`}
          </h4>
          <span className="text-xs text-gray-500">#{edge.id}</span>
        </div>
        <div className="flex items-center space-x-1">
          <Button
            variant="secondary"
            size="small"
            onClick={() => onFocusElement('edge', edge.id)}
            className="h-6 w-6 p-0"
            title="Focus edge"
          >
            <ExternalLink className="h-3 w-3" />
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={() => setEditingEdge(editingEdge === edge.id ? null : edge.id)}
            className="h-6 w-6 p-0"
            title="Edit edge"
          >
            <Edit3 className="h-3 w-3" />
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={() => onEdgeDelete(edge.id)}
            className="h-6 w-6 p-0 text-red-600 hover:text-red-700"
            title="Delete edge"
          >
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      </div>

      {/* Properties */}
      <div className="space-y-2">
        {editingEdge === edge.id ? (
          <EdgeEditForm
            edge={edge}
            onSave={(updates) => {
              onEdgeUpdate(edge.id, updates);
              setEditingEdge(null);
            }}
            onCancel={() => setEditingEdge(null)}
          />
        ) : (
          <EdgePropertiesView edge={edge} />
        )}
      </div>
    </div>
  );

  const renderLayerInspector = (layer: LayerData) => (
    <div key={layer.id} className="border-b border-gray-200 pb-4 mb-4 last:border-b-0">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <div
            className="w-4 h-4 rounded border"
            style={{ backgroundColor: layer.backgroundColor, borderColor: layer.borderColor }}
          />
          <h4 className="font-medium text-gray-900">{layer.label}</h4>
          <span className="text-xs text-gray-500">#{layer.id}</span>
        </div>
        <div className="flex items-center space-x-1">
          <Button
            variant="secondary"
            size="small"
            onClick={() => onLayerToggle(layer.id)}
            className="h-6 w-6 p-0"
            title={layer.visible ? "Hide layer" : "Show layer"}
          >
            {layer.visible ? <Eye className="h-3 w-3" /> : <EyeOff className="h-3 w-3" />}
          </Button>
        </div>
      </div>

      {/* Layer stats */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-gray-50 p-2 rounded">
          <div className="font-medium text-gray-700">Nodes</div>
          <div className="text-gray-900">{layer.nodeCount}</div>
        </div>
        <div className="bg-gray-50 p-2 rounded">
          <div className="font-medium text-gray-700">Edges</div>
          <div className="text-gray-900">{layer.edgeCount}</div>
        </div>
      </div>
    </div>
  );

  const positionClasses = {
    left: 'left-0',
    right: 'right-0',
  };

  return (
    <div 
      className={`fixed top-16 ${positionClasses[position]} h-[calc(100vh-4rem)] w-80 bg-white border-l border-gray-200 shadow-lg z-20 overflow-hidden flex flex-col`}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <h3 className="text-lg font-medium text-gray-900">Inspector</h3>
        <Button
          variant="secondary"
          size="small"
          onClick={onClose}
          className="h-6 w-6 p-0"
        >
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        <button
          className={`flex-1 px-4 py-2 text-sm font-medium border-b-2 ${
            activeTab === 'nodes'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700'
          }`}
          onClick={() => setActiveTab('nodes')}
        >
          Nodes ({selectedNodes.length})
        </button>
        <button
          className={`flex-1 px-4 py-2 text-sm font-medium border-b-2 ${
            activeTab === 'edges'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700'
          }`}
          onClick={() => setActiveTab('edges')}
        >
          Edges ({selectedEdges.length})
        </button>
        <button
          className={`flex-1 px-4 py-2 text-sm font-medium border-b-2 ${
            activeTab === 'layers'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700'
          }`}
          onClick={() => setActiveTab('layers')}
        >
          Layers ({selectedLayers.length})
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {!hasSelection && activeTab !== 'layers' ? (
          <div className="p-4 text-center text-gray-500">
            <p>No {activeTab} selected</p>
            <p className="text-sm mt-1">
              Click on {activeTab} in the graph to inspect them
            </p>
          </div>
        ) : (
          <div className="p-4">
            {activeTab === 'nodes' && selectedNodes.map(renderNodeInspector)}
            {activeTab === 'edges' && selectedEdges.map(renderEdgeInspector)}
            {activeTab === 'layers' && selectedLayers.map(renderLayerInspector)}
          </div>
        )}
      </div>

      {/* Actions */}
      {hasSelection && (
        <div className="p-4 border-t border-gray-200">
          <Button
            variant="secondary"
            size="small"
            onClick={onClearSelection}
            className="w-full"
          >
            Clear Selection
          </Button>
        </div>
      )}
    </div>
  );
};

// Helper components for editing forms
const NodeEditForm: React.FC<{
  node: NodeData;
  onSave: (updates: Partial<NodeData>) => void;
  onCancel: () => void;
}> = ({ node, onSave, onCancel }) => {
  const [formData, setFormData] = useState({
    label: node.label,
    layer: node.layer,
    weight: node.weight,
    comment: node.comment || '',
  });

  const handleSave = () => {
    onSave({
      label: formData.label,
      layer: formData.layer,
      weight: formData.weight,
      comment: formData.comment || undefined,
    });
  };

  return (
    <div className="space-y-3">
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Label</label>
        <Input
          value={formData.label}
          onChange={(e) => setFormData({ ...formData, label: e.target.value })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Layer</label>
        <Input
          value={formData.layer}
          onChange={(e) => setFormData({ ...formData, layer: e.target.value })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Weight</label>
        <Input
          type="number"
          value={formData.weight}
          onChange={(e) => setFormData({ ...formData, weight: Number(e.target.value) })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Comment</label>
        <textarea
          value={formData.comment}
          onChange={(e) => setFormData({ ...formData, comment: e.target.value })}
          className="w-full text-sm border border-gray-300 rounded-md p-2 focus:ring-blue-500 focus:border-blue-500"
          rows={2}
        />
      </div>
      
      <div className="flex space-x-2">
        <Button variant="primary" size="small" onClick={handleSave}>Save</Button>
        <Button variant="secondary" size="small" onClick={onCancel}>Cancel</Button>
      </div>
    </div>
  );
};

const EdgeEditForm: React.FC<{
  edge: EdgeData;
  onSave: (updates: Partial<EdgeData>) => void;
  onCancel: () => void;
}> = ({ edge, onSave, onCancel }) => {
  const [formData, setFormData] = useState({
    label: edge.label,
    layer: edge.layer,
    weight: edge.weight,
    comment: edge.comment || '',
  });

  const handleSave = () => {
    onSave({
      label: formData.label,
      layer: formData.layer,
      weight: formData.weight,
      comment: formData.comment || undefined,
    });
  };

  return (
    <div className="space-y-3">
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Label</label>
        <Input
          value={formData.label}
          onChange={(e) => setFormData({ ...formData, label: e.target.value })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Layer</label>
        <Input
          value={formData.layer}
          onChange={(e) => setFormData({ ...formData, layer: e.target.value })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Weight</label>
        <Input
          type="number"
          value={formData.weight}
          onChange={(e) => setFormData({ ...formData, weight: Number(e.target.value) })}
          className="w-full text-sm"
        />
      </div>
      
      <div>
        <label className="block text-xs font-medium text-gray-700 mb-1">Comment</label>
        <textarea
          value={formData.comment}
          onChange={(e) => setFormData({ ...formData, comment: e.target.value })}
          className="w-full text-sm border border-gray-300 rounded-md p-2 focus:ring-blue-500 focus:border-blue-500"
          rows={2}
        />
      </div>
      
      <div className="flex space-x-2">
        <Button variant="primary" size="small" onClick={handleSave}>Save</Button>
        <Button variant="secondary" size="small" onClick={onCancel}>Cancel</Button>
      </div>
    </div>
  );
};

const NodePropertiesView: React.FC<{ node: NodeData }> = ({ node }) => (
  <div className="space-y-2 text-xs">
    <div className="grid grid-cols-2 gap-2">
      <div>
        <span className="font-medium text-gray-700">Layer:</span>
        <div className="text-gray-900">{node.layer}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Weight:</span>
        <div className="text-gray-900">{node.weight}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Degree:</span>
        <div className="text-gray-900">{node.degree}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Position:</span>
        <div className="text-gray-900">{Math.round(node.x)}, {Math.round(node.y)}</div>
      </div>
    </div>
    
    {node.comment && (
      <div>
        <span className="font-medium text-gray-700">Comment:</span>
        <div className="text-gray-900 text-wrap">{node.comment}</div>
      </div>
    )}
    
    {node.isPartition && (
      <div className="bg-blue-50 p-2 rounded text-blue-800">
        <span className="font-medium">Partition Node</span>
      </div>
    )}
  </div>
);

const EdgePropertiesView: React.FC<{ edge: EdgeData }> = ({ edge }) => (
  <div className="space-y-2 text-xs">
    <div className="grid grid-cols-2 gap-2">
      <div>
        <span className="font-medium text-gray-700">Source:</span>
        <div className="text-gray-900">{edge.source}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Target:</span>
        <div className="text-gray-900">{edge.target}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Layer:</span>
        <div className="text-gray-900">{edge.layer}</div>
      </div>
      <div>
        <span className="font-medium text-gray-700">Weight:</span>
        <div className="text-gray-900">{edge.weight}</div>
      </div>
    </div>
    
    {edge.comment && (
      <div>
        <span className="font-medium text-gray-700">Comment:</span>
        <div className="text-gray-900 text-wrap">{edge.comment}</div>
      </div>
    )}
  </div>
);

export default GraphInspector;