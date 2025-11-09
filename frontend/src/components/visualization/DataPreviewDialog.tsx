import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { DataPreview } from './DataPreview';
import { DataSetPreview } from '../../graphql/preview';

interface DataPreviewDialogProps {
  opened: boolean;
  onClose: () => void;
  preview: DataSetPreview | null;
  loading?: boolean;
  error?: Error | null;
  title?: string;
}

export const DataPreviewDialog = ({
  opened,
  onClose,
  preview,
  loading,
  error,
  title
}: DataPreviewDialogProps) => {
  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-[100vw] w-full h-screen p-0 gap-0">
        <DialogHeader className="px-6 py-4 border-b">
          <DialogTitle>{title || 'Data Preview'}</DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-hidden" style={{ height: 'calc(100vh - 60px)' }}>
          <DataPreview preview={preview} loading={loading} error={error} />
        </div>
      </DialogContent>
    </Dialog>
  );
};
