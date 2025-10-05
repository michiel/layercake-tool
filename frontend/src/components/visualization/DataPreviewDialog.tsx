import { Modal } from '@mantine/core';
import { DataPreview } from './DataPreview';
import { DataSourcePreview } from '../../graphql/preview';

interface DataPreviewDialogProps {
  opened: boolean;
  onClose: () => void;
  preview: DataSourcePreview | null;
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
    <Modal
      opened={opened}
      onClose={onClose}
      title={title || 'Data Preview'}
      size="100%"
      fullScreen
      padding={0}
      styles={{
        body: {
          height: 'calc(100vh - 60px)',
          padding: 0,
        },
      }}
    >
      <DataPreview preview={preview} loading={loading} error={error} />
    </Modal>
  );
};
