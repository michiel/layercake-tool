import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { ArrowLeft, Download, Upload, Settings as SettingsIcon } from 'lucide-react';
import { Link } from 'react-router-dom';
import { GraphVisualization } from '@/components/graph/GraphVisualization';
import { GraphControls, GraphStats, GraphToolbar } from '@/components/graph/GraphControls';
import { Button } from '@/components/ui/Button';
import { Loading } from '@/components/ui/Loading';
import { ErrorMessage } from '@/components/ui/ErrorMessage';
import { useGraphData } from '@/hooks/useGraphData';
import { useProject } from '@/hooks/useProjects';

export function Graph() {
  const { projectId } = useParams<{ projectId: string }>();
  const parsedProjectId = projectId ? parseInt(projectId, 10) : 0;
  
  const { data: project, isLoading: projectLoading } = useProject(parsedProjectId);
  const { nodes, edges, layers, isLoading, error, refetch } = useGraphData(parsedProjectId);
  
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  if (projectLoading || isLoading) {
    return <Loading />;
  }

  if (error) {
    return <ErrorMessage message="Failed to load graph data" onRetry={refetch} />;
  }

  if (!project) {
    return <ErrorMessage message="Project not found" />;
  }

  const handleNodeClick = (node: any) => {
    setSelectedNode(node.id);
  };

  const handleEdgeClick = (edge: any) => {
    console.log('Edge clicked:', edge);
  };

  const handleZoomIn = () => {
    // Zoom functionality will be handled by D3 in the GraphVisualization component
    console.log('Zoom in');
  };

  const handleZoomOut = () => {
    console.log('Zoom out');
  };

  const handleReset = () => {
    console.log('Reset view');
  };

  const handleToggleSimulation = () => {
    console.log('Toggle simulation');
  };

  const handleExport = () => {
    // Export graph data as JSON
    const graphData = { nodes, edges, layers };
    const dataStr = JSON.stringify(graphData, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${project.name}-graph.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json,.csv';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        console.log('Import file:', file.name);
        // TODO: Implement file import functionality
      }
    };
    input.click();
  };

  const handleLayoutChange = (layout: string) => {
    console.log('Layout changed to:', layout);
    // TODO: Implement layout switching
  };

  return (
    <div className="h-screen flex flex-col">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link to={`/projects/${projectId}`}>
              <Button variant="ghost" size="sm">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Project
              </Button>
            </Link>
            <div>
              <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
                {project.name} - Graph View
              </h1>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Visualize and explore your data graph
              </p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={handleImport}>
              <Upload className="w-4 h-4 mr-2" />
              Import
            </Button>
            <Button variant="outline" size="sm" onClick={handleExport}>
              <Download className="w-4 h-4 mr-2" />
              Export
            </Button>
            <Button 
              variant="outline" 
              size="sm" 
              onClick={() => setShowSettings(!showSettings)}
            >
              <SettingsIcon className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex">
        {/* Left Sidebar */}
        <div className="w-64 bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 p-4 space-y-4">
          <GraphStats
            nodeCount={nodes.length}
            edgeCount={edges.length}
            layerCount={layers.length}
            selectedNode={selectedNode}
          />
          
          <GraphControls
            onZoomIn={handleZoomIn}
            onZoomOut={handleZoomOut}
            onReset={handleReset}
            onToggleSimulation={handleToggleSimulation}
            isSimulationRunning={true}
            onOpenSettings={() => setShowSettings(!showSettings)}
          />
        </div>

        {/* Graph Container */}
        <div className="flex-1 flex flex-col">
          {/* Toolbar */}
          <GraphToolbar
            onExport={handleExport}
            onImport={handleImport}
            onLayout={handleLayoutChange}
            className="m-4 mb-2"
          />
          
          {/* Graph Visualization */}
          <div className="flex-1 p-4 pt-2">
            <GraphVisualization
              nodes={nodes}
              edges={edges}
              layers={layers}
              width={1200}
              height={700}
              onNodeClick={handleNodeClick}
              onEdgeClick={handleEdgeClick}
              className="w-full h-full"
            />
          </div>
        </div>

        {/* Settings Panel */}
        {showSettings && (
          <div className="w-80 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 p-4">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
              Graph Settings
            </h3>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Node Size
                </label>
                <input
                  type="range"
                  min="8"
                  max="20"
                  defaultValue="12"
                  className="w-full"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Link Distance
                </label>
                <input
                  type="range"
                  min="50"
                  max="150"
                  defaultValue="80"
                  className="w-full"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Charge Strength
                </label>
                <input
                  type="range"
                  min="-500"
                  max="-100"
                  defaultValue="-300"
                  className="w-full"
                />
              </div>
              
              <div className="flex items-center">
                <input
                  type="checkbox"
                  id="showLabels"
                  defaultChecked
                  className="mr-2"
                />
                <label htmlFor="showLabels" className="text-sm text-gray-700 dark:text-gray-300">
                  Show Node Labels
                </label>
              </div>
              
              <div className="flex items-center">
                <input
                  type="checkbox"
                  id="showLegend"
                  defaultChecked
                  className="mr-2"
                />
                <label htmlFor="showLegend" className="text-sm text-gray-700 dark:text-gray-300">
                  Show Layer Legend
                </label>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}