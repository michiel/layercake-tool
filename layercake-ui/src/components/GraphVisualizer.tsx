import { useEffect, useRef } from 'react';
import { Typography, Card, Empty } from 'antd';
import { Graph } from '../types';

interface GraphVisualizerProps {
  graph: Graph;
}

const GraphVisualizer: React.FC<GraphVisualizerProps> = ({ graph }) => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Here we would initialize a graph visualization library
    // For example, using a library like vis-network, cytoscape, or d3
    // This is a placeholder for the actual implementation
    
    if (!containerRef.current) return;
    
    const container = containerRef.current;
    container.innerHTML = '';
    
    // For this demo, we'll just show a message
    const messageEl = document.createElement('div');
    messageEl.style.padding = '20px';
    messageEl.style.textAlign = 'center';
    messageEl.style.color = '#666';
    messageEl.innerText = `This is where the graph visualization would be rendered.\n\nGraph contains ${graph.nodes.length} nodes and ${graph.edges.length} edges.`;
    
    container.appendChild(messageEl);
    
    // Cleanup function
    return () => {
      if (container) {
        container.innerHTML = '';
      }
    };
  }, [graph]);

  if (!graph.nodes.length && !graph.edges.length) {
    return <Empty description="No graph data available" />;
  }

  return (
    <div>
      <Card title="Graph Visualization">
        <div 
          ref={containerRef} 
          style={{ 
            width: '100%', 
            height: '400px', 
            border: '1px solid #f0f0f0',
            borderRadius: '4px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
          }}
        ></div>
        <Typography.Text type="secondary" style={{ display: 'block', marginTop: '10px' }}>
          Note: In a real implementation, this would be replaced with an interactive graph visualization
          using a library like vis-network, cytoscape, or d3.js.
        </Typography.Text>
      </Card>
    </div>
  );
};

export default GraphVisualizer;
