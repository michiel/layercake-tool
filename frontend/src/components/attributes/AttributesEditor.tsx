import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Stack, Group } from '@/components/layout-primitives';
import { IconPlus, IconTrash } from '@tabler/icons-react';
import { AttributesMap, sanitizeAttributes } from '@/utils/attributes';

type AttributeType = 'string' | 'int';

type AttributeRow = {
  id: string;
  key: string;
  value: string;
  type: AttributeType;
};

interface AttributesEditorProps {
  value: AttributesMap;
  onChange: (next: AttributesMap) => void;
}

const toRows = (value: AttributesMap): AttributeRow[] => {
  return Object.entries(value).map(([key, val], idx) => ({
    id: `${key}-${idx}`,
    key,
    value: String(val),
    type: typeof val === 'number' ? 'int' : 'string',
  }));
};

const rowsToMap = (rows: AttributeRow[]): AttributesMap => {
  const map: AttributesMap = {};
  rows.forEach(row => {
    const trimmedKey = row.key.trim();
    if (!trimmedKey) return;
    if (row.type === 'int') {
      const parsed = parseInt(row.value, 10);
      if (!Number.isNaN(parsed)) {
        map[trimmedKey] = parsed;
      }
      return;
    }
    map[trimmedKey] = row.value;
  });
  return sanitizeAttributes(map);
};

export const AttributesEditor: React.FC<AttributesEditorProps> = ({ value, onChange }) => {
  const [rows, setRows] = useState<AttributeRow[]>(() => toRows(value));

  const emitChange = (nextRows: AttributeRow[]) => {
    setRows(nextRows);
    onChange(rowsToMap(nextRows));
  };

  const updateRow = (id: string, patch: Partial<AttributeRow>) => {
    emitChange(rows.map(row => (row.id === id ? { ...row, ...patch } : row)));
  };

  const addRow = () => {
    emitChange([
      ...rows,
      { id: `row-${Date.now()}`, key: '', value: '', type: 'string' },
    ]);
  };

  const removeRow = (id: string) => {
    emitChange(rows.filter(row => row.id !== id));
  };

  return (
    <Stack gap="sm">
      {rows.map(row => (
        <Group key={row.id} gap="xs" align="center">
          <Input
            placeholder="Key"
            value={row.key}
            onChange={e => updateRow(row.id, { key: e.target.value })}
          />
          <Select
            value={row.type}
            onValueChange={val => updateRow(row.id, { type: val as AttributeType })}
          >
            <SelectTrigger className="w-28">
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="string">String</SelectItem>
              <SelectItem value="int">Integer</SelectItem>
            </SelectContent>
          </Select>
          <Input
            placeholder={row.type === 'int' ? '0' : 'value'}
            value={row.value}
            onChange={e => updateRow(row.id, { value: e.target.value })}
          />
          <Button
            variant="ghost"
            size="icon"
            onClick={() => removeRow(row.id)}
            title="Remove attribute"
          >
            <IconTrash className="h-4 w-4" />
          </Button>
        </Group>
      ))}

      <Button variant="secondary" onClick={addRow} size="sm">
        <IconPlus className="mr-2 h-4 w-4" />
        Add attribute
      </Button>
    </Stack>
  );
};
