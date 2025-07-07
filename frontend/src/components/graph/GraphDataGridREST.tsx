import React, { useState, useCallback, useMemo } from 'react';
import { 
  getCoreRowModel, 
  useReactTable, 
  flexRender,
  getFilteredRowModel,
  getPaginationRowModel,
  type ColumnDef,
} from '@tanstack/react-table';
import { Plus, Download, Save, X, Check, Eye, EyeOff, RefreshCw } from 'lucide-react';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Loading } from '../ui/Loading';
import { ErrorMessage } from '../ui/ErrorMessage';
import { useGraphData } from '@/hooks/useGraphData';
import type { Node, Edge, Layer } from '@/types/api';

interface GraphDataGridRESTProps {
  projectId: number;
  editMode?: 'transformation' | 'in-place' | 'read-only';
  onDataChange?: (changes: any) => void;
}

type TabType = 'nodes' | 'edges' | 'layers';

export const GraphDataGridREST: React.FC<GraphDataGridRESTProps> = ({
  projectId,
  editMode = 'transformation',
  onDataChange,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('nodes');
  const [selectedRows, setSelectedRows] = useState<string[]>([]);
  const [pendingChanges, setPendingChanges] = useState<any>({});
  const [isEditing, setIsEditing] = useState(false);

  const {
    nodes,
    edges,
    layers,
    isLoading,
    error,
    createNode,
    updateNode,
    deleteNode,
    createEdge,
    updateEdge,
    deleteEdge,
    createLayer,
    updateLayer,
    deleteLayer,
    refetch,
  } = useGraphData(projectId);

  // Node columns configuration
  const nodeColumns = useMemo<ColumnDef<Node>[]>(() => [
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
          />
        );
      },
    },
  ], [editMode]);

  // Edge columns configuration
  const edgeColumns = useMemo<ColumnDef<Edge>[]>(() => [
    {
      accessorKey: 'id',
      header: 'ID',
      enableSorting: true,
      cell: ({ getValue }) => (
        <span className="font-mono text-sm">{getValue() as string}</span>
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
  const layerColumns = useMemo<ColumnDef<Layer>[]>(() => [
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
    switch (activeTab) {
      case 'nodes':
        return nodes;
      case 'edges':
        return edges;
      case 'layers':
        return layers;
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

  const handleCellEdit = useCallback(async (id: string, field: string, value: any) => {
    if (editMode === 'read-only') return;
    
    setIsEditing(true);
    
    try {
      switch (activeTab) {
        case 'nodes':
          await updateNode(id, { [field]: value });
          break;
        case 'edges':
          await updateEdge(id, { [field]: value });
          break;
        case 'layers':
          await updateLayer(id, { [field]: value });
          break;
      }
    } catch (error) {
      console.error('Failed to update:', error);
    }
    
    setIsEditing(false);
  }, [activeTab, editMode, updateNode, updateEdge, updateLayer]);

  const handleAddNew = useCallback(async () => {
    if (editMode === 'read-only') return;

    try {
      switch (activeTab) {
        case 'nodes':
          await createNode({
            node_id: `node_${Date.now()}`,
            label: `New Node`,
            layer_id: '',
            properties: {},
          });
          break;
        case 'edges':
          await createEdge({
            source_node_id: '',
            target_node_id: '',
            properties: {},
          });
          break;
        case 'layers':
          await createLayer({
            layer_id: `layer_${Date.now()}`,
            name: `New Layer`,
            color: '#6366f1',
            properties: {},
          });
          break;
      }
    } catch (error) {
      console.error('Failed to create:', error);
    }
  }, [activeTab, editMode, createNode, createEdge, createLayer]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loading size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load graph data"
        message={error?.message || 'Graph data could not be found'}
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
            {editMode !== 'read-only' && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleAddNew}
                className="flex items-center gap-2"
              >
                <Plus className="w-4 h-4" />
                Add {activeTab.slice(0, -1)}
              </Button>
            )}
            
            <Button 
              variant="outline" 
              size="sm"
              onClick={refetch}
            >
              <RefreshCw className="w-4 h-4 mr-1" />
              Refresh
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
}

const EditableCell: React.FC<EditableCellProps> = ({ 
  value, 
  type = 'text', 
  onChange,
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
      className="cursor-pointer rounded px-1 py-0.5 block w-full hover:bg-gray-100 dark:hover:bg-gray-700"
    >
      {value || <span className="text-gray-400 italic">Click to edit</span>}
    </span>
  );
};