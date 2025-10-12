import { Table, ScrollArea, Text, Stack, Group, Badge, Alert, Loader, Center } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { DataSourcePreview } from '../../graphql/preview';

export interface DataPreviewProps {
  preview: DataSourcePreview | null;
  loading?: boolean;
  error?: Error | null;
}

export const DataPreview = ({ preview, loading, error }: DataPreviewProps) => {
  if (loading) {
    return (
      <Center h="100%">
        <Stack align="center" gap="md">
          <Loader size="lg" />
          <Text size="sm" c="dimmed">Loading preview data...</Text>
        </Stack>
      </Center>
    );
  }

  if (error) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="Error loading preview" color="red" m="md">
        {error.message}
      </Alert>
    );
  }

  if (!preview) {
    return (
      <Center h="100%">
        <Stack align="center" gap="md">
          <IconAlertCircle size="3rem" color="gray" />
          <Stack align="center" gap="xs">
            <Text size="lg" fw={500}>No preview data available</Text>
            <Text size="sm" c="dimmed" ta="center" maw={400}>
              This data source hasn't been processed yet. Execute the plan to load and process the data.
            </Text>
          </Stack>
        </Stack>
      </Center>
    );
  }

  if (preview.errorMessage) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} title="Execution Error" color="red" m="md">
        {preview.errorMessage}
      </Alert>
    );
  }

  if (!preview.columns || preview.columns.length === 0) {
    return (
      <Center h="100%">
        <Text size="sm" c="dimmed">No columns available</Text>
      </Center>
    );
  }

  return (
    <Stack gap="md" p="md" h="100%">
      {/* Header with metadata */}
      <Group gap="md" wrap="wrap">
        <Badge variant="light" size="lg">
          {preview.totalRows.toLocaleString()} rows
        </Badge>
        <Badge variant="light" size="lg" color="blue">
          {preview.columns.length} columns
        </Badge>
        <Badge variant="outline" size="lg" color="gray">
          {preview.fileType}
        </Badge>
        {preview.importDate && (
          <Text size="sm" c="dimmed">
            Imported: {new Date(preview.importDate).toLocaleString()}
          </Text>
        )}
      </Group>

      {/* Table */}
      <ScrollArea style={{ flex: 1 }}>
        <Table striped highlightOnHover withTableBorder withColumnBorders>
          <Table.Thead>
            <Table.Tr>
              <Table.Th style={{ minWidth: 60 }}>Row</Table.Th>
              {preview.columns.map((col) => (
                <Table.Th key={col.name} style={{ minWidth: 150 }}>
                  <Stack gap={4}>
                    <Text size="sm" fw={600}>{col.name}</Text>
                    <Group gap={4}>
                      <Badge size="xs" variant="outline" color="gray">
                        {col.dataType}
                      </Badge>
                      {col.nullable && (
                        <Badge size="xs" variant="outline" color="orange">
                          nullable
                        </Badge>
                      )}
                    </Group>
                  </Stack>
                </Table.Th>
              ))}
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {preview.rows.map((row) => (
              <Table.Tr key={row.rowNumber}>
                <Table.Td>
                  <Text size="xs" c="dimmed" ff="monospace">
                    {row.rowNumber}
                  </Text>
                </Table.Td>
                {preview.columns.map((col) => {
                  const value = row.data[col.name];
                  const displayValue = value === null || value === undefined
                    ? <Text c="dimmed" fs="italic">null</Text>
                    : String(value);

                  return (
                    <Table.Td key={col.name}>
                      <Text size="sm" lineClamp={2} title={String(value)}>
                        {displayValue}
                      </Text>
                    </Table.Td>
                  );
                })}
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </ScrollArea>

      {/* Footer info */}
      {preview.rows.length < preview.totalRows && (
        <Text size="xs" c="dimmed" ta="center">
          Showing {preview.rows.length} of {preview.totalRows.toLocaleString()} rows
        </Text>
      )}
    </Stack>
  );
};
