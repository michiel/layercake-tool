import React, { useState, useCallback, useMemo, useEffect } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { 
  ColumnDef, 
  getCoreRowModel, 
  useReactTable, 
  flexRender,
  getFilteredRowModel,
  getPaginationRowModel,
} from '@tanstack/react-table';
import { Plus, Download, Upload, Save, X, Check, Eye, EyeOff, RefreshCw } from 'lucide-react';
import { GET_GRAPH_ARTIFACT, CREATE_PLAN_NODE } from '../../graphql/dag';
import { GraphArtifact } from '../../types/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';
import { useGraphSync, GraphVisualizationRef } from '../../hooks/useGraphSync';

interface GraphDataGridProps {
  projectId: number;
  planId: number;
  planNodeId: string;
  editMode?: 'transformation' | 'in-place' | 'read-only';
  syncWithVisualization?: boolean;
  onDataChange?: (changes: GraphDataChanges) => void;
  onNodeSelect?: (nodeIds: string[]) => void;
  onEdgeSelect?: (edgeIds: string[]) => void;
  onValidationError?: (errors: ValidationError[]) => void;
  visualizationRef?: GraphVisualizationRef;
}

interface ValidationError {
  type: 'node' | 'edge' | 'layer';
  id: string;
  field: string;
  message: string;
  severity: 'error' | 'warning';
}

interface GraphDataChanges {
  nodes: {
    added: GraphNode[];
    updated: { id: string; changes: Partial<GraphNode> }[];
    deleted: string[];
  };
  edges: {
    added: GraphEdge[];
    updated: { id: string; changes: Partial<GraphEdge> }[];
    deleted: string[];
  };
  layers: {
    added: GraphLayer[];
    updated: { id: string; changes: Partial<GraphLayer> }[];
    deleted: string[];
  };
}

interface GraphNode {
  id: string;
  label: string;
  layer: string;
  x?: number;
  y?: number;
  weight?: number;
  properties?: Record<string, any>;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  weight?: number;
  properties?: Record<string, any>;
}

interface GraphLayer {
  id: string;
  name: string;
  color: string;
  description?: string;
  visible?: boolean;
  order?: number;
}

type TabType = 'nodes' | 'edges' | 'layers';

