import React, { useState, useEffect } from 'react';
import { IconDatabase, IconRefresh, IconFolder, IconAlertCircle, IconCheck, IconX } from '@tabler/icons-react';
import { invoke } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";
import { Stack, Group } from '../layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '../ui/alert';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Spinner } from '../ui/spinner';

interface DatabaseInfo {
  path: string;
  directory: string;
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

      if (!location) {
        await message('Database directory is unavailable.', { title: 'Database Location' });
        return;
      }

      await message(`Database directory:\n${location}`, { title: 'Database Location' });

      try {
        await open(location);
      } catch (openError) {
        console.warn('Unable to open directory in file manager:', openError);
      }
    } catch (err) {
      setError(`Failed to get database location: ${err}`);
    }
  };

  return (
    <div className="container max-w-3xl py-12">
      <Stack gap="lg">
        <div>
          <h2 className="text-2xl font-bold">Database Settings</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Manage your local database
          </p>
        </div>

        {error && (
          <Alert variant="destructive" className="relative pr-10">
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
            <Button
              variant="ghost"
              size="icon"
              className="absolute top-2 right-2 h-6 w-6"
              onClick={() => setError(null)}
            >
              <IconX className="h-4 w-4" />
            </Button>
          </Alert>
        )}

        {success && (
          <Alert className="relative pr-10 border-green-200 bg-green-50 text-green-900">
            <IconCheck className="h-4 w-4 text-green-600" />
            <AlertTitle>Success</AlertTitle>
            <AlertDescription>{success}</AlertDescription>
            <Button
              variant="ghost"
              size="icon"
              className="absolute top-2 right-2 h-6 w-6"
              onClick={() => setSuccess(null)}
            >
              <IconX className="h-4 w-4" />
            </Button>
          </Alert>
        )}

        <Card className="border shadow-sm relative">
          {loading && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
              <Spinner className="h-8 w-8" />
            </div>
          )}

          <CardContent className="pt-6">
            <Group justify="between" className="mb-4">
              <Group gap="sm">
                <IconDatabase className="h-6 w-6" />
                <h4 className="text-lg font-semibold">Database Information</h4>
              </Group>
              <Badge
                className={databaseInfo?.exists ? 'bg-green-500 hover:bg-green-600' : 'bg-red-500 hover:bg-red-600'}
              >
                {databaseInfo?.exists ? 'Exists' : 'Not Found'}
              </Badge>
            </Group>

            {databaseInfo && (
              <Stack gap="sm">
                <div>
                  <p className="text-sm font-semibold">Path:</p>
                  <p className="text-sm text-muted-foreground break-all">
                    {databaseInfo.path}
                  </p>
                </div>

                <div>
                  <p className="text-sm font-semibold">Directory:</p>
                  <p className="text-sm text-muted-foreground break-all">
                    {databaseInfo.directory}
                  </p>
                </div>

                <div>
                  <p className="text-sm font-semibold">Size:</p>
                  <p className="text-sm text-muted-foreground">
                    {databaseInfo.size_mb} MB ({databaseInfo.size_bytes.toLocaleString()} bytes)
                  </p>
                </div>
              </Stack>
            )}
          </CardContent>
        </Card>

        <Card className="border shadow-sm">
          <CardContent className="pt-6">
            <Stack gap="md">
              <div>
                <h4 className="text-lg font-semibold mb-1">Database Operations</h4>
                <p className="text-sm text-muted-foreground">
                  Manage your database with these operations
                </p>
              </div>

              <Button
                variant="secondary"
                onClick={handleShowLocation}
                className="justify-start"
              >
                <IconFolder className="mr-2 h-4 w-4" />
                Show Database Location
              </Button>

              <Button
                variant="destructive"
                onClick={handleReinitialize}
                disabled={loading}
                className="justify-start"
              >
                <IconRefresh className="mr-2 h-4 w-4" />
                Reinitialise Database
              </Button>

              <Alert className="border-yellow-200 bg-yellow-50 text-yellow-900">
                <IconAlertCircle className="h-4 w-4 text-yellow-600" />
                <AlertTitle>Warning</AlertTitle>
                <AlertDescription>
                  Reinitialising the database will permanently delete all data. This action cannot be undone.
                  Make sure to export any important data before proceeding.
                </AlertDescription>
              </Alert>
            </Stack>
          </CardContent>
        </Card>
      </Stack>
    </div>
  );
};
