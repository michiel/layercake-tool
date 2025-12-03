import React, { useMemo, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { AttributesEditor } from './AttributesEditor';
import { AttributesMap, sanitizeAttributes } from '@/utils/attributes';

interface AttributesEditorDialogProps {
  open: boolean;
  title?: string;
  initialValue?: AttributesMap;
  onClose: () => void;
  onSave: (next: AttributesMap) => void;
}

export const AttributesEditorDialog: React.FC<AttributesEditorDialogProps> = ({
  open,
  title = 'Edit Attributes',
  initialValue,
  onClose,
  onSave,
}) => {
  const [draft, setDraft] = useState<AttributesMap>(sanitizeAttributes(initialValue));

  React.useEffect(() => {
    setDraft(sanitizeAttributes(initialValue));
  }, [JSON.stringify(initialValue)]);

  const resetKey = useMemo(() => JSON.stringify(initialValue ?? {}), [open, initialValue]);

  const handleOpenChange = (nextOpen: boolean) => {
    if (!nextOpen) {
      onClose();
      setDraft(sanitizeAttributes(initialValue));
    }
  };

  const handleSave = () => {
    onSave(sanitizeAttributes(draft));
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <AttributesEditor key={resetKey} value={draft} onChange={setDraft} />
        <DialogFooter className="mt-4">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave}>Save</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
