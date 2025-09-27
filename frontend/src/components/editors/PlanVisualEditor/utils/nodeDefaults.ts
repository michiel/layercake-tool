import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag';

export const generateNodeId = (type: PlanDagNodeType): string => {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substr(2, 5);
  const typePrefix = type.toLowerCase().replace('_', '');
  return `${typePrefix}_${timestamp}_${random}`;
};

export const getDefaultNodeConfig = (type: PlanDagNodeType): NodeConfig => {
  switch (type) {
    case PlanDagNodeType.DATA_SOURCE:
      return {
        dataSourceId: 0,
        outputGraphRef: ''
      };

    case PlanDagNodeType.GRAPH:
      return {
        graphId: 0,
        isReference: false,
        metadata: {}
      };

    case PlanDagNodeType.TRANSFORM:
      return {
        inputGraphRef: '',
        outputGraphRef: '',
        transformType: 'PartitionDepthLimit',
        transformConfig: {}
      };

    case PlanDagNodeType.MERGE:
      return {
        inputRefs: [],
        outputGraphRef: '',
        mergeStrategy: 'Union',
        conflictResolution: 'PreferFirst'
      };

    case PlanDagNodeType.COPY:
      return {
        sourceGraphRef: '',
        outputGraphRef: '',
        copyType: 'DeepCopy',
        preserveMetadata: true
      };

    case PlanDagNodeType.OUTPUT:
      return {
        sourceGraphRef: '',
        renderTarget: 'DOT',
        outputPath: '',
        renderConfig: {},
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
    [PlanDagNodeType.MERGE]: 'Merge',
    [PlanDagNodeType.COPY]: 'Copy',
    [PlanDagNodeType.OUTPUT]: 'Output'
  };

  return {
    label: `${typeNames[type]} Node`,
    description: `Unconfigured ${typeNames[type].toLowerCase()} node`
  };
};

export const getNodeColors = () => ({
  [PlanDagNodeType.DATA_SOURCE]: '#51cf66',
  [PlanDagNodeType.GRAPH]: '#339af0',
  [PlanDagNodeType.TRANSFORM]: '#ff8cc8',
  [PlanDagNodeType.MERGE]: '#ffd43b',
  [PlanDagNodeType.COPY]: '#74c0fc',
  [PlanDagNodeType.OUTPUT]: '#ff6b6b'
});