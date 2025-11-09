import React, { useState, useRef } from 'react'
import { useMutation } from '@apollo/client/react'
import { IconUpload, IconX, IconAlertCircle, IconCheck } from '@tabler/icons-react'
import { BULK_UPLOAD_DATASOURCES, BulkUploadDataSetInput } from '../../graphql/datasets'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { Progress } from '../ui/progress'
import { Spinner } from '../ui/spinner'

interface BulkDataSetUploaderProps {
  projectId: number
  opened: boolean
  onClose: () => void
  onSuccess: () => void
}

interface FileWithData {
  file: File
  name: string
  base64: string | null
  status: 'pending' | 'uploading' | 'success' | 'error'
  errorMessage?: string
}

export const BulkDataSetUploader: React.FC<BulkDataSetUploaderProps> = ({
  projectId,
  opened,
  onClose,
  onSuccess
}) => {
  const [files, setFiles] = useState<FileWithData[]>([])
  const [uploadProgress, setUploadProgress] = useState(0)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [bulkUpload, { loading }] = useMutation(BULK_UPLOAD_DATASOURCES)

  const handleFilesSelected = async (selectedFiles: File[]) => {
    const newFiles: FileWithData[] = []

    for (const file of selectedFiles) {
      // Read file as base64
      const reader = new FileReader()
      const base64Promise = new Promise<string>((resolve, reject) => {
        reader.onload = () => {
          const result = reader.result as string
          // Extract base64 data (remove data:*/*;base64, prefix)
          const base64 = result.split(',')[1]
          resolve(base64)
        }
        reader.onerror = reject
      })

      reader.readAsDataURL(file)

      const base64 = await base64Promise

      // Use filename without extension as name
      const name = file.name.replace(/\.[^/.]+$/, '')

      newFiles.push({
        file,
        name,
        base64,
        status: 'pending'
      })
    }

    setFiles([...files, ...newFiles])
  }

  const handleRemoveFile = (index: number) => {
    setFiles(files.filter((_, i) => i !== index))
  }

  const handleUpload = async () => {
    if (files.length === 0) return

    try {
      setUploadProgress(0)

      // Prepare input
      const filesInput: BulkUploadDataSetInput[] = files.map(f => ({
        name: f.name,
        description: `Uploaded from ${f.file.name}`,
        filename: f.file.name,
        fileContent: f.base64 || ''
      }))

      // Execute bulk upload
      await bulkUpload({
        variables: {
          projectId,
          files: filesInput
        }
      })

      setUploadProgress(100)

      // Clear files and close
      setFiles([])
      onSuccess()
      onClose()
    } catch (error) {
      console.error('Bulk upload failed:', error)
      // TODO: Show error notification
    }
  }

  const handleClose = () => {
    if (!loading) {
      setFiles([])
      setUploadProgress(0)
      onClose()
    }
  }

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || [])
    if (selectedFiles.length > 0) {
      handleFilesSelected(selectedFiles)
    }
    // Reset input so same files can be selected again
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const totalSize = files.reduce((sum, f) => sum + f.file.size, 0)
  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Bulk Upload Data Sets</DialogTitle>
          <DialogDescription>
            Upload multiple CSV, TSV, or JSON files. File types will be automatically detected based on content.
          </DialogDescription>
        </DialogHeader>

        <Stack gap="md" className="py-4">
          {loading && (
            <Progress value={uploadProgress} className="h-2" />
          )}

          <input
            ref={fileInputRef}
            type="file"
            accept=".csv,.tsv,.json"
            multiple
            onChange={handleFileInputChange}
            className="hidden"
          />

          <Button
            variant="secondary"
            onClick={() => fileInputRef.current?.click()}
            disabled={loading}
          >
            <IconUpload className="mr-2 h-4 w-4" />
            Select Files
          </Button>

        {files.length > 0 && (
          <>
            <Alert className="border-blue-200 bg-blue-50 text-blue-900">
              <IconCheck className="h-4 w-4 text-blue-600" />
              <AlertDescription>
                {files.length} file{files.length > 1 ? 's' : ''} selected ({formatSize(totalSize)})
              </AlertDescription>
            </Alert>

            <div className="space-y-2">
              {files.map((fileData, index) => (
                <div key={index} className="flex items-start justify-between p-2 rounded-md border">
                  <div className="flex-1">
                    <p className="text-sm font-medium">{fileData.name}</p>
                    <Group gap="xs" className="mt-1">
                      <p className="text-xs text-muted-foreground">{fileData.file.name}</p>
                      <Badge variant="secondary" className="text-xs">
                        {formatSize(fileData.file.size)}
                      </Badge>
                    </Group>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleRemoveFile(index)}
                    disabled={loading}
                    className="text-red-600 hover:text-red-700"
                  >
                    <IconX className="h-4 w-4" />
                  </Button>
                </div>
              ))}
            </div>
          </>
        )}

          {files.length === 0 && !loading && (
            <Alert className="border-gray-200 bg-gray-50 text-gray-900">
              <IconAlertCircle className="h-4 w-4 text-gray-600" />
              <AlertDescription>
                No files selected. Click "Select Files" to choose files to upload.
              </AlertDescription>
            </Alert>
          )}
        </Stack>

        <DialogFooter>
          <Button
            variant="secondary"
            onClick={handleClose}
            disabled={loading}
          >
            Cancel
          </Button>
          <Button
            onClick={handleUpload}
            disabled={files.length === 0 || loading}
          >
            {loading && <Spinner className="mr-2 h-4 w-4" />}
            <IconUpload className="mr-2 h-4 w-4" />
            Upload {files.length > 0 ? `${files.length} File${files.length > 1 ? 's' : ''}` : ''}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
