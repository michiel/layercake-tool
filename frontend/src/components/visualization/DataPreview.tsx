import { IconAlertCircle } from '@tabler/icons-react';
import { DataSetPreview } from '../../graphql/preview';
import { Stack, Group, Center } from '@/components/layout-primitives';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Spinner } from '@/components/ui/spinner';

export interface DataPreviewProps {
  preview: DataSetPreview | null;
  loading?: boolean;
  error?: Error | null;
}

export const DataPreview = ({ preview, loading, error }: DataPreviewProps) => {
  if (loading) {
    return (
      <Center className="h-full">
        <Stack align="center" gap="md">
          <Spinner size="lg" />
          <p className="text-sm text-muted-foreground">Loading preview data...</p>
        </Stack>
      </Center>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive" className="m-4">
        <IconAlertCircle className="h-4 w-4" />
        <AlertTitle>Error loading preview</AlertTitle>
        <AlertDescription>{error.message}</AlertDescription>
      </Alert>
    );
  }

  if (!preview) {
    return (
      <Center className="h-full">
        <Stack align="center" gap="md">
          <IconAlertCircle size={48} className="text-gray-400" />
          <Stack align="center" gap="xs">
            <p className="text-lg font-medium">No preview data available</p>
            <p className="text-sm text-muted-foreground text-center max-w-md">
              This data source hasn't been processed yet. Execute the plan to load and process the data.
            </p>
          </Stack>
        </Stack>
      </Center>
    );
  }

  if (preview.errorMessage) {
    return (
      <Alert variant="destructive" className="m-4">
        <IconAlertCircle className="h-4 w-4" />
        <AlertTitle>Execution Error</AlertTitle>
        <AlertDescription>{preview.errorMessage}</AlertDescription>
      </Alert>
    );
  }

  if (!preview.columns || preview.columns.length === 0) {
    return (
      <Center className="h-full">
        <p className="text-sm text-muted-foreground">No columns available</p>
      </Center>
    );
  }

  return (
    <Stack gap="md" className="p-4 h-full">
      {/* Header with metadata */}
      <Group gap="md" wrap={true}>
        <Badge variant="secondary" className="text-base px-3 py-1">
          {preview.totalRows.toLocaleString()} rows
        </Badge>
        <Badge variant="secondary" className="text-base px-3 py-1 bg-blue-100 text-blue-800">
          {preview.columns.length} columns
        </Badge>
        <Badge variant="outline" className="text-base px-3 py-1">
          {preview.fileType}
        </Badge>
        {preview.importDate && (
          <p className="text-sm text-muted-foreground">
            Imported: {new Date(preview.importDate).toLocaleString()}
          </p>
        )}
      </Group>

      {/* Table */}
      <ScrollArea className="flex-1">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead style={{ minWidth: 60 }}>Row</TableHead>
              {preview.columns.map((col) => (
                <TableHead key={col.name} style={{ minWidth: 150 }}>
                  <Stack gap="xs">
                    <p className="text-sm font-semibold">{col.name}</p>
                    <Group gap="xs">
                      <Badge variant="outline" className="text-xs">
                        {col.dataType}
                      </Badge>
                      {col.nullable && (
                        <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
                          nullable
                        </Badge>
                      )}
                    </Group>
                  </Stack>
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {preview.rows.map((row) => (
              <TableRow key={row.rowNumber}>
                <TableCell>
                  <p className="text-xs text-muted-foreground font-mono">
                    {row.rowNumber}
                  </p>
                </TableCell>
                {preview.columns.map((col) => {
                  const value = row.data[col.name];
                  const displayValue = value === null || value === undefined
                    ? <span className="text-muted-foreground italic">null</span>
                    : String(value);

                  return (
                    <TableCell key={col.name}>
                      <p className="text-sm line-clamp-2" title={String(value)}>
                        {displayValue}
                      </p>
                    </TableCell>
                  );
                })}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </ScrollArea>

      {/* Footer info */}
      {preview.rows.length < preview.totalRows && (
        <p className="text-xs text-muted-foreground text-center">
          Showing {preview.rows.length} of {preview.totalRows.toLocaleString()} rows
        </p>
      )}
    </Stack>
  );
};
