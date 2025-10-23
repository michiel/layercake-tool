import React, { useState, useRef, useEffect } from 'react';
import { NodeProps } from 'reactflow';

interface EditableNodeProps extends NodeProps {
  onLabelChange?: (nodeId: string, newLabel: string) => void;
}

export const EditableNode: React.FC<EditableNodeProps> = ({ id, data, selected, onLabelChange }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [label, setLabel] = useState<string>(data.label || '');
  const inputRef = useRef<HTMLInputElement>(null);

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
      onLabelChange(id, label.trim());
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
        padding: '10px 20px',
        borderRadius: '3px',
        background: data.style?.backgroundColor || '#ffffff',
        border: isEditing ? '2px solid #1a73e8' : selected ? '2px solid #1a73e8' : '1px solid transparent',
        color: data.style?.color || '#000000',
        minWidth: '150px',
        textAlign: 'center',
        cursor: isEditing ? 'text' : 'default',
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
            width: '100%',
            border: 'none',
            background: 'transparent',
            outline: 'none',
            textAlign: 'center',
            font: 'inherit',
            color: 'inherit',
          }}
        />
      ) : (
        <div>{label}</div>
      )}
    </div>
  );
};
