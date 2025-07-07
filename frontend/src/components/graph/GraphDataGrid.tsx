import React, { useState, useCallback, useMemo, useEffect } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { 
  getCoreRowModel, 
  useReactTable, 
  flexRender,
  getFilteredRowModel,
  getPaginationRowModel,
  type ColumnDef,
} from '@tanstack/react-table';
import { Plus, Download, Upload, Save, X, Check, Eye, EyeOff, RefreshCw } from 'lucide-react';
import { 
  GET_GRAPH_DATA, 
  CREATE_NODE, UPDATE_NODE, DELETE_NODE,
  CREATE_EDGE, UPDATE_EDGE, DELETE_EDGE,
  CREATE_LAYER, UPDATE_LAYER, DELETE_LAYER,
  type CreateNodeInput, type UpdateNodeInput,
  type CreateEdgeInput, type UpdateEdgeInput,
  type CreateLayerInput, type UpdateLayerInput,
} from '../../graphql/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';
import { useGraphSync, type GraphVisualizationRef } from '../../hooks/useGraphSync';

interface GraphDataGridProps {
  projectId: number;
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
  id: number;
  project_id: number;
  node_id: string;
  label: string;
  layer_id?: string | null;
  properties?: any;
}

interface GraphEdge {
  id: number;
  project_id: number;
  source_node_id: string;
  target_node_id: string;
  properties?: any;
}

interface GraphLayer {
  id: number;
  project_id: number;
  layer_id: string;
  name: string;
  color?: string | null;
  properties?: any;
}

type TabType = 'nodes' | 'edges' | 'layers';

export const GraphDataGrid: React.FC<GraphDataGridProps> = ({
  projectId,
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
  const { data, loading, error, refetch } = useQuery(GET_GRAPH_DATA, {
    variables: { projectId },
  });

  // GraphQL mutations
  const [createNode] = useMutation(CREATE_NODE);
  const [updateNode] = useMutation(UPDATE_NODE);
  const [deleteNode] = useMutation(DELETE_NODE);
  const [createEdge] = useMutation(CREATE_EDGE);
  const [updateEdge] = useMutation(UPDATE_EDGE);
  const [deleteEdge] = useMutation(DELETE_EDGE);
  const [createLayer] = useMutation(CREATE_LAYER);
  const [updateLayer] = useMutation(UPDATE_LAYER);
  const [deleteLayer] = useMutation(DELETE_LAYER);

  const graphData = useMemo(() => {
    if (!data?.graph_data) return null;
    return data.graph_data;
  }, [data]);

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

  // Validation functions
  const validateNodeData = useCallback((node: GraphNode): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!node.node_id || node.node_id.trim() === '') {
      errors.push({
        type: 'node',
        id: node.node_id,
        field: 'node_id',
        message: 'Node ID cannot be empty',
        severity: 'error'
      });
    }
    
    if (!node.label || node.label.trim() === '') {
      errors.push({
        type: 'node',
        id: node.node_id,
        field: 'label',
        message: 'Node label cannot be empty',
        severity: 'error'
      });
    }
    
    return errors;
  }, []);

  const validateEdgeData = useCallback((edge: GraphEdge): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!edge.source_node_id || edge.source_node_id.trim() === '') {
      errors.push({
        type: 'edge',
        id: edge.id.toString(),
        field: 'source_node_id',
        message: 'Edge source cannot be empty',
        severity: 'error'
      });
    }
    
    if (!edge.target_node_id || edge.target_node_id.trim() === '') {
      errors.push({
        type: 'edge',
        id: edge.id.toString(),
        field: 'target_node_id',
        message: 'Edge target cannot be empty',
        severity: 'error'
      });
    }
    
    if (edge.source_node_id === edge.target_node_id) {
      errors.push({
        type: 'edge',
        id: edge.id.toString(),
        field: 'target_node_id',
        message: 'Edge cannot connect a node to itself',
        severity: 'error'
      });
    }
    
    return errors;
  }, []);

  const validateLayerData = useCallback((layer: GraphLayer): ValidationError[] => {
    const errors: ValidationError[] = [];
    
    if (!layer.layer_id || layer.layer_id.trim() === '') {
      errors.push({
        type: 'layer',
        id: layer.layer_id,
        field: 'layer_id',
        message: 'Layer ID cannot be empty',
        severity: 'error'
      });
    }
    
    if (!layer.name || layer.name.trim() === '') {
      errors.push({
        type: 'layer',
        id: layer.layer_id,
        field: 'name',
        message: 'Layer name cannot be empty',
        severity: 'error'
      });
    }
    
    if (layer.color && !layer.color.match(/^#[0-9A-F]{6}$/i)) {
      errors.push({
        type: 'layer',
        id: layer.layer_id,
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
      accessorKey: 'node_id',
      header: 'Node ID',
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono text-sm">{getValue() as string}</span>
      ),
    },
    {
      accessorKey: 'label',
      header: 'Label',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value}</span>;
        }
        return (
          <EditableCell
            value={value}
            onChange={(newValue) => handleCellEdit(row.original.node_id, 'label', newValue)}
            errors={getCellErrors(row.original.node_id, 'label')}
          />
        );
      },
    },
    {
      accessorKey: 'layer_id',
      header: 'Layer',
      enableSorting: true,
      cell: ({ getValue, row }) => {
        const value = getValue() as string;
        if (editMode === 'read-only') {
          return <span>{value || ''}</span>;
        }
        return (
          <EditableCell
            value={value || ''}
            onChange={(newValue) => handleCellEdit(row.original.node_id, 'layer_id', newValue)}
            errors={getCellErrors(row.original.node_id, 'layer_id')}
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
        <span className="font-mono text-sm">{getValue() as number}</span>
      ),
    },
    {
      accessorKey: 'source_node_id',
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
            onChange={(newValue) => handleCellEdit(row.original.id.toString(), 'source_node_id', newValue)}
          />
        );
      },
    },
    {
      accessorKey: 'target_node_id',
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
            onChange={(newValue) => handleCellEdit(row.original.id.toString(), 'target_node_id', newValue)}
          />
        );
      },
    },
  ], [editMode]);

  // Layer columns configuration
  const layerColumns = useMemo<ColumnDef<GraphLayer>[]>(() => [
    {
      accessorKey: 'layer_id',
      header: 'Layer ID',
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
            onChange={(newValue) => handleCellEdit(row.original.layer_id, 'name', newValue)}
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
              style={{ backgroundColor: value || '#6366f1' }}
            />
            {editMode === 'read-only' ? (
              <span className="font-mono text-sm">{value || '#6366f1'}</span>
            ) : (
              <EditableCell
                value={value || '#6366f1'}
                onChange={(newValue) => handleCellEdit(row.original.layer_id, 'color', newValue)}
              />
            )}
          </div>
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