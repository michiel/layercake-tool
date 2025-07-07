import React, { useState, useCallback, useMemo } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { 
  ColumnDef, 
  getCoreRowModel, 
  useReactTable, 
  flexRender,
  getFilteredRowModel,
  getPaginationRowModel,
} from '@tanstack/react-table';
import { Plus, Download, Upload, Save, X, Check, Eye, EyeOff } from 'lucide-react';
import { GET_GRAPH_ARTIFACT, CREATE_PLAN_NODE } from '../../graphql/dag';
import { GraphArtifact } from '../../types/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';

interface GraphDataGridProps {
  projectId: number;
  planId: number;
  planNodeId: string;
  editMode?: 'transformation' | 'in-place' | 'read-only';
  syncWithVisualization?: boolean;
  onDataChange?: (changes: GraphDataChanges) => void;
  onNodeSelect?: (nodeIds: string[]) => void;
  onEdgeSelect?: (edgeIds: string[]) => void;
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
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('nodes');
  const [selectedRows, setSelectedRows] = useState<string[]>([]);
  const [pendingChanges, setPendingChanges] = useState<GraphDataChanges>({
    nodes: { added: [], updated: [], deleted: [] },
    edges: { added: [], updated: [], deleted: [] },
    layers: { added: [], updated: [], deleted: [] },
  });
  const [isEditing, setIsEditing] = useState(false);

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

    if (onDataChange) {
      onDataChange(pendingChanges);
    }
  }, [activeTab, editMode, pendingChanges, onDataChange]);

  const handleCommitChanges = async () => {
    if (editMode !== 'transformation') return;
    
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
                  className="bg-blue-600 hover:bg-blue-700"
                >
                  <Check className="w-4 h-4 mr-1" />
                  Commit Changes
                </Button>
              </>
            )}
            
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
}

const EditableCell: React.FC<EditableCellProps> = ({ 
  value, 
  type = 'text', 
  onChange 
}) => {
  const [editing, setEditing] = useState(false);
  const [editValue, setEditValue] = useState(value);

  const handleSave = () => {
    onChange(editValue);
    setEditing(false);
  };

  const handleCancel = () => {
    setEditValue(value);
    setEditing(false);
  };

  if (editing) {
    return (
      <Input
        type={type}
        value={editValue}
        onChange={(e) => setEditValue(e.target.value)}
        onBlur={handleSave}
        onKeyDown={(e) => {
          if (e.key === 'Enter') handleSave();
          if (e.key === 'Escape') handleCancel();
        }}
        className="w-full min-w-0"
        autoFocus
      />
    );
  }

  return (
    <span 
      onClick={() => setEditing(true)}
      className="cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 rounded px-1 py-0.5 block w-full"
    >
      {value || <span className="text-gray-400 italic">Click to edit</span>}
    </span>
  );
};