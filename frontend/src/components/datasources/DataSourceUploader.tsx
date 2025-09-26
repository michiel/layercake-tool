import React, { useState, useRef } from 'react'
import { useMutation } from '@apollo/client/react'
import {
  Modal,
  Stack,
  Group,
  Button,
  TextInput,
  Textarea,
  Text,
  Card,
  Badge,
  Alert,
  Progress,
  ActionIcon,
  Center
} from '@mantine/core'
import {
  IconUpload,
  IconFile,
  IconFileTypeCsv,
  IconFileText,
  IconX,
  IconCheck,
  IconAlertCircle,
  IconCloudUpload
} from '@tabler/icons-react'
import { useForm } from '@mantine/form'
// Note: Using simple file input instead of dropzone for now
import {
  CREATE_DATASOURCE_FROM_FILE,
  CreateDataSourceInput,
  getDataSourceTypeDisplayName,
  formatFileSize
} from '../../graphql/datasources'

interface DataSourceUploaderProps {
  projectId: number
  opened: boolean
  onClose: () => void
  onSuccess?: (dataSource: any) => void
}

interface FileInfo {
  file: File
  name: string
  type: 'csv_nodes' | 'csv_edges' | 'csv_layers' | 'json_graph' | 'unknown'
  preview?: string
}

