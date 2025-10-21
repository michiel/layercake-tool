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
  Center,
  Select
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
  FileFormat,
  DataType,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  detectFileFormat,
  formatFileSize
} from '../../graphql/datasources'
import {
  CREATE_LIBRARY_SOURCE,
  CreateLibrarySourceInput
} from '../../graphql/librarySources'

interface DataSourceUploaderProps {
  projectId?: number
  mode?: 'project' | 'library'
  opened: boolean
  onClose: () => void
  onSuccess?: (dataSource: any) => void
}

interface FileInfo {
  file: File
  name: string
  format: FileFormat | null
  preview?: string
}

export const DataSourceUploader: React.FC<DataSourceUploaderProps> = ({
  projectId,
  mode = 'project',
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
  const [createLibrarySource, { loading: createLibraryLoading, error: createLibraryError }] =
    useMutation(CREATE_LIBRARY_SOURCE)

  const isLibraryMode = mode === 'library'
  const mutationLoading = isLibraryMode ? createLibraryLoading : createLoading
  const mutationError = isLibraryMode ? createLibraryError : createError

  const form = useForm({
    initialValues: {
      name: '',
      description: '',
      dataType: '' as string
    },
    validate: {
      name: (value) => (value.trim().length > 0 ? null : 'Name is required'),
      dataType: (value) => (value ? null : 'Data type is required')
    }
  })

  // Get available data types based on file format
  const getAvailableDataTypes = (format: FileFormat | null): DataType[] => {
    if (!format) return []
    if (format === FileFormat.CSV || format === FileFormat.TSV) {
      return [DataType.NODES, DataType.EDGES, DataType.LAYERS]
    }
    if (format === FileFormat.JSON) {
      return [DataType.GRAPH]
    }
    return []
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

    const format = detectFileFormat(file.name)
    const preview = await generatePreview(file)

    const fileInfo: FileInfo = {
      file,
      name: file.name.replace(/\.[^/.]+$/, ''), // Remove extension for default name
      format,
      preview
    }

    setSelectedFile(fileInfo)
    setPreviewData(preview)

    // Auto-populate form name
    form.setFieldValue('name', fileInfo.name)

    // Auto-select data type if only one option (JSON -> GRAPH)
    const availableTypes = getAvailableDataTypes(format)
    if (availableTypes.length === 1) {
      form.setFieldValue('dataType', availableTypes[0])
    } else {
      form.setFieldValue('dataType', '')
    }
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

  const handleSubmit = async (values: { name: string; description: string; dataType: string }) => {
    if (!selectedFile || !selectedFile.format) return

    try {
      setUploadProgress(10)

      // Convert file to base64
      const fileContent = await fileToBase64(selectedFile.file)
      setUploadProgress(30)

      setUploadProgress(50)

      let createdRecord: any = null

      if (isLibraryMode) {
        const input: CreateLibrarySourceInput = {
          name: values.name,
          description: values.description || undefined,
          filename: selectedFile.file.name,
          fileContent,
          fileFormat: selectedFile.format,
          dataType: values.dataType as DataType
        }

        const result = await createLibrarySource({
          variables: { input }
        })

        createdRecord = (result.data as any)?.createLibrarySource || null
      } else {
        if (projectId === undefined) {
          throw new Error('projectId is required to create a project data source')
        }

        const input: CreateDataSourceInput = {
          projectId,
          name: values.name,
          description: values.description || undefined,
          filename: selectedFile.file.name,
          fileContent,
          fileFormat: selectedFile.format,
          dataType: values.dataType as DataType
        }

        const result = await createDataSource({
          variables: { input }
        })

        createdRecord = (result.data as any)?.createDataSourceFromFile || null
      }

      setUploadProgress(100)

      // Reset form and close
      form.reset()
      setSelectedFile(null)
      setPreviewData(null)
      setUploadProgress(0)

      if (onSuccess && createdRecord) {
        onSuccess(createdRecord)
      }

      onClose()
    } catch (error) {
      console.error('Upload failed:', error)
      setUploadProgress(0)
      // Error will surface through mutationError
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

  const getFileIcon = (format: FileFormat | null) => {
    if (!format) return <IconFile size={24} color="gray" />
    switch (format) {
      case FileFormat.CSV:
      case FileFormat.TSV:
        return <IconFileTypeCsv size={24} color="green" />
      case FileFormat.JSON:
        return <IconFileText size={24} color="blue" />
      default:
        return <IconFile size={24} color="gray" />
    }
  }

  const isValidFileFormat = selectedFile?.format !== null
  const availableDataTypes = selectedFile ? getAvailableDataTypes(selectedFile.format) : []
  const modalTitle = isLibraryMode ? 'Add Library Source' : 'Upload Data Source'

  return (
    <>
      <Modal
        opened={opened}
        onClose={handleClose}
        title={modalTitle}
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
                        Upload CSV, TSV, or JSON files for your data source
                      </Text>
                      <Text size="xs" c="dimmed" mt="md">
                        Supported formats:
                      </Text>
                      <ul style={{ fontSize: '12px', color: 'var(--mantine-color-dimmed)', margin: '4px 0' }}>
                        <li>CSV/TSV Nodes - id, label, layer, x, y, ...</li>
                        <li>CSV/TSV Edges - id, source, target, label, ...</li>
                        <li>CSV/TSV Layers - id, label, color, ...</li>
                        <li>JSON Graph - {'{nodes: [], edges: [], layers: []}'}</li>
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
                    {getFileIcon(selectedFile.format)}
                    <div>
                      <Text fw={500}>{selectedFile.file.name}</Text>
                      <Group gap="xs" mt="xs">
                        <Badge
                          variant="light"
                          color={isValidFileFormat ? 'blue' : 'red'}
                          size="sm"
                        >
                          {isValidFileFormat && selectedFile.format
                            ? getFileFormatDisplayName(selectedFile.format)
                            : 'Unknown Format'
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

                {!isValidFileFormat && (
                  <Alert
                    icon={<IconAlertCircle size={16} />}
                    title="Unsupported File Format"
                    color="orange"
                    mt="md"
                  >
                    Please upload a CSV (.csv), TSV (.tsv), or JSON (.json) file.
                  </Alert>
                )}

                {previewData && isValidFileFormat && (
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
            {selectedFile && isValidFileFormat && (
              <>
                <Select
                  label="Data Type"
                  placeholder="Select what kind of data this file contains"
                  required
                  data={availableDataTypes.map(type => ({
                    value: type,
                    label: getDataTypeDisplayName(type)
                  }))}
                  {...form.getInputProps('dataType')}
                  description={
                    selectedFile.format === FileFormat.JSON
                      ? 'JSON files must contain a complete graph structure'
                      : 'Choose whether this file contains nodes, edges, or layers'
                  }
                />

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
            {mutationError && (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Upload Failed"
                color="red"
              >
                {mutationError.message}
              </Alert>
            )}

            {/* Progress Bar */}
            {uploadProgress > 0 && uploadProgress < 100 && (
              <Progress value={uploadProgress} animated />
            )}

            {/* Action Buttons */}
            <Group justify="flex-end" gap="sm">
              <Button variant="light" onClick={handleClose} disabled={mutationLoading}>
                Cancel
              </Button>
              <Button
                type="submit"
                loading={mutationLoading}
                disabled={!selectedFile || !isValidFileFormat}
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
        accept=".csv,.tsv,.json"
        style={{ display: 'none' }}
        onChange={handleFileInputChange}
      />
    </>
  )
}
