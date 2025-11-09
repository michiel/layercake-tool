import React, { useState } from 'react'
import { useQuery } from '@apollo/client/react'
import {
  IconSearch,
  IconFile,
  IconAlertCircle,
  IconCheck,
  IconClock,
  IconX,
  IconRefresh
} from '@tabler/icons-react'
import {
  GET_DATASOURCES,
  DataSet,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  formatFileSize
} from '../../../../graphql/datasets'
import { Stack, Group } from '../../../layout-primitives'
import { Button } from '../../../ui/button'
import { Badge } from '../../../ui/badge'
import { Card } from '../../../ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '../../../ui/dialog'
import { Input } from '../../../ui/input'
import { ScrollArea } from '../../../ui/scroll-area'
import { Alert, AlertDescription } from '../../../ui/alert'
import { Spinner } from '../../../ui/spinner'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../../../ui/tooltip'

interface DataSetSelectionDialogProps {
  opened: boolean
  onClose: () => void
  onSelect: (dataSource: DataSet) => void
  currentDataSetId?: number
  projectId: number
}

export const DataSetSelectionDialog: React.FC<DataSetSelectionDialogProps> = ({
  opened,
  onClose,
  onSelect,
  currentDataSetId,
  projectId
}) => {
  const [searchQuery, setSearchQuery] = useState('')

  // Query for DataSets in current project
  const {
    data: dataSourcesData,
    loading: dataSourcesLoading,
    error: dataSourcesError,
    refetch: refetchDataSets
  } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId || projectId === 0,
    errorPolicy: 'all',
    fetchPolicy: 'cache-and-network'
  })

  const dataSources: DataSet[] = (dataSourcesData as any)?.dataSets || []

  // Filter DataSets based on search query
  const filteredDataSets = dataSources.filter(ds =>
    ds.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    ds.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (ds.description || '').toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleSelect = (dataSource: DataSet) => {
    onSelect(dataSource)
    onClose()
  }

  const getStatusIcon = (status: DataSet['status']) => {
    switch (status) {
      case 'active':
        return <IconCheck size={16} />
      case 'processing':
        return <IconClock size={16} />
      case 'error':
        return <IconX size={16} />
      default:
        return null
    }
  }

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Select Data Source</DialogTitle>
        </DialogHeader>

        <TooltipProvider>
          <Stack gap="md" className="py-4">
            <Group>
              <div className="relative flex-1">
                <IconSearch className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search data sources..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.currentTarget.value)}
                  className="pl-9"
                />
              </div>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="secondary" size="icon" onClick={() => refetchDataSets()}>
                    <IconRefresh className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Refresh data sources</TooltipContent>
              </Tooltip>
            </Group>

            {dataSourcesError && (
              <Alert variant="destructive">
                <IconAlertCircle className="h-4 w-4" />
                <AlertDescription>
                  <p className="font-semibold mb-1">Error Loading Data Sources</p>
                  <p className="text-sm">{dataSourcesError.message}</p>
                  <p className="text-xs mt-1 text-muted-foreground">
                    Project ID: {projectId}, Query Variables: {JSON.stringify({ projectId })}
                  </p>
                </AlertDescription>
              </Alert>
            )}

            {/* Show loading state when projectId is invalid */}
            {(!projectId || projectId === 0) && !dataSourcesError && (
              <Alert>
                <IconAlertCircle className="h-4 w-4" />
                <AlertDescription>
                  <p className="font-semibold mb-1">Invalid Project</p>
                  <p className="text-sm">
                    No valid project ID provided. Project ID: {projectId}
                  </p>
                </AlertDescription>
              </Alert>
            )}

            <div className="relative">
              <ScrollArea className="h-[400px]">
                {dataSourcesLoading && (
                  <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
                    <Spinner className="h-8 w-8" />
                  </div>
                )}

                {filteredDataSets.length === 0 && !dataSourcesLoading ? (
                  <div className="flex flex-col items-center justify-center py-12 gap-4">
                    <IconFile size={48} className="text-gray-400" />
                    <div className="text-center">
                      <h4 className="font-semibold mb-1">No Data Sources Found</h4>
                      <p className="text-sm text-muted-foreground">
                        {searchQuery
                          ? 'No data sources match your search criteria.'
                          : 'Create data sources to use them in your Plan DAG.'
                        }
                      </p>
                    </div>
                  </div>
                ) : (
                  <Stack gap="sm">
                    {filteredDataSets.map((dataSource) => (
                      <Card
                        key={dataSource.id}
                        className={`border p-4 transition-opacity ${
                          dataSource.status === 'active'
                            ? 'cursor-pointer hover:bg-accent'
                            : 'cursor-not-allowed opacity-70'
                        } ${
                          currentDataSetId === dataSource.id
                            ? 'border-blue-500 border-2 bg-blue-50'
                            : ''
                        }`}
                        onClick={() => dataSource.status === 'active' && handleSelect(dataSource)}
                      >
                        <Group justify="between" align="start">
                          <div className="flex-1">
                            <Group gap="sm" className="mb-2">
                              <p className="font-medium">{dataSource.name}</p>
                              {currentDataSetId === dataSource.id && (
                                <Badge variant="secondary">
                                  Current
                                </Badge>
                              )}
                            </Group>

                            {dataSource.description && (
                              <p className="text-sm text-muted-foreground mb-2">
                                {dataSource.description}
                              </p>
                            )}

                            <Group gap="xs" className="mb-2">
                              <Badge
                                variant="secondary"
                                className={`${
                                  dataSource.status === 'active'
                                    ? 'bg-green-100 text-green-900'
                                    : dataSource.status === 'processing'
                                      ? 'bg-blue-100 text-blue-900'
                                      : 'bg-red-100 text-red-900'
                                }`}
                              >
                                <span className="mr-1">{getStatusIcon(dataSource.status)}</span>
                                {dataSource.status}
                              </Badge>
                              <Badge variant="outline">
                                {getFileFormatDisplayName(dataSource.fileFormat)}
                              </Badge>
                              <Badge variant="outline">
                                {getDataTypeDisplayName(dataSource.dataType)}
                              </Badge>
                            </Group>

                            <Group gap="sm" className="items-center">
                              <p className="text-xs text-muted-foreground font-mono">
                                {dataSource.filename}
                              </p>
                              <span className="text-xs text-muted-foreground">•</span>
                              <p className="text-xs text-muted-foreground">
                                {formatFileSize(dataSource.fileSize)}
                              </p>
                              <span className="text-xs text-muted-foreground">•</span>
                              <p className="text-xs text-muted-foreground">
                                {new Date(dataSource.updatedAt).toLocaleDateString()}
                              </p>
                            </Group>

                            {dataSource.status === 'error' && dataSource.errorMessage && (
                              <Group gap="xs" className="mt-2">
                                <IconAlertCircle size={14} className="text-red-600" />
                                <p className="text-xs text-red-600 line-clamp-1">
                                  {dataSource.errorMessage}
                                </p>
                              </Group>
                            )}
                          </div>

                          {dataSource.status === 'active' && (
                            <Button
                              size="sm"
                              variant="secondary"
                              onClick={(e) => {
                                e.stopPropagation()
                                handleSelect(dataSource)
                              }}
                            >
                              Select
                            </Button>
                          )}
                        </Group>
                      </Card>
                    ))}
                  </Stack>
                )}
              </ScrollArea>
            </div>

            <DialogFooter>
              <Button variant="secondary" onClick={onClose}>
                Cancel
              </Button>
            </DialogFooter>
          </Stack>
        </TooltipProvider>
      </DialogContent>
    </Dialog>
  )
}