export const DataSourceUploader: React.FC<DataSourceUploaderProps> = ({
  projectId,
  opened,
  onClose,
  onSuccess
}) => {
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null)
  const [uploadProgress, setUploadProgress] = useState(0)
  const [previewData, setPreviewData] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [createDataSource, { loading: createLoading, error: createError }] = useMutation(
    CREATE_DATASOURCE_FROM_FILE
  )

  const form = useForm({
    initialValues: {
      name: '',
      description: ''
    },
    validate: {
      name: (value) => (value.trim().length > 0 ? null : 'Name is required')
    }
  })

  // Determine file type from filename
  const determineFileType = (filename: string): FileInfo['type'] => {
    const lower = filename.toLowerCase()
    if (lower.includes('node') && lower.endsWith('.csv')) return 'csv_nodes'
    if (lower.includes('edge') && lower.endsWith('.csv')) return 'csv_edges'
    if (lower.includes('layer') && lower.endsWith('.csv')) return 'csv_layers'
    if (lower.endsWith('.json')) return 'json_graph'
    return 'unknown'
  }

  // Generate preview of file content
  const generatePreview = async (file: File): Promise<string> => {
    return new Promise((resolve) => {
      const reader = new FileReader()
      reader.onload = (e) => {
        const content = e.target?.result as string
        // Show first 500 characters
        const preview = content.substring(0, 500) + (content.length > 500 ? '...' : '')
        resolve(preview)
      }
      reader.readAsText(file)
    })
  }

  const handleFileSelect = async (files: File[]) => {
    const file = files[0]
    if (!file) return

    const fileType = determineFileType(file.name)
    const preview = await generatePreview(file)

    const fileInfo: FileInfo = {
      file,
      name: file.name.replace(/\.[^/.]+$/, ''), // Remove extension for default name
      type: fileType,
      preview
    }

    setSelectedFile(fileInfo)
    setPreviewData(preview)

    // Auto-populate form name
    form.setFieldValue('name', fileInfo.name)
  }

  const handleManualFileSelect = () => {
    fileInputRef.current?.click()
  }

  const handleFileInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      handleFileSelect([file])
    }
  }

  const handleRemoveFile = () => {
    setSelectedFile(null)
    setPreviewData(null)
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  // Convert file to base64
  const fileToBase64 = (file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // Remove the data URL prefix (e.g., "data:text/csv;base64,")
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
  }

  const handleSubmit = async (values: { name: string; description: string }) => {
    if (!selectedFile) return

    try {
      setUploadProgress(10)

      // Convert file to base64
      const fileContent = await fileToBase64(selectedFile.file)
      setUploadProgress(30)

      const input: CreateDataSourceInput = {
        projectId,
        name: values.name,
        description: values.description || undefined,
        filename: selectedFile.file.name,
        fileContent
      }

      setUploadProgress(50)

      const result = await createDataSource({
        variables: { input }
      })

      setUploadProgress(100)

      // Reset form and close
      form.reset()
      setSelectedFile(null)
      setPreviewData(null)
      setUploadProgress(0)

      if (onSuccess && result.data) {
        onSuccess((result.data as any).createDataSourceFromFile)
      }

      onClose()
    } catch (error) {
      console.error('Upload failed:', error)
      setUploadProgress(0)
      // Error will be shown through createError
    }
  }

  const handleClose = () => {
    // Reset all state
    form.reset()
    setSelectedFile(null)
    setPreviewData(null)
    setUploadProgress(0)
    onClose()
  }

  const getFileIcon = (type: FileInfo['type']) => {
    switch (type) {
      case 'csv_nodes':
      case 'csv_edges':
      case 'csv_layers':
        return <IconFileTypeCsv size={24} color="green" />
      case 'json_graph':
        return <IconFileText size={24} color="blue" />
      default:
        return <IconFile size={24} color="gray" />
    }
  }

  const isValidFileType = selectedFile?.type !== 'unknown'

  return (
    <>
      <Modal
        opened={opened}
        onClose={handleClose}
        title="Upload Data Source"
        size="lg"
      >
        <form onSubmit={form.onSubmit(handleSubmit)}>
          <Stack gap="md">
            {/* File Upload Area */}
            {!selectedFile ? (
              <>
                <Card
                  withBorder
                  style={{
                    minHeight: 220,
                    cursor: 'pointer',
                    borderStyle: 'dashed',
                    borderWidth: '2px'
                  }}
                  onClick={handleManualFileSelect}
                >
                  <Group justify="center" gap="xl" style={{ minHeight: 180 }}>
                    <IconCloudUpload size={52} stroke={1.5} />
                    <div>
                      <Text size="xl" inline>
                        Click to select files
                      </Text>
                      <Text size="sm" c="dimmed" inline mt={7}>
                        Upload CSV or JSON files for your data source
                      </Text>
                      <Text size="xs" c="dimmed" mt="md">
                        Supported formats:
                      </Text>
                      <ul style={{ fontSize: '12px', color: 'var(--mantine-color-dimmed)', margin: '4px 0' }}>
                        <li>CSV Nodes (nodes.csv) - id, label, layer, x, y, ...</li>
                        <li>CSV Edges (edges.csv) - id, source, target, label, ...</li>
                        <li>CSV Layers (layers.csv) - id, label, color, ...</li>
                        <li>JSON Graph (graph.json) - {'{nodes: [], edges: [], layers: []}'}</li>
                      </ul>
                    </div>
                  </Group>
                </Card>

                <Center>
                  <Text size="sm" c="dimmed">or</Text>
                </Center>

                <Button
                  variant="light"
                  fullWidth
                  leftSection={<IconFile size={16} />}
                  onClick={handleManualFileSelect}
                >
                  Choose File from Computer
                </Button>
              </>
            ) : (
              <Card withBorder>
                <Group justify="space-between" align="flex-start">
                  <Group>
                    {getFileIcon(selectedFile.type)}
                    <div>
                      <Text fw={500}>{selectedFile.file.name}</Text>
                      <Group gap="xs" mt="xs">
                        <Badge
                          variant="light"
                          color={isValidFileType ? 'blue' : 'red'}
                          size="sm"
                        >
                          {isValidFileType && selectedFile.type !== 'unknown'
                            ? getDataSourceTypeDisplayName(selectedFile.type as any)
                            : 'Unknown Type'
                          }
                        </Badge>
                        <Text size="sm" c="dimmed">
                          {formatFileSize(selectedFile.file.size)}
                        </Text>
                      </Group>
                    </div>
                  </Group>
                  <ActionIcon
                    variant="subtle"
                    color="red"
                    onClick={handleRemoveFile}
                  >
                    <IconX size={16} />
                  </ActionIcon>
                </Group>

                {!isValidFileType && (
                  <Alert
                    icon={<IconAlertCircle size={16} />}
                    title="Unsupported File Type"
                    color="orange"
                    mt="md"
                  >
                    Please upload a CSV file with 'node', 'edge', or 'layer' in the filename, or a JSON file.
                  </Alert>
                )}

                {previewData && isValidFileType && (
                  <div style={{ marginTop: '12px' }}>
                    <Text size="sm" fw={500} mb="xs">Preview:</Text>
                    <Text
                      size="xs"
                      ff="monospace"
                      style={{
                        backgroundColor: 'var(--mantine-color-gray-0)',
                        padding: '8px',
                        borderRadius: '4px',
                        whiteSpace: 'pre-wrap',
                        maxHeight: '120px',
                        overflow: 'auto'
                      }}
                    >
                      {previewData}
                    </Text>
                  </div>
                )}
              </Card>
            )}

            {/* Form Fields */}
            {selectedFile && isValidFileType && (
              <>
                <TextInput
                  label="Name"
                  placeholder="Enter a name for this data source"
                  required
                  {...form.getInputProps('name')}
                />

                <Textarea
                  label="Description"
                  placeholder="Optional description of what this data contains"
                  rows={3}
                  {...form.getInputProps('description')}
                />
              </>
            )}

            {/* Error Display */}
            {createError && (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Upload Failed"
                color="red"
              >
                {createError.message}
              </Alert>
            )}

            {/* Progress Bar */}
            {uploadProgress > 0 && uploadProgress < 100 && (
              <Progress value={uploadProgress} animated />
            )}

            {/* Action Buttons */}
            <Group justify="flex-end" gap="sm">
              <Button variant="light" onClick={handleClose} disabled={createLoading}>
                Cancel
              </Button>
              <Button
                type="submit"
                loading={createLoading}
                disabled={!selectedFile || !isValidFileType}
                leftSection={uploadProgress === 100 ? <IconCheck size={16} /> : <IconUpload size={16} />}
              >
                {uploadProgress === 100 ? 'Complete!' : 'Upload Data Source'}
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      {/* Hidden file input for manual selection */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".csv,.json"
        style={{ display: 'none' }}
        onChange={handleFileInputChange}
      />
    </>
  )
}