import React, { useState, useRef } from 'react'
import { useMutation } from '@apollo/client/react'
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
import { useForm } from 'react-hook-form'
// Note: Using simple file input instead of dropzone for now
import {
  CREATE_DATASOURCE_FROM_FILE,
  CREATE_EMPTY_DATASOURCE,
  BULK_UPLOAD_DATASOURCES,
  IMPORT_DATASOURCES,
  CreateDataSetInput,
  CreateEmptyDataSetInput,
  BulkUploadDataSetInput,
  FileFormat,
  DataType,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  detectFileFormat,
  formatFileSize
} from '../../graphql/datasets'
import {
  CREATE_LIBRARY_SOURCE,
  CreateLibrarySourceInput
} from '../../graphql/librarySources'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Progress } from '../ui/progress'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select'
import { Spinner } from '../ui/spinner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs'
import { Textarea } from '../ui/textarea'

interface DataSetUploaderProps {
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

export const DataSetUploader: React.FC<DataSetUploaderProps> = ({
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

  const [createDataSet, { loading: createLoading, error: createError }] = useMutation(
    CREATE_DATASOURCE_FROM_FILE
  )
  const [createEmptyDataSet, { loading: createEmptyLoading, error: createEmptyError }] = useMutation(
    CREATE_EMPTY_DATASOURCE
  )
  const [createLibrarySource, { loading: createLibraryLoading, error: createLibraryError }] =
    useMutation(CREATE_LIBRARY_SOURCE)
  const [bulkUploadDataSets, { loading: bulkLoading, error: bulkError }] = useMutation(
    BULK_UPLOAD_DATASOURCES
  )
  const [importDataSets, { loading: importLoading, error: importError }] = useMutation(
    IMPORT_DATASOURCES
  )

  const isLibraryMode = mode === 'library'
  const mutationLoading = isLibraryMode
    ? createLibraryLoading
    : (activeTab === 'empty' ? createEmptyLoading : activeTab === 'bulk' ? bulkLoading : activeTab === 'import' ? importLoading : createLoading)
  const mutationError = isLibraryMode
    ? createLibraryError
    : (activeTab === 'empty' ? createEmptyError : activeTab === 'bulk' ? bulkError : activeTab === 'import' ? importError : createError)

  const form = useForm<{name: string; description: string; dataType: string}>({
    defaultValues: {
      name: '',
      description: '',
      dataType: ''
    }
  })

  // Get available data types based on file format or active tab
  const getAvailableDataTypes = (format: FileFormat | null): DataType[] => {
    if (activeTab === 'empty') {
      // For empty datasets, all types are available
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
    form.setValue('name', fileInfo.name)

    // Auto-select data type if only one option (JSON -> GRAPH)
    const availableTypes = getAvailableDataTypes(format)
    if (availableTypes.length === 1) {
      form.setValue('dataType', availableTypes[0])
    } else {
      form.setValue('dataType', '')
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

        const filesInput: BulkUploadDataSetInput[] = bulkFiles.map(f => ({
          name: f.name,
          description: `Uploaded from ${f.file.name}`,
          filename: f.file.name,
          fileContent: f.base64
        }))

        setUploadProgress(50)

        await bulkUploadDataSets({
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

        await importDataSets({
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
        // Create empty dataset
        if (projectId === undefined) {
          throw new Error('projectId is required to create a project data source')
        }

        const input: CreateEmptyDataSetInput = {
          projectId,
          name: values.name,
          description: values.description || undefined,
          dataType: values.dataType as DataType
        }

        setUploadProgress(50)

        await createEmptyDataSet({
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

          const input: CreateDataSetInput = {
            projectId,
            name: values.name,
            description: values.description || undefined,
            filename: selectedFile.file.name,
            fileContent,
            fileFormat: selectedFile.format,
            dataType: values.dataType as DataType
          }

          await createDataSet({
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
            className="border border-dashed min-h-[220px] cursor-pointer"
            onClick={handleManualFileSelect}
          >
            <CardContent className="pt-6">
              <Group justify="center" gap="xl" className="min-h-[180px]">
                <IconCloudUpload size={52} stroke={1.5} className="text-muted-foreground" />
                <div>
                  <p className="text-xl">
                    Click to select file
                  </p>
                  <p className="text-sm text-muted-foreground mt-2">
                    Upload CSV, TSV, or JSON file for your data source
                  </p>
                </div>
              </Group>
            </CardContent>
          </Card>
          <div className="flex justify-center">
            <p className="text-sm text-muted-foreground">or</p>
          </div>
          <Button
            variant="secondary"
            className="w-full"
            onClick={handleManualFileSelect}
          >
            <IconFile className="mr-2 h-4 w-4" />
            Choose File from Computer
          </Button>
        </>
      )
    }

    return (
      <>
        <Card className="border">
          <CardContent className="pt-6">
            <Group justify="between" align="start">
              <Group gap="sm">
                {getFileIcon(selectedFile.format)}
                <div>
                  <p className="font-medium">{selectedFile.file.name}</p>
                  <Group gap="xs" className="mt-2">
                    <Badge
                      variant="secondary"
                      className={isValidFileFormat ? 'bg-blue-100 text-blue-900' : 'bg-red-100 text-red-900'}
                    >
                      {isValidFileFormat && selectedFile.format
                        ? getFileFormatDisplayName(selectedFile.format)
                        : 'Unknown Format'
                      }
                    </Badge>
                    <p className="text-sm text-muted-foreground">
                      {formatFileSize(selectedFile.file.size)}
                    </p>
                  </Group>
                </div>
              </Group>
              <Button
                variant="ghost"
                size="icon"
                onClick={handleRemoveFile}
                className="text-red-600 hover:text-red-700"
              >
                <IconX className="h-4 w-4" />
              </Button>
            </Group>
          </CardContent>
        </Card>

        {!isValidFileFormat && (
          <Alert className="border-orange-200 bg-orange-50 text-orange-900">
            <IconAlertCircle className="h-4 w-4 text-orange-600" />
            <AlertTitle>Unsupported File Format</AlertTitle>
            <AlertDescription>
              Please upload a CSV (.csv), TSV (.tsv), or JSON (.json) file.
            </AlertDescription>
          </Alert>
        )}

        {previewData && isValidFileFormat && (
          <div className="mt-3">
            <p className="text-sm font-medium mb-2">Preview:</p>
            <pre className="text-xs font-mono bg-muted p-2 rounded-md whitespace-pre-wrap max-h-[120px] overflow-auto">
              {previewData}
            </pre>
          </div>
        )}

        {isValidFileFormat && (
          <>
            <div className="space-y-2">
              <Label htmlFor="data-type">Data Type *</Label>
              <Select
                value={form.watch('dataType')}
                onValueChange={(value) => form.setValue('dataType', value)}
              >
                <SelectTrigger id="data-type">
                  <SelectValue placeholder="Select what kind of data this file contains" />
                </SelectTrigger>
                <SelectContent>
                  {availableDataTypes.map(type => (
                    <SelectItem key={type} value={type}>
                      {getDataTypeDisplayName(type)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                {selectedFile?.format === FileFormat.JSON
                  ? 'JSON files must contain a complete graph structure'
                  : 'Choose whether this file contains nodes, edges, or layers'
                }
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                placeholder="Enter a name for this data source"
                {...form.register('name', { required: true })}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                placeholder="Optional description of what this data contains"
                rows={3}
                {...form.register('description')}
              />
            </div>
          </>
        )}
      </>
    )
  }

  const renderEmptyTab = () => {
    return (
      <>
        <Alert className="border-blue-200 bg-blue-50 text-blue-900 mb-4">
          <IconAlertCircle className="h-4 w-4 text-blue-600" />
          <AlertDescription>
            Create an empty data source that can be filled with data using the graph editor.
          </AlertDescription>
        </Alert>

        <div className="space-y-2">
          <Label htmlFor="empty-data-type">Data Type *</Label>
          <Select
            value={form.watch('dataType')}
            onValueChange={(value) => form.setValue('dataType', value)}
          >
            <SelectTrigger id="empty-data-type">
              <SelectValue placeholder="Select what kind of data this will contain" />
            </SelectTrigger>
            <SelectContent>
              {availableDataTypes.map(type => (
                <SelectItem key={type} value={type}>
                  {getDataTypeDisplayName(type)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            Choose what type of data this dataset will contain
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="empty-name">Name *</Label>
          <Input
            id="empty-name"
            placeholder="Enter a name for this data source"
            {...form.register('name', { required: true })}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="empty-description">Description</Label>
          <Textarea
            id="empty-description"
            placeholder="Optional description of what this data will contain"
            rows={3}
            {...form.register('description')}
          />
        </div>
      </>
    )
  }

  const renderImportTab = () => {
    const importInputRef = useRef<HTMLInputElement>(null)

    return (
      <>
        <Alert className="border-blue-200 bg-blue-50 text-blue-900 mb-4">
          <IconAlertCircle className="h-4 w-4 text-blue-600" />
          <AlertDescription>
            Upload an XLSX or ODS file containing data sources. Each sheet will be imported as a data source.
            If a sheet name matches an existing data source, it will be updated.
          </AlertDescription>
        </Alert>

        <input
          ref={importInputRef}
          type="file"
          accept=".xlsx,.ods"
          onChange={(e) => setImportFile(e.target.files?.[0] || null)}
          className="hidden"
        />

        <Button
          variant="secondary"
          className="w-full"
          onClick={() => importInputRef.current?.click()}
        >
          <IconUpload className="mr-2 h-4 w-4" />
          Select Spreadsheet File (.xlsx or .ods)
        </Button>

        {importFile && (
          <Card className="border">
            <CardContent className="pt-6">
              <Group justify="between">
                <div>
                  <p className="font-medium">{importFile.name}</p>
                  <p className="text-sm text-muted-foreground">{formatFileSize(importFile.size)}</p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setImportFile(null)}
                  className="text-red-600 hover:text-red-700"
                >
                  <IconX className="h-4 w-4" />
                </Button>
              </Group>
            </CardContent>
          </Card>
        )}
      </>
    )
  }

  const renderBulkTab = () => {
    const totalSize = bulkFiles.reduce((sum, f) => sum + f.file.size, 0)
    const bulkInputRef = useRef<HTMLInputElement>(null)

    return (
      <>
        <Alert className="border-blue-200 bg-blue-50 text-blue-900 mb-4">
          <IconAlertCircle className="h-4 w-4 text-blue-600" />
          <AlertDescription>
            Upload multiple CSV, TSV, or JSON files at once. File types will be automatically detected.
          </AlertDescription>
        </Alert>

        <input
          ref={bulkInputRef}
          type="file"
          accept=".csv,.tsv,.json"
          multiple
          onChange={(e) => {
            const files = Array.from(e.target.files || [])
            handleBulkFilesSelected(files)
          }}
          className="hidden"
        />

        <Button
          variant="secondary"
          className="w-full"
          onClick={() => bulkInputRef.current?.click()}
          disabled={mutationLoading}
        >
          <IconUpload className="mr-2 h-4 w-4" />
          Select Files
        </Button>

        {bulkFiles.length > 0 && (
          <>
            <Alert className="border-blue-200 bg-blue-50 text-blue-900">
              <IconCheck className="h-4 w-4 text-blue-600" />
              <AlertDescription>
                {bulkFiles.length} file{bulkFiles.length > 1 ? 's' : ''} selected ({formatFileSize(totalSize)})
              </AlertDescription>
            </Alert>

            <div className="space-y-2">
              {bulkFiles.map((fileData, index) => (
                <div key={index} className="flex items-start justify-between p-2 rounded-md border">
                  <div className="flex-1">
                    <p className="text-sm font-medium">{fileData.name}</p>
                    <Group gap="xs" className="mt-1">
                      <p className="text-xs text-muted-foreground">{fileData.file.name}</p>
                      <Badge variant="secondary" className="text-xs">
                        {formatFileSize(fileData.file.size)}
                      </Badge>
                    </Group>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleRemoveBulkFile(index)}
                    disabled={mutationLoading}
                    className="text-red-600 hover:text-red-700"
                  >
                    <IconX className="h-4 w-4" />
                  </Button>
                </div>
              ))}
            </div>
          </>
        )}

        {bulkFiles.length === 0 && (
          <Alert className="border-gray-200 bg-gray-50 text-gray-900">
            <IconAlertCircle className="h-4 w-4 text-gray-600" />
            <AlertDescription>
              No files selected. Click "Select Files" to choose files to upload.
            </AlertDescription>
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
      <Dialog open={opened} onOpenChange={(open) => !open && handleClose()}>
        <DialogContent className="sm:max-w-[800px]">
          <DialogHeader>
            <DialogTitle>{modalTitle}</DialogTitle>
          </DialogHeader>
          <form onSubmit={form.handleSubmit(handleSubmit)}>
            <Stack gap="md" className="py-4">
              {/* Tabs - only show for project mode */}
              {!isLibraryMode && (
                <Tabs value={activeTab} onValueChange={(value) => {
                  setActiveTab(value || 'upload')
                  setSelectedFile(null)
                  setPreviewData(null)
                  form.setValue('dataType', '')
                }}>
                  <TabsList className="grid w-full grid-cols-4">
                    <TabsTrigger value="upload">Upload File</TabsTrigger>
                    <TabsTrigger value="empty">Create Empty</TabsTrigger>
                    <TabsTrigger value="import">Import Sheet</TabsTrigger>
                    <TabsTrigger value="bulk">Bulk Import</TabsTrigger>
                  </TabsList>

                  {/* Upload Tab */}
                  <TabsContent value="upload" className="pt-4">
                    {renderUploadTab()}
                  </TabsContent>

                  {/* Create Empty Tab */}
                  <TabsContent value="empty" className="pt-4">
                    {renderEmptyTab()}
                  </TabsContent>

                  {/* Import Sheet Tab */}
                  <TabsContent value="import" className="pt-4">
                    {renderImportTab()}
                  </TabsContent>

                  {/* Bulk Import Tab */}
                  <TabsContent value="bulk" className="pt-4">
                    {renderBulkTab()}
                  </TabsContent>
                </Tabs>
              )}

              {/* Library mode UI (simplified) */}
              {isLibraryMode && renderUploadTab()}

              {/* Error Display */}
              {mutationError && (
                <Alert variant="destructive">
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertTitle>Upload Failed</AlertTitle>
                  <AlertDescription>{mutationError.message}</AlertDescription>
                </Alert>
              )}

              {/* Progress Bar */}
              {uploadProgress > 0 && uploadProgress < 100 && (
                <Progress value={uploadProgress} className="h-2" />
              )}
            </Stack>

            {/* Action Buttons */}
            <DialogFooter>
              <Button variant="secondary" onClick={handleClose} disabled={mutationLoading}>
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={isSubmitDisabled() || mutationLoading}
              >
                {mutationLoading && <Spinner className="mr-2 h-4 w-4" />}
                {uploadProgress === 100 ? (
                  <IconCheck className="mr-2 h-4 w-4" />
                ) : (
                  <IconUpload className="mr-2 h-4 w-4" />
                )}
                {getSubmitButtonText()}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

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
