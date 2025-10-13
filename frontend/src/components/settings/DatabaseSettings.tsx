import React, { useState, useEffect } from 'react';
import { Container, Title, Card, Text, Group, Button, Stack, Alert, LoadingOverlay, Badge } from '@mantine/core';
import { IconDatabase, IconRefresh, IconFolder, IconAlertCircle, IconCheck } from '@tabler/icons-react';
import { invoke } from "@tauri-apps/api/primitives";

interface DatabaseInfo {
  path: string;
  size_bytes: number;
  size_mb: string;
  exists: boolean;
}

export const DatabaseSettings: React.FC = () => {
  const [databaseInfo, setDatabaseInfo] = useState<DatabaseInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const loadDatabaseInfo = async () => {
    try {
      const info = await invoke<DatabaseInfo>('get_database_info');
      setDatabaseInfo(info);
      setError(null);
    } catch (err) {
      setError(`Failed to load database info: ${err}`);
    }
  };

  useEffect(() => {
    loadDatabaseInfo();
  }, []);

  const handleReinitialize = async () => {
    if (!window.confirm(
      'Are you sure you want to reinitialise the database?\n\n' +
      'This will DELETE ALL DATA including:\n' +
      '- All projects\n' +
      '- All graphs\n' +
      '- All data sources\n' +
      '- All plans\n\n' +
      'This action CANNOT be undone!'
    )) {
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await invoke<string>('reinitialize_database');
      setSuccess(result);
      await loadDatabaseInfo();

      // Reload the page after a short delay to reconnect to the new database
      setTimeout(() => {
        window.location.reload();
      }, 2000);
    } catch (err) {
      setError(`Failed to reinitialise database: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleShowLocation = async () => {
    try {
      const location = await invoke<string>('show_database_location');
      alert(`Database directory:\n${location}`);
    } catch (err) {
      setError(`Failed to get database location: ${err}`);
    }
  };

  return (
    <Container size="md" py="xl">
      <Stack gap="lg">
        <div>
          <Title order={2}>Database Settings</Title>
          <Text c="dimmed" size="sm" mt="xs">
            Manage your local database
          </Text>
        </div>

        {error && (
          <Alert icon={<IconAlertCircle size={16} />} color="red" title="Error" onClose={() => setError(null)} withCloseButton>
            {error}
          </Alert>
        )}

        {success && (
          <Alert icon={<IconCheck size={16} />} color="green" title="Success" onClose={() => setSuccess(null)} withCloseButton>
            {success}
          </Alert>
        )}

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <LoadingOverlay visible={loading} />

          <Group justify="space-between" mb="md">
            <Group>
              <IconDatabase size={24} />
              <Title order={4}>Database Information</Title>
            </Group>
            <Badge color={databaseInfo?.exists ? 'green' : 'red'}>
              {databaseInfo?.exists ? 'Exists' : 'Not Found'}
            </Badge>
          </Group>

          {databaseInfo && (
            <Stack gap="sm">
              <div>
                <Text size="sm" fw={600}>Path:</Text>
                <Text size="sm" c="dimmed" style={{ wordBreak: 'break-all' }}>
                  {databaseInfo.path}
                </Text>
              </div>

              <div>
                <Text size="sm" fw={600}>Size:</Text>
                <Text size="sm" c="dimmed">
                  {databaseInfo.size_mb} MB ({databaseInfo.size_bytes.toLocaleString()} bytes)
                </Text>
              </div>
            </Stack>
          )}
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Stack gap="md">
            <div>
              <Title order={4} mb="xs">Database Operations</Title>
              <Text size="sm" c="dimmed">
                Manage your database with these operations
              </Text>
            </div>

            <Button
              leftSection={<IconFolder size={16} />}
              variant="light"
              onClick={handleShowLocation}
            >
              Show Database Location
            </Button>

            <Button
              leftSection={<IconRefresh size={16} />}
              color="red"
              onClick={handleReinitialize}
              disabled={loading}
            >
              Reinitialise Database
            </Button>

            <Alert icon={<IconAlertCircle size={16} />} color="yellow" title="Warning">
              Reinitialising the database will permanently delete all data. This action cannot be undone.
              Make sure to export any important data before proceeding.
            </Alert>
          </Stack>
        </Card>
      </Stack>
    </Container>
  );
};