export const GraphDataGrid: React.FC<GraphDataGridProps> = ({
  projectId,
  planId,
  planNodeId,
  editMode = 'transformation',
  syncWithVisualization = true,
  onDataChange,
  onNodeSelect,
  onEdgeSelect,
  onValidationError,
  visualizationRef,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('nodes');
  const [selectedRows, setSelectedRows] = useState<string[]>([]);

  const handleRowSelection = useCallback((rowIds: string[]) => {
    setSelectedRows(rowIds);
    
    // Sync selection with visualization
    if (activeTab === 'nodes' && onNodeSelect) {
      onNodeSelect(rowIds);
      highlightInVisualization(rowIds, []);
    } else if (activeTab === 'edges' && onEdgeSelect) {
      onEdgeSelect(rowIds);
      highlightInVisualization([], rowIds);
    }
  }, [activeTab, onNodeSelect, onEdgeSelect, highlightInVisualization]);
  const [pendingChanges, setPendingChanges] = useState<GraphDataChanges>({
    nodes: { added: [], updated: [], deleted: [] },
    edges: { added: [], updated: [], deleted: [] },
    layers: { added: [], updated: [], deleted: [] },
  });
  const [isEditing, setIsEditing] = useState(false);
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);

  // Initialize graph sync hook
  const {
    registerVisualization,
    unregisterVisualization,
    syncToVisualization,
    highlightInVisualization,
    focusInVisualization,
    refreshVisualization,
    isVisualizationConnected,
  } = useGraphSync({
    syncWithVisualization,
    onSyncError: (error) => {
      console.error('Graph visualization sync error:', error);
    },
  });

  // Fetch graph data
  const { data, loading, error, refetch } = useQuery(GET_GRAPH_ARTIFACT, {
    variables: { planNodeId },
  });

  const [createTransformationNode] = useMutation(CREATE_PLAN_NODE);

  const graphData = useMemo(() => {
    if (!data?.graph_artifact?.graph_data) return null;
    
    try {
      return JSON.parse(data.graph_artifact.graph_data);
    } catch (error) {
      console.error('Failed to parse graph data:', error);
      return null;
    }
  }, [data]);

  // Validation functions
  const validateNodeData = useCallback((node: GraphNode): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!node.id || node.id.trim() === '') {
      errors.push({
        type: 'node',
        id: node.id,
        field: 'id',
        message: 'Node ID cannot be empty',
        severity: 'error'
      });
    }
    
    if (!node.label || node.label.trim() === '') {
      errors.push({
        type: 'node',
        id: node.id,
        field: 'label',
        message: 'Node label cannot be empty',
        severity: 'error'
      });
    }
    
    if (!node.layer || node.layer.trim() === '') {
      errors.push({
        type: 'node',
        id: node.id,
        field: 'layer',
        message: 'Node must be assigned to a layer',
        severity: 'error'
      });
    }
    
    if (node.weight !== undefined && (node.weight < 0 || node.weight > 1000)) {
      errors.push({
        type: 'node',
        id: node.id,
        field: 'weight',
        message: 'Node weight must be between 0 and 1000',
        severity: 'warning'
      });
    }
    
    return errors;
  }, []);

  const validateEdgeData = useCallback((edge: GraphEdge): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!edge.id || edge.id.trim() === '') {
      errors.push({
        type: 'edge',
        id: edge.id,
        field: 'id',
        message: 'Edge ID cannot be empty',
        severity: 'error'
      });
    }
    
    if (!edge.source || edge.source.trim() === '') {
      errors.push({
        type: 'edge',
        id: edge.id,
        field: 'source',
        message: 'Edge source cannot be empty',
        severity: 'error'
      });
    }
    
    if (!edge.target || edge.target.trim() === '') {
      errors.push({
        type: 'edge',
        id: edge.id,
        field: 'target',
        message: 'Edge target cannot be empty',
        severity: 'error'
      });
    }
    
    if (edge.source === edge.target) {
      errors.push({
        type: 'edge',
        id: edge.id,
        field: 'target',
        message: 'Edge cannot connect a node to itself',
        severity: 'error'
      });
    }
    
    if (edge.weight !== undefined && (edge.weight < 0 || edge.weight > 1000)) {
      errors.push({
        type: 'edge',
        id: edge.id,
        field: 'weight',
        message: 'Edge weight must be between 0 and 1000',
        severity: 'warning'
      });
    }
    
    return errors;
  }, []);

  const validateLayerData = useCallback((layer: GraphLayer): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!layer.id || layer.id.trim() === '') {
      errors.push({
        type: 'layer',
        id: layer.id,
        field: 'id',
        message: 'Layer ID cannot be empty',
        severity: 'error'
      });
    }
    
    if (!layer.name || layer.name.trim() === '') {
      errors.push({
        type: 'layer',
        id: layer.id,
        field: 'name',
        message: 'Layer name cannot be empty',
        severity: 'error'
      });
    }
    
    if (!layer.color || !layer.color.match(/^#[0-9A-F]{6}$/i)) {
      errors.push({
        type: 'layer',
        id: layer.id,
        field: 'color',
        message: 'Layer color must be a valid hex color (e.g., #FF0000)',
        severity: 'error'
      });
    }
    
    return errors;
  }, []);

  const validateAllData = useCallback(() => {
    if (!graphData) return;
    
    const allErrors: ValidationError[] = [];
    
    // Validate nodes
    (graphData.nodes || []).forEach((node: GraphNode) => {
      allErrors.push(...validateNodeData(node));
    });
    
    // Validate edges
    (graphData.edges || []).forEach((edge: GraphEdge) => {
      allErrors.push(...validateEdgeData(edge));
    });
    
    // Validate layers
    (graphData.layers || []).forEach((layer: GraphLayer) => {
      allErrors.push(...validateLayerData(layer));
    });
    
    // Check for orphaned edges (edges pointing to non-existent nodes)
    const nodeIds = new Set((graphData.nodes || []).map((n: GraphNode) => n.id));
    (graphData.edges || []).forEach((edge: GraphEdge) => {
      if (!nodeIds.has(edge.source)) {
        allErrors.push({
          type: 'edge',
          id: edge.id,
          field: 'source',
          message: `Source node '${edge.source}' does not exist`,
          severity: 'error'
        });
      }
      if (!nodeIds.has(edge.target)) {
        allErrors.push({
          type: 'edge',
          id: edge.id,
          field: 'target',
          message: `Target node '${edge.target}' does not exist`,
          severity: 'error'
        });
      }
    });
    
    setValidationErrors(allErrors);
    if (onValidationError) {
      onValidationError(allErrors);
    }
  }, [graphData, validateNodeData, validateEdgeData, validateLayerData, onValidationError]);

  // Register visualization ref
  useEffect(() => {
    if (visualizationRef) {
      registerVisualization(visualizationRef);
    }
    return () => {
      unregisterVisualization();
    };
  }, [visualizationRef, registerVisualization, unregisterVisualization]);

  // Sync data to visualization when graph data changes
  useEffect(() => {
    if (graphData && syncWithVisualization) {
      syncToVisualization({
        nodes: graphData.nodes || [],
        edges: graphData.edges || [],
        layers: graphData.layers || [],
      });
    }
  }, [graphData, syncWithVisualization, syncToVisualization]);

  // Run validation when data changes
  useEffect(() => {
    validateAllData();
  }, [validateAllData]);

  // Node columns configuration
  const nodeColumns = useMemo<ColumnDef<GraphNode>[]>(() => [
    {
      accessorKey: 'id',
      header: 'ID',
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono text-sm">{getValue() as string}</span>
      ),
    },
    {
      accessorKey: 'label',
      header: 'Label',
      enableSorting: true,
      cell: ({ getValue, row, column }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.id, 'label', newValue)}
            errors={getCellErrors(row.original.id, 'label')}
          />
        );
      },
    },
    {
      accessorKey: 'layer',
      header: 'Layer',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.id, 'layer', newValue)}
            errors={getCellErrors(row.original.id, 'layer')}
          />
        );
      },
    },
    {
      accessorKey: 'x',
      header: 'X',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as number;
        if (editMode === 'read-only') {
          return <span>{value || 0}</span>;
        }
        return (
          <EditableCell
            value={value?.toString() || '0'}
            type="number"
            onChange={(newValue) => handleCellEdit(row.original.id, 'x', parseFloat(newValue) || 0)}
            errors={getCellErrors(row.original.id, 'x')}
          />
        );
      },
    },
    {
      accessorKey: 'y',
      header: 'Y',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as number;
        if (editMode === 'read-only') {
          return <span>{value || 0}</span>;
        }
        return (
          <EditableCell
            value={value?.toString() || '0'}
            type="number"
            onChange={(newValue) => handleCellEdit(row.original.id, 'y', parseFloat(newValue) || 0)}
          />
        );
      },
    },
    {
      accessorKey: 'weight',
      header: 'Weight',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as number;
        if (editMode === 'read-only') {
          return <span>{value || 0}</span>;
        }
        return (
          <EditableCell
            value={value?.toString() || '0'}
            type="number"
            onChange={(newValue) => handleCellEdit(row.original.id, 'weight', parseFloat(newValue) || 0)}
          />
        );
      },
    },
  ], [editMode]);

  // Edge columns configuration
  const edgeColumns = useMemo<ColumnDef<GraphEdge>[]>(() => [
    {
      accessorKey: 'id',
      header: 'ID',
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono text-sm">{getValue() as string}</span>
      ),
    },
    {
      accessorKey: 'source',
      header: 'Source',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.id, 'source', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'target',
      header: 'Target',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.id, 'target', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'label',
      header: 'Label',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value || ''}</span>;
        }
        return (
          <EditableCell
            value={value || ''}
            onChange={(newValue) => handleCellEdit(row.original.id, 'label', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'weight',
      header: 'Weight',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as number;
        if (editMode === 'read-only') {
          return <span>{value || 0}</span>;
        }
        return (
          <EditableCell
            value={value?.toString() || '0'}
            type="number"
            onChange={(newValue) => handleCellEdit(row.original.id, 'weight', parseFloat(newValue) || 0)}
          />
        );
      },
    },
  ], [editMode]);

  // Layer columns configuration
  const layerColumns = useMemo<ColumnDef<GraphLayer>[]>(() => [
    {
      accessorKey: 'id',
      header: 'ID',
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono text-sm">{getValue() as string}</span>
      ),
    },
    {
      accessorKey: 'name',
      header: 'Name',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.id, 'name', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'color',
      header: 'Color',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        return (
          <div className="flex items-center space-x-2">
            <div 
              className="w-4 h-4 rounded border border-gray-300"
              style={{ backgroundColor: value }}
            />
            {editMode === 'read-only' ? (
              <span className="font-mono text-sm">{value}</span>
            ) : (
              <EditableCell
                value={value}
                onChange={(newValue) => handleCellEdit(row.original.id, 'color', newValue)}
              />
            )}
          </div>
        );
      },
    },
    {
      accessorKey: 'description',
      header: 'Description',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value || ''}</span>;
        }
        return (
          <EditableCell
            value={value || ''}
            onChange={(newValue) => handleCellEdit(row.original.id, 'description', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'visible',
      header: 'Visible',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as boolean;
        if (editMode === 'read-only') {
          return value ? <Eye className="w-4 h-4 text-green-600" /> : <EyeOff className="w-4 h-4 text-gray-400" />;
        }
        return (
          <button
            onClick={() => handleCellEdit(row.original.id, 'visible', !value)}
            className="p-1 rounded hover:bg-gray-100"
          >
            {value ? <Eye className="w-4 h-4 text-green-600" /> : <EyeOff className="w-4 h-4 text-gray-400" />}
          </button>
        );
      },
    },
  ], [editMode]);

  const getCurrentData = () => {
    if (!graphData) return [];
    
    switch (activeTab) {
      case 'nodes':
        return graphData.nodes || [];
      case 'edges':
        return graphData.edges || [];
      case 'layers':
        return graphData.layers || [];
      default:
        return [];
    }
  };

  const getCurrentColumns = () => {
    switch (activeTab) {
      case 'nodes':
        return nodeColumns;
      case 'edges':
        return edgeColumns;
      case 'layers':
        return layerColumns;
      default:
        return [];
    }
  };

  const table = useReactTable({
    data: getCurrentData(),
    columns: getCurrentColumns(),
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: {
      pagination: {
        pageSize: 50,
      },
    },
  });

  const getCellErrors = useCallback((id: string, field: string): ValidationError[] => {
    return validationErrors.filter(error => error.id === id && error.field === field);
  }, [validationErrors]);

  const handleCellEdit = useCallback((id: string, field: string, value: any) => {
    if (editMode === 'read-only') return;
    
    setIsEditing(true);
    
    // Track changes for transformation node strategy
    setPendingChanges(prev => {
      const newChanges = { ...prev };
      
      switch (activeTab) {
        case 'nodes':
          const nodeUpdate = newChanges.nodes.updated.find(u => u.id === id);
          if (nodeUpdate) {
            nodeUpdate.changes[field] = value;
          } else {
            newChanges.nodes.updated.push({ id, changes: { [field]: value } });
          }
          break;
        case 'edges':
          const edgeUpdate = newChanges.edges.updated.find(u => u.id === id);
          if (edgeUpdate) {
            edgeUpdate.changes[field] = value;
          } else {
            newChanges.edges.updated.push({ id, changes: { [field]: value } });
          }
          break;
        case 'layers':
          const layerUpdate = newChanges.layers.updated.find(u => u.id === id);
          if (layerUpdate) {
            layerUpdate.changes[field] = value;
          } else {
            newChanges.layers.updated.push({ id, changes: { [field]: value } });
          }
          break;
      }
      
      return newChanges;
    });

    // Re-validate after changes
    setTimeout(() => {
      validateAllData();
    }, 0);

    if (onDataChange) {
      onDataChange(pendingChanges);
    }
  }, [activeTab, editMode, pendingChanges, onDataChange, validateAllData]);

  const getValidationSummary = () => {
    const errorCount = validationErrors.filter(e => e.severity === 'error').length;
    const warningCount = validationErrors.filter(e => e.severity === 'warning').length;
    
    if (errorCount === 0 && warningCount === 0) return null;
    
    const parts = [];
    if (errorCount > 0) parts.push(`${errorCount} error${errorCount !== 1 ? 's' : ''}`);
    if (warningCount > 0) parts.push(`${warningCount} warning${warningCount !== 1 ? 's' : ''}`);
    
    return parts.join(', ');
  };

  const canCommitChanges = () => {
    const hasErrors = validationErrors.some(e => e.severity === 'error');
    return !hasErrors && isEditing;
  };

  const handleCommitChanges = async () => {
    if (editMode !== 'transformation') return;
    
    if (!canCommitChanges()) {
      alert('Cannot commit changes: there are validation errors that must be fixed first.');
      return;
    }
    
    try {
      // Create transformation node with changes
      await createTransformationNode({
        variables: {
          input: {
            plan_id: planId,
            node_type: 'transform',
            name: `Manual Edits - ${new Date().toLocaleDateString()}`,
            configuration: JSON.stringify({
              operation: 'manual_edit',
              changes: pendingChanges,
              metadata: {
                edited_at: new Date().toISOString(),
                edit_source: 'data_grid',
              },
            }),
          },
        },
      });
      
      // Reset pending changes
      setPendingChanges({
        nodes: { added: [], updated: [], deleted: [] },
        edges: { added: [], updated: [], deleted: [] },
        layers: { added: [], updated: [], deleted: [] },
      });
      setIsEditing(false);
      setValidationErrors([]);
      
      // Refetch data
      refetch();
    } catch (error) {
      console.error('Failed to commit changes:', error);
    }
  };

  const handleDiscardChanges = () => {
    setPendingChanges({
      nodes: { added: [], updated: [], deleted: [] },
      edges: { added: [], updated: [], deleted: [] },
      layers: { added: [], updated: [], deleted: [] },
    });
    setIsEditing(false);
    refetch();
  };

  const getChangesSummary = () => {
    const totalChanges = 
      pendingChanges.nodes.updated.length + 
      pendingChanges.edges.updated.length + 
      pendingChanges.layers.updated.length;
    
    if (totalChanges === 0) return null;
    
    return `${totalChanges} change${totalChanges !== 1 ? 's' : ''} pending`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (error || !data?.graph_artifact) {
    return (
      <ErrorMessage
        title="Failed to load graph data"
        message={error?.message || 'Graph data could not be found'}
      />
    );
  }

  if (!graphData) {
    return (
      <ErrorMessage
        title="Invalid graph data"
        message="The graph data format is invalid or corrupted"
      />
    );
  }

  return (
    <Card className="h-full flex flex-col">
      {/* Header with tabs and actions */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-1">
            {(['nodes', 'edges', 'layers'] as TabType[]).map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`
                  px-3 py-2 text-sm font-medium rounded-md transition-colors
                  ${activeTab === tab
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300'
                    : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }
                `}
              >
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
                <span className="ml-1 text-xs text-gray-500">
                  ({getCurrentData().length})
                </span>
              </button>
            ))}
          </div>
          
          <div className="flex items-center space-x-2">
            {getChangesSummary() && (
              <span className="text-sm text-blue-600 dark:text-blue-400">
                {getChangesSummary()}
              </span>
            )}
            
            {getValidationSummary() && (
              <span className={`text-sm ${validationErrors.some(e => e.severity === 'error') 
                ? 'text-red-600 dark:text-red-400' 
                : 'text-yellow-600 dark:text-yellow-400'
              }`}>
                {getValidationSummary()}
              </span>
            )}
            
            {editMode !== 'read-only' && isEditing && (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleDiscardChanges}
                  className="text-red-600 hover:text-red-700"
                >
                  <X className="w-4 h-4 mr-1" />
                  Discard
                </Button>
                <Button
                  size="sm"
                  onClick={handleCommitChanges}
                  disabled={!canCommitChanges()}
                  className={canCommitChanges() 
                    ? "bg-blue-600 hover:bg-blue-700" 
                    : "bg-gray-400 cursor-not-allowed"
                  }
                  title={!canCommitChanges() ? "Fix validation errors before committing" : ""}
                >
                  <Check className="w-4 h-4 mr-1" />
                  Commit Changes
                </Button>
              </>
            )}
            
            <Button 
              variant="outline" 
              size="sm"
              onClick={refreshVisualization}
              disabled={!isVisualizationConnected}
              title={isVisualizationConnected ? "Refresh visualization" : "No visualization connected"}
            >
              <RefreshCw className="w-4 h-4 mr-1" />
              Sync
            </Button>
            
            <Button variant="outline" size="sm">
              <Download className="w-4 h-4 mr-1" />
              Export
            </Button>
          </div>
        </div>
      </div>

      {/* Data table */}
      <div className="flex-1 overflow-auto">
        <table className="w-full">
          <thead className="bg-gray-50 dark:bg-gray-800 sticky top-0">
            {table.getHeaderGroups().map(headerGroup => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map(header => (
                  <th 
                    key={header.id}
                    className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider border-b border-gray-200 dark:border-gray-700"
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
            {table.getRowModel().rows.map(row => (
              <tr 
                key={row.id}
                className="hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                {row.getVisibleCells().map(cell => (
                  <td 
                    key={cell.id}
                    className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="p-4 border-t border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="text-sm text-gray-700 dark:text-gray-300">
            Showing {table.getState().pagination.pageIndex * table.getState().pagination.pageSize + 1} to{' '}
            {Math.min(
              (table.getState().pagination.pageIndex + 1) * table.getState().pagination.pageSize,
              table.getPrePaginationRowModel().rows.length
            )} of {table.getPrePaginationRowModel().rows.length} entries
          </div>
          
          <div className="flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              Next
            </Button>
          </div>
        </div>
      </div>
    </Card>
  );
};

// Editable cell component
interface EditableCellProps {
  value: string;
  type?: 'text' | 'number';
  onChange: (value: string) => void;
  errors?: ValidationError[];
}

const EditableCell: React.FC<EditableCellProps> = ({ 
  value, 
  type = 'text', 
  onChange,
  errors = []
}) => {
  const [editing, setEditing] = useState(false);
  const [editValue, setEditValue] = useState(value);

  const hasErrors = errors.some(e => e.severity === 'error');
  const hasWarnings = errors.some(e => e.severity === 'warning');

  const handleSave = () => {
    onChange(editValue);
    setEditing(false);
  };

  const handleCancel = () => {
    setEditValue(value);
    setEditing(false);
  };

  const getCellClassName = () => {
    let className = "cursor-pointer rounded px-1 py-0.5 block w-full ";
    if (hasErrors) {
      className += "bg-red-50 hover:bg-red-100 border border-red-300 dark:bg-red-900/20 dark:hover:bg-red-900/30 dark:border-red-700";
    } else if (hasWarnings) {
      className += "bg-yellow-50 hover:bg-yellow-100 border border-yellow-300 dark:bg-yellow-900/20 dark:hover:bg-yellow-900/30 dark:border-yellow-700";
    } else {
      className += "hover:bg-gray-100 dark:hover:bg-gray-700";
    }
    return className;
  };

  const getInputClassName = () => {
    let className = "w-full min-w-0 ";
    if (hasErrors) {
      className += "border-red-500 focus:border-red-500 focus:ring-red-500";
    } else if (hasWarnings) {
      className += "border-yellow-500 focus:border-yellow-500 focus:ring-yellow-500";
    }
    return className;
  };

  if (editing) {
    return (
      <div className="relative">
        <Input
          type={type}
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onBlur={handleSave}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleSave();
            if (e.key === 'Escape') handleCancel();
          }}
          className={getInputClassName()}
          autoFocus
        />
        {errors.length > 0 && (
          <div className="absolute z-10 mt-1 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-lg text-xs">
            {errors.map((error, index) => (
              <div 
                key={index}
                className={`${error.severity === 'error' ? 'text-red-600' : 'text-yellow-600'}`}
              >
                {error.message}
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="relative group">
      <span 
        onClick={() => setEditing(true)}
        className={getCellClassName()}
        title={errors.length > 0 ? errors.map(e => e.message).join(', ') : ''}
      >
        {value || <span className="text-gray-400 italic">Click to edit</span>}
      </span>
      {errors.length > 0 && (
        <div className="absolute left-0 top-full z-10 mt-1 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-lg text-xs opacity-0 group-hover:opacity-100 transition-opacity">
          {errors.map((error, index) => (
            <div 
              key={index}
              className={`${error.severity === 'error' ? 'text-red-600' : 'text-yellow-600'}`}
            >
              {error.message}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};