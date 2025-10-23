import React, { useState, useRef, useEffect } from 'react';
import { NodeProps } from 'reactflow';

interface EditableLabelNodeProps extends NodeProps {
  onLabelChange?: (parentNodeId: string, newLabel: string) => void;
}

export const EditableLabelNode: React.FC<EditableLabelNodeProps> = ({ id, data, onLabelChange }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [label, setLabel] = useState<string>(data.label || '');
  const inputRef = useRef<HTMLInputElement>(null);

  // Extract parent node ID from label node ID (format: `${parentId}-label`)
  const parentNodeId = id.replace(/-label$/, '');

  useEffect(() => {
    setLabel(data.label || '');
  }, [data.label]);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsEditing(true);
  };

  const handleSave = () => {
    if (onLabelChange && label.trim() !== data.label) {
      onLabelChange(parentNodeId, label.trim());
    }
    setIsEditing(false);
  };

  const handleCancel = () => {
    setLabel(data.label || '');
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  };

  const handleBlur = () => {
    handleCancel();
  };

  const handleInputClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  return (
    <div
      style={{
        background: 'transparent',
        border: 'none',
        fontSize: '11px',
        fontWeight: '500',
        color: data.style?.color || '#666',
        padding: 0,
        cursor: isEditing ? 'text' : 'pointer',
        minWidth: 'auto',
        width: 'auto',
        height: 'auto',
      }}
      onDoubleClick={handleDoubleClick}
    >
      {isEditing ? (
        <input
          ref={inputRef}
          type="text"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          onClick={handleInputClick}
          style={{
            border: '2px solid #1a73e8',
            background: '#fff',
            outline: 'none',
            font: 'inherit',
            color: '#000',
            padding: '2px 4px',
            borderRadius: '2px',
          }}
        />
      ) : (
        <div>{label}</div>
      )}
    </div>
  );
};
