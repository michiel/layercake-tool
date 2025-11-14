import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag';

export const generateNodeId = (type: PlanDagNodeType, existingNodeIds: string[] = []): string => {
  const typePrefix = type.toLowerCase().replace('_', '');

  // Extract all numeric suffixes from existing node IDs
  // Match any node ID that ends with _NNN (where NNN is digits)
  const numberPattern = /_(\d+)$/;
  const existingNumbers = existingNodeIds
    .map(id => {
      const match = id.match(numberPattern);
      return match ? parseInt(match[1], 10) : 0;
    })
    .filter(num => !isNaN(num));

  // Find the max number and increment
  const maxNumber = existingNumbers.length > 0 ? Math.max(...existingNumbers) : 0;
  const nextNumber = maxNumber + 1;

  // Format with leading zeros (3 digits)
  const paddedNumber = String(nextNumber).padStart(3, '0');

  return `${typePrefix}_${paddedNumber}`;
};

export const getDefaultNodeConfig = (type: PlanDagNodeType): NodeConfig => {
  switch (type) {
    case PlanDagNodeType.DATA_SOURCE:
      return {
        dataSetId: 0
      };

    case PlanDagNodeType.GRAPH:
      return {
        metadata: {}
      };

    case PlanDagNodeType.TRANSFORM:
      return {
        transforms: [
          {
            kind: 'AggregateEdges',
            params: { enabled: true }
          }
        ]
      };

    case PlanDagNodeType.FILTER:
      return {
        query: {
          targets: ['nodes'],
          mode: 'include',
          linkPruningMode: 'autoDropDanglingEdges',
          ruleGroup: { combinator: 'and', rules: [] },
          fieldMetadataVersion: 'v1'
        }
      };

    case PlanDagNodeType.MERGE:
      return {
        mergeStrategy: 'Union',
        conflictResolution: 'PreferFirst'
      };

    case PlanDagNodeType.GRAPH_ARTEFACT:
      return {
        renderTarget: 'DOT',
        outputPath: '',
        renderConfig: {
          containNodes: true,
          orientation: 'TB',
          applyLayers: true,
          builtInStyles: 'light'
        },
        graphConfig: {}
      };
    case PlanDagNodeType.TREE_ARTEFACT:
      return {
        renderTarget: 'PlantUmlMindmap',
        outputPath: '',
        renderConfig: {
          containNodes: true,
          orientation: 'TB',
          applyLayers: true,
          builtInStyles: 'light'
        },
        graphConfig: {}
      };

    default:
      return {} as NodeConfig;
  }
};

export const getDefaultNodeMetadata = (type: PlanDagNodeType): NodeMetadata => {
  const typeNames = {
    [PlanDagNodeType.DATA_SOURCE]: 'Data Source',
    [PlanDagNodeType.GRAPH]: 'Graph',
    [PlanDagNodeType.TRANSFORM]: 'Transform',
    [PlanDagNodeType.FILTER]: 'Filter',
    [PlanDagNodeType.MERGE]: 'Merge',
    [PlanDagNodeType.GRAPH_ARTEFACT]: 'Graph Artefact',
    [PlanDagNodeType.TREE_ARTEFACT]: 'Tree Artefact'
  };

  return {
    label: typeNames[type],
    description: ''
  };
};

export const getNodeColors = () => ({
  [PlanDagNodeType.DATA_SOURCE]: '#51cf66',
  [PlanDagNodeType.GRAPH]: '#339af0',
  [PlanDagNodeType.TRANSFORM]: '#ff8cc8',
  [PlanDagNodeType.FILTER]: '#a78bfa',
  [PlanDagNodeType.MERGE]: '#ffd43b',
  [PlanDagNodeType.GRAPH_ARTEFACT]: '#ff6b6b',
  [PlanDagNodeType.TREE_ARTEFACT]: '#845ef7'
});
