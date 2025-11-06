import React, { useState } from 'react';
import { IconEye, IconEyeOff } from '@tabler/icons-react';
import { Layer } from '../../graphql/graphs';
import { Stack, Group } from '../layout-primitives';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '../ui/popover';
import { Input } from '../ui/input';
import { Label } from '../ui/label';

interface LayerListItemProps {
  layer: Layer;
  nodeCount: number;
  edgeCount: number;
  isVisible: boolean;
  onVisibilityToggle: () => void;
  onColorChange?: (layerId: string, colorType: 'background' | 'border' | 'text', color: string) => void;
}

export const LayerListItem: React.FC<LayerListItemProps> = ({
  layer,
  nodeCount,
  edgeCount,
  isVisible,
  onVisibilityToggle,
  onColorChange,
}) => {
  const [bgPickerOpen, setBgPickerOpen] = useState(false);
  const [borderPickerOpen, setBorderPickerOpen] = useState(false);
  const [textPickerOpen, setTextPickerOpen] = useState(false);

  const backgroundColor = layer.backgroundColor
    ? `#${layer.backgroundColor}`
    : '#f0f0f0';
  const borderColor = layer.borderColor
    ? `#${layer.borderColor}`
    : '#999';
  const textColor = layer.textColor
    ? `#${layer.textColor}`
    : '#000';

  const handleColorChange = (colorType: 'background' | 'border' | 'text', color: string) => {
    if (onColorChange) {
      // Remove # prefix for storage
      const hexColor = color.replace('#', '');
      onColorChange(layer.layerId, colorType, hexColor);
    }
  };

  return (
    <div
      className="p-2 rounded border mb-2"
      style={{
        opacity: isVisible ? 1 : 0.5,
        backgroundColor: isVisible ? 'white' : '#f8f9fa',
      }}
    >
      <Stack gap="xs">
        <Group justify="between" align="center">
          <Group gap="sm">
            {/* Visibility toggle */}
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={onVisibilityToggle}
              title={isVisible ? 'Hide layer' : 'Show layer'}
            >
              {isVisible ? <IconEye className="h-4 w-4" /> : <IconEyeOff className="h-4 w-4" />}
            </Button>

            {/* Color swatches with pickers */}
            <Group gap="xs">
              {/* Background & Border color picker */}
              <Popover open={bgPickerOpen} onOpenChange={setBgPickerOpen}>
                <PopoverTrigger asChild>
                  <button
                    className="w-5 h-5 rounded cursor-pointer"
                    style={{
                      backgroundColor: backgroundColor,
                      border: `2px solid ${borderColor}`,
                    }}
                    title="Click to change background color"
                    onClick={() => setBgPickerOpen(!bgPickerOpen)}
                  />
                </PopoverTrigger>
                <PopoverContent className="w-auto p-3">
                  <Stack gap="xs">
                    <Label className="text-xs font-medium">Background Color</Label>
                    <Input
                      type="color"
                      value={backgroundColor}
                      onChange={(e) => handleColorChange('background', e.target.value)}
                      className="h-8 w-24"
                    />
                  </Stack>
                </PopoverContent>
              </Popover>

              {/* Border color picker */}
              <Popover open={borderPickerOpen} onOpenChange={setBorderPickerOpen}>
                <PopoverTrigger asChild>
                  <button
                    className="w-5 h-5 rounded cursor-pointer border"
                    style={{
                      backgroundColor: borderColor,
                    }}
                    title="Click to change border color"
                    onClick={() => setBorderPickerOpen(!borderPickerOpen)}
                  />
                </PopoverTrigger>
                <PopoverContent className="w-auto p-3">
                  <Stack gap="xs">
                    <Label className="text-xs font-medium">Border Color</Label>
                    <Input
                      type="color"
                      value={borderColor}
                      onChange={(e) => handleColorChange('border', e.target.value)}
                      className="h-8 w-24"
                    />
                  </Stack>
                </PopoverContent>
              </Popover>

              {/* Text color picker */}
              <Popover open={textPickerOpen} onOpenChange={setTextPickerOpen}>
                <PopoverTrigger asChild>
                  <button
                    className="w-5 h-5 rounded cursor-pointer border"
                    style={{
                      backgroundColor: textColor,
                    }}
                    title="Click to change text color"
                    onClick={() => setTextPickerOpen(!textPickerOpen)}
                  />
                </PopoverTrigger>
                <PopoverContent className="w-auto p-3">
                  <Stack gap="xs">
                    <Label className="text-xs font-medium">Text Color</Label>
                    <Input
                      type="color"
                      value={textColor}
                      onChange={(e) => handleColorChange('text', e.target.value)}
                      className="h-8 w-24"
                    />
                  </Stack>
                </PopoverContent>
              </Popover>
            </Group>

            <p className="text-sm font-medium">
              {layer.name || layer.layerId}
            </p>
          </Group>

          {/* Statistics badges */}
          <Group gap="xs">
            <Badge variant="secondary" className="text-xs">
              {nodeCount}N
            </Badge>
            <Badge variant="secondary" className="text-xs">
              {edgeCount}E
            </Badge>
          </Group>
        </Group>

        {/* Layer ID if different from name */}
        {layer.name && layer.name !== layer.layerId && (
          <p className="text-xs text-muted-foreground font-mono">
            {layer.layerId}
          </p>
        )}
      </Stack>
    </div>
  );
};
