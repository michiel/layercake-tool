import React, { useState, useEffect } from 'react';
import { DagPlan, GraphValidationResult } from '../../types/dag';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Loading } from '../ui/Loading';

interface DagValidationProps {
  dagPlan: DagPlan;
  onValidate?: (result: GraphValidationResult) => void;
  autoValidate?: boolean;
}

export const DagValidation: React.FC<DagValidationProps> = ({
  dagPlan,
  onValidate,
  autoValidate = false,
}) => {
  const [validationResult, setValidationResult] = useState<GraphValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  const validateDag = async () => {
    setIsValidating(true);
    
    try {
      // Perform client-side validation first
      const clientValidation = performClientSideValidation(dagPlan);
      
      // TODO: Call GraphQL mutation for server-side validation
      // const serverValidation = await validateDagOnServer(dagPlan);
      
      setValidationResult(clientValidation);
      
      if (onValidate) {
        onValidate(clientValidation);
      }
    } catch (error) {
      console.error('Validation error:', error);
      setValidationResult({
        is_valid: false,
        errors: ['Validation failed due to unexpected error'],
        warnings: [],
      });
    } finally {
      setIsValidating(false);
    }
  };

  const performClientSideValidation = (dag: DagPlan): GraphValidationResult => {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Check for empty DAG
    if (dag.nodes.length === 0) {
      warnings.push('DAG is empty - no nodes defined');
    }

    // Check for nodes without names
    const nodesWithoutNames = dag.nodes.filter(node => !node.name || node.name.trim() === '');
    if (nodesWithoutNames.length > 0) {
      errors.push(`${nodesWithoutNames.length} nodes are missing names`);
    }

    // Check for invalid JSON configurations
    const invalidConfigurations = dag.nodes.filter(node => {
      try {
        JSON.parse(node.configuration);
        return false;
      } catch {
        return true;
      }
    });
    if (invalidConfigurations.length > 0) {
      errors.push(`${invalidConfigurations.length} nodes have invalid JSON configurations`);
    }

    // Check for disconnected nodes (nodes with no edges)
    const connectedNodeIds = new Set<string>();
    dag.edges.forEach(edge => {
      connectedNodeIds.add(edge.source);
      connectedNodeIds.add(edge.target);
    });
    
    const disconnectedNodes = dag.nodes.filter(node => 
      !connectedNodeIds.has(node.id) && dag.nodes.length > 1
    );
    
    if (disconnectedNodes.length > 0) {
      warnings.push(`${disconnectedNodes.length} nodes are disconnected from the workflow`);
    }

    // Check for cycles (simple cycle detection)
    const cycleDetected = detectCycles(dag);
    if (cycleDetected) {
      errors.push('Cycle detected in DAG - workflows must be acyclic');
    }

    // Check for multiple input nodes
    const inputNodes = dag.nodes.filter(node => node.node_type === 'input');
    if (inputNodes.length > 1) {
      warnings.push('Multiple input nodes detected - consider using merge nodes');
    }

    // Check for nodes without outputs
    const outputNodes = dag.nodes.filter(node => node.node_type === 'output');
    if (outputNodes.length === 0 && dag.nodes.length > 0) {
      warnings.push('No output nodes defined - workflow results may not be saved');
    }

    return {
      is_valid: errors.length === 0,
      errors,
      warnings,
    };
  };

  const detectCycles = (dag: DagPlan): boolean => {
    const visited = new Set<string>();
    const recursionStack = new Set<string>();
    
    const adjacencyList = new Map<string, string[]>();
    
    // Build adjacency list
    dag.nodes.forEach(node => {
      adjacencyList.set(node.id, []);
    });
    
    dag.edges.forEach(edge => {
      const sources = adjacencyList.get(edge.source) || [];
      sources.push(edge.target);
      adjacencyList.set(edge.source, sources);
    });
    
    // DFS to detect cycles
    const hasCycle = (nodeId: string): boolean => {
      if (recursionStack.has(nodeId)) return true;
      if (visited.has(nodeId)) return false;
      
      visited.add(nodeId);
      recursionStack.add(nodeId);
      
      const neighbors = adjacencyList.get(nodeId) || [];
      for (const neighbor of neighbors) {
        if (hasCycle(neighbor)) return true;
      }
      
      recursionStack.delete(nodeId);
      return false;
    };
    
    for (const node of dag.nodes) {
      if (!visited.has(node.id)) {
        if (hasCycle(node.id)) return true;
      }
    }
    
    return false;
  };

  // Auto-validate when DAG changes
  useEffect(() => {
    if (autoValidate && dagPlan.nodes.length > 0) {
      const timeoutId = setTimeout(validateDag, 500); // Debounce validation
      return () => clearTimeout(timeoutId);
    }
  }, [dagPlan, autoValidate]);

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">DAG Validation</h3>
        <Button 
          onClick={validateDag}
          disabled={isValidating}
          variant="outline"
          size="sm"
        >
          {isValidating ? <Loading size="sm" /> : 'Validate'}
        </Button>
      </div>

      {validationResult && (
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <div className={`w-3 h-3 rounded-full ${
              validationResult.is_valid ? 'bg-green-500' : 'bg-red-500'
            }`} />
            <span className={`font-medium ${
              validationResult.is_valid ? 'text-green-700' : 'text-red-700'
            }`}>
              {validationResult.is_valid ? 'Valid DAG' : 'Invalid DAG'}
            </span>
          </div>

          {validationResult.errors.length > 0 && (
            <div className="bg-red-50 border border-red-200 rounded-md p-3">
              <h4 className="font-medium text-red-800 mb-2">Errors:</h4>
              <ul className="list-disc list-inside space-y-1 text-sm text-red-700">
                {validationResult.errors.map((error, index) => (
                  <li key={index}>{error}</li>
                ))}
              </ul>
            </div>
          )}

          {validationResult.warnings.length > 0 && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-md p-3">
              <h4 className="font-medium text-yellow-800 mb-2">Warnings:</h4>
              <ul className="list-disc list-inside space-y-1 text-sm text-yellow-700">
                {validationResult.warnings.map((warning, index) => (
                  <li key={index}>{warning}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </Card>
  );
};