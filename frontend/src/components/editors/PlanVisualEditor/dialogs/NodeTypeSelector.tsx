import { Modal, Stack, Button, Text } from '@mantine/core';
import { PlanDagNodeType } from '../../../../types/plan-dag';
import { getNodeIcon, getNodeTypeLabel, getNodeColor } from '../../../../utils/nodeStyles';

interface NodeTypeSelectorProps {
  opened: boolean;
  onClose: () => void;
  onSelect: (nodeType: PlanDagNodeType) => void;
}

export const NodeTypeSelector = ({ opened, onClose, onSelect }: NodeTypeSelectorProps) => {
  const nodeTypes = [
    PlanDagNodeType.GRAPH,
    PlanDagNodeType.TRANSFORM,
    PlanDagNodeType.FILTER,
    PlanDagNodeType.MERGE,
    PlanDagNodeType.COPY,
    PlanDagNodeType.OUTPUT,
  ];

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title="Select Node Type"
      size="sm"
      centered
      withinPortal
    >
      <Text size="sm" c="dimmed" mb="md">
        Choose the type of node to create:
      </Text>
      <Stack gap="xs">
        {nodeTypes.map((nodeType) => (
          <Button
            key={nodeType}
            variant="light"
            fullWidth
            size="md"
            leftSection={getNodeIcon(nodeType, '1.2rem')}
            styles={{
              root: {
                backgroundColor: getNodeColor(nodeType) + '15',
                borderColor: getNodeColor(nodeType),
                color: getNodeColor(nodeType),
                '&:hover': {
                  backgroundColor: getNodeColor(nodeType) + '25',
                },
              },
            }}
            onClick={() => onSelect(nodeType)}
          >
            {getNodeTypeLabel(nodeType)}
          </Button>
        ))}
      </Stack>
    </Modal>
  );
};
