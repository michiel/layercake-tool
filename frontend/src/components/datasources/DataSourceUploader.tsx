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
  Select,
  Tabs,
  FileButton,
  List
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
  CREATE_EMPTY_DATASOURCE,
  BULK_UPLOAD_DATASOURCES,
  IMPORT_DATASOURCES,
  CreateDataSourceInput,
  CreateEmptyDataSourceInput,
  BulkUploadDataSourceInput,
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
  onSuccess?: () => void
}

interface FileInfo {
  file: File
  name: string
  format: FileFormat | null
  preview?: string
}

interface BulkFileWithData {
  file: File
  name: string
  base64: string
}

export const DataSourceUploader: React.FC<DataSourceUploaderProps> = ({
  projectId,
  mode = 'project',
  opened,
  onClose,
  onSuccess
}) => {
  const [activeTab, setActiveTab] = useState<string>('upload')
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null)
  const [uploadProgress, setUploadProgress] = useState(0)
  const [previewData, setPreviewData] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // State for bulk upload
  const [bulkFiles, setBulkFiles] = useState<BulkFileWithData[]>([])

  // State for import sheet
  const [importFile, setImportFile] = useState<File | null>(null)

  const [createDataSource, { loading: createLoading, error: createError }] = useMutation(
    CREATE_DATASOURCE_FROM_FILE
  )
  const [createEmptyDataSource, { loading: createEmptyLoading, error: createEmptyError }] = useMutation(
    CREATE_EMPTY_DATASOURCE
  )
  const [createLibrarySource, { loading: createLibraryLoading, error: createLibraryError }] =
    useMutation(CREATE_LIBRARY_SOURCE)
  const [bulkUploadDataSources, { loading: bulkLoading, error: bulkError }] = useMutation(
    BULK_UPLOAD_DATASOURCES
  )
  const [importDataSources, { loading: importLoading, error: importError }] = useMutation(
    IMPORT_DATASOURCES
  )

  const isLibraryMode = mode === 'library'
  const mutationLoading = isLibraryMode
    ? createLibraryLoading
    : (activeTab === 'empty' ? createEmptyLoading : activeTab === 'bulk' ? bulkLoading : activeTab === 'import' ? importLoading : createLoading)
  const mutationError = isLibraryMode
    ? createLibraryError
    : (activeTab === 'empty' ? createEmptyError : activeTab === 'bulk' ? bulkError : activeTab === 'import' ? importError : createError)

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

  // Get available data types based on file format or active tab
  const getAvailableDataTypes = (format: FileFormat | null): DataType[] => {
    if (activeTab === 'empty') {
      // For empty datasources, all types are available
      return [DataType.NODES, DataType.EDGES, DataType.LAYERS, DataType.GRAPH]
    }

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

  // Handlers for bulk upload
  const handleBulkFilesSelected = async (selectedFiles: File[]) => {
    const newFiles: BulkFileWithData[] = []

    for (const file of selectedFiles) {
      const base64 = await fileToBase64(file)
      const name = file.name.replace(/\.[^/.]+$/, '')

      newFiles.push({
        file,
        name,
        base64
      })
    }

    setBulkFiles([...bulkFiles, ...newFiles])
  }

  const handleRemoveBulkFile = (index: number) => {
    setBulkFiles(bulkFiles.filter((_, i) => i !== index))
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
    try {
      setUploadProgress(10)

      if (activeTab === 'bulk') {
        // Handle bulk upload
        if (projectId === undefined) {
          throw new Error('projectId is required to create project data sources')
        }

        if (bulkFiles.length === 0) return

        const filesInput: BulkUploadDataSourceInput[] = bulkFiles.map(f => ({
          name: f.name,
          description: `Uploaded from ${f.file.name}`,
          filename: f.file.name,
          fileContent: f.base64
        }))

        setUploadProgress(50)

        await bulkUploadDataSources({
          variables: {
            projectId,
            files: filesInput
          }
        })

        setBulkFiles([])
      } else if (activeTab === 'import') {
        // Handle import from spreadsheet
        if (projectId === undefined) {
          throw new Error('projectId is required to import data sources')
        }

        if (!importFile) return

        // Read file as base64
        const arrayBuffer = await importFile.arrayBuffer()
        const bytes = new Uint8Array(arrayBuffer)
        let binary = ''
        for (let i = 0; i < bytes.byteLength; i++) {
          binary += String.fromCharCode(bytes[i])
        }
        const base64 = btoa(binary)

        setUploadProgress(50)

        await importDataSources({
          variables: {
            input: {
              projectId,
              fileContent: base64,
              filename: importFile.name
            }
          }
        })

        setImportFile(null)
      } else if (activeTab === 'empty') {
        // Create empty datasource
        if (projectId === undefined) {
          throw new Error('projectId is required to create a project data source')
        }

        const input: CreateEmptyDataSourceInput = {
          projectId,
          name: values.name,
          description: values.description || undefined,
          dataType: values.dataType as DataType
        }

        setUploadProgress(50)

        await createEmptyDataSource({
          variables: { input }
        })
      } else {
        // Upload single file
        if (!selectedFile || !selectedFile.format) return

        // Convert file to base64
        const fileContent = await fileToBase64(selectedFile.file)
        setUploadProgress(30)

        setUploadProgress(50)

        if (isLibraryMode) {
          const input: CreateLibrarySourceInput = {
            name: values.name,
            description: values.description || undefined,
            filename: selectedFile.file.name,
            fileContent,
            fileFormat: selectedFile.format,
            dataType: values.dataType as DataType
          }

          await createLibrarySource({
            variables: { input }
          })
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

          await createDataSource({
            variables: { input }
          })
        }
      }

      setUploadProgress(100)

      // Reset form and close
      form.reset()
      setSelectedFile(null)
      setPreviewData(null)
      setBulkFiles([])
      setImportFile(null)
      setUploadProgress(0)

      if (onSuccess) {
        onSuccess()
      }

      onClose()
    } catch (error) {
      console.error('Operation failed:', error)
      setUploadProgress(0)
      // Error will surface through mutationError
    }
  }

  const handleClose = () => {
    // Reset all state
    form.reset()
    setSelectedFile(null)
    setPreviewData(null)
    setBulkFiles([])
    setImportFile(null)
    setUploadProgress(0)
    setActiveTab('upload')
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

  // Render functions for each tab
  const renderUploadTab = () => {
    if (!selectedFile) {
      return (
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
                  Click to select file
                </Text>
                <Text size="sm" c="dimmed" inline mt={7}>
                  Upload CSV, TSV, or JSON file for your data source
                </Text>
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
      )
    }

    return (
      <>
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

        {isValidFileFormat && (
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
                selectedFile?.format === FileFormat.JSON
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
      </>
    )
  }

  const renderEmptyTab = () => {
    return (
      <>
        <Alert icon={<IconAlertCircle size={16} />} color="blue" mb="md">
          Create an empty data source that can be filled with data using the graph editor.
        </Alert>

        <Select
          label="Data Type"
          placeholder="Select what kind of data this will contain"
          required
          data={availableDataTypes.map(type => ({
            value: type,
            label: getDataTypeDisplayName(type)
          }))}
          {...form.getInputProps('dataType')}
          description="Choose what type of data this datasource will contain"
        />

        <TextInput
          label="Name"
          placeholder="Enter a name for this data source"
          required
          {...form.getInputProps('name')}
        />

        <Textarea
          label="Description"
          placeholder="Optional description of what this data will contain"
          rows={3}
          {...form.getInputProps('description')}
        />
      </>
    )
  }

  const renderImportTab = () => {
    return (
      <>
        <Alert icon={<IconAlertCircle size={16} />} color="blue" mb="md">
          Upload an XLSX or ODS file containing data sources. Each sheet will be imported as a data source.
          If a sheet name matches an existing data source, it will be updated.
        </Alert>

        <FileButton
          onChange={setImportFile}
          accept=".xlsx,.ods"
        >
          {(props) => (
            <Button
              {...props}
              leftSection={<IconUpload size={16} />}
              variant="light"
              fullWidth
            >
              Select Spreadsheet File (.xlsx or .ods)
            </Button>
          )}
        </FileButton>

        {importFile && (
          <Card withBorder>
            <Group justify="space-between">
              <div>
                <Text fw={500}>{importFile.name}</Text>
                <Text size="sm" c="dimmed">{formatFileSize(importFile.size)}</Text>
              </div>
              <ActionIcon
                variant="subtle"
                color="red"
                onClick={() => setImportFile(null)}
              >
                <IconX size={16} />
              </ActionIcon>
            </Group>
          </Card>
        )}
      </>
    )
  }

  const renderBulkTab = () => {
    const totalSize = bulkFiles.reduce((sum, f) => sum + f.file.size, 0)

    return (
      <>
        <Alert icon={<IconAlertCircle size={16} />} color="blue" mb="md">
          Upload multiple CSV, TSV, or JSON files at once. File types will be automatically detected.
        </Alert>

        <FileButton
          onChange={handleBulkFilesSelected}
          accept=".csv,.tsv,.json"
          multiple
        >
          {(props) => (
            <Button
              {...props}
              leftSection={<IconUpload size={16} />}
              variant="light"
              fullWidth
              disabled={mutationLoading}
            >
              Select Files
            </Button>
          )}
        </FileButton>

        {bulkFiles.length > 0 && (
          <>
            <Alert icon={<IconCheck size={16} />} color="blue">
              {bulkFiles.length} file{bulkFiles.length > 1 ? 's' : ''} selected ({formatFileSize(totalSize)})
            </Alert>

            <List spacing="xs" size="sm">
              {bulkFiles.map((fileData, index) => (
                <List.Item key={index}>
                  <Group justify="space-between">
                    <div>
                      <Text size="sm" fw={500}>{fileData.name}</Text>
                      <Group gap="xs">
                        <Text size="xs" c="dimmed">{fileData.file.name}</Text>
                        <Badge size="xs" variant="light">
                          {formatFileSize(fileData.file.size)}
                        </Badge>
                      </Group>
                    </div>
                    <ActionIcon
                      color="red"
                      variant="subtle"
                      onClick={() => handleRemoveBulkFile(index)}
                      disabled={mutationLoading}
                    >
                      <IconX size={16} />
                    </ActionIcon>
                  </Group>
                </List.Item>
              ))}
            </List>
          </>
        )}

        {bulkFiles.length === 0 && (
          <Alert icon={<IconAlertCircle size={16} />} color="gray">
            No files selected. Click "Select Files" to choose files to upload.
          </Alert>
        )}
      </>
    )
  }

  const isValidFileFormat = selectedFile?.format !== null
  const availableDataTypes = activeTab === 'empty'
    ? getAvailableDataTypes(null)
    : (selectedFile ? getAvailableDataTypes(selectedFile.format) : [])
  const modalTitle = isLibraryMode ? 'Add Library Source' : 'New Data Source'

  // Determine if submit should be disabled
  const isSubmitDisabled = () => {
    if (mutationLoading) return true
    if (activeTab === 'upload') return !selectedFile || !isValidFileFormat
    if (activeTab === 'empty') return false
    if (activeTab === 'bulk') return bulkFiles.length === 0
    if (activeTab === 'import') return !importFile
    return false
  }

  // Determine submit button text
  const getSubmitButtonText = () => {
    if (uploadProgress === 100) return 'Complete!'
    if (activeTab === 'upload') return 'Upload Data Source'
    if (activeTab === 'empty') return 'Create Data Source'
    if (activeTab === 'bulk') return `Upload ${bulkFiles.length} File${bulkFiles.length !== 1 ? 's' : ''}`
    if (activeTab === 'import') return 'Import Data Sources'
    return 'Submit'
  }

  return (
    <>
      <Modal
        opened={opened}
        onClose={handleClose}
        title={modalTitle}
        size="xl"
      >
        <form onSubmit={form.onSubmit(handleSubmit)}>
          <Stack gap="md">
            {/* Tabs - only show for project mode */}
            {!isLibraryMode && (
              <Tabs value={activeTab} onChange={(value) => {
                setActiveTab(value || 'upload')
                setSelectedFile(null)
                setPreviewData(null)
                form.setFieldValue('dataType', '')
              }}>
                <Tabs.List>
                  <Tabs.Tab value="upload">Upload File</Tabs.Tab>
                  <Tabs.Tab value="empty">Create Empty</Tabs.Tab>
                  <Tabs.Tab value="import">Import Sheet</Tabs.Tab>
                  <Tabs.Tab value="bulk">Bulk Import</Tabs.Tab>
                </Tabs.List>

                {/* Upload Tab */}
                <Tabs.Panel value="upload" pt="md">
                  {renderUploadTab()}
                </Tabs.Panel>

                {/* Create Empty Tab */}
                <Tabs.Panel value="empty" pt="md">
                  {renderEmptyTab()}
                </Tabs.Panel>

                {/* Import Sheet Tab */}
                <Tabs.Panel value="import" pt="md">
                  {renderImportTab()}
                </Tabs.Panel>

                {/* Bulk Import Tab */}
                <Tabs.Panel value="bulk" pt="md">
                  {renderBulkTab()}
                </Tabs.Panel>
              </Tabs>
            )}

            {/* Library mode UI (simplified) */}
            {isLibraryMode && renderUploadTab()}

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
                disabled={isSubmitDisabled()}
                leftSection={uploadProgress === 100 ? <IconCheck size={16} /> : <IconUpload size={16} />}
              >
                {getSubmitButtonText()}
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
