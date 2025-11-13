import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Stack } from '@/components/layout-primitives';
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
    PlanDagNodeType.GRAPH_ARTEFACT,
    PlanDagNodeType.TREE_ARTEFACT,
  ];

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Select Node Type</DialogTitle>
          <DialogDescription>
            Choose the type of node to create:
          </DialogDescription>
        </DialogHeader>
        <Stack gap="sm">
          {nodeTypes.map((nodeType) => (
            <Button
              key={nodeType}
              variant="outline"
              className="w-full justify-start gap-3"
              style={{
                backgroundColor: `${getNodeColor(nodeType)}15`,
                borderColor: getNodeColor(nodeType),
                color: getNodeColor(nodeType),
              }}
              onClick={() => onSelect(nodeType)}
            >
              {getNodeIcon(nodeType, '1.2rem')}
              {getNodeTypeLabel(nodeType)}
            </Button>
          ))}
        </Stack>
      </DialogContent>
    </Dialog>
  );
};
