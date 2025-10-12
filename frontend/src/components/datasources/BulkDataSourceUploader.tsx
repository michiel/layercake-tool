import React, { useState } from 'react'
import { useMutation } from '@apollo/client/react'
import {
  Modal,
  Button,
  Stack,
  Text,
  Group,
  FileButton,
  Badge,
  ActionIcon,
  Alert,
  Progress,
  List
} from '@mantine/core'
import { IconUpload, IconX, IconAlertCircle, IconCheck } from '@tabler/icons-react'
import { BULK_UPLOAD_DATASOURCES, BulkUploadDataSourceInput } from '../../graphql/datasources'

interface BulkDataSourceUploaderProps {
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

export const BulkDataSourceUploader: React.FC<BulkDataSourceUploaderProps> = ({
  projectId,
  opened,
  onClose,
  onSuccess
}) => {
  const [files, setFiles] = useState<FileWithData[]>([])
  const [uploadProgress, setUploadProgress] = useState(0)

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
      const filesInput: BulkUploadDataSourceInput[] = files.map(f => ({
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

  const totalSize = files.reduce((sum, f) => sum + f.file.size, 0)
  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Bulk Upload Data Sources"
      size="lg"
    >
      <Stack gap="md">
        <Text size="sm" c="dimmed">
          Upload multiple CSV, TSV, or JSON files. File types will be automatically detected based on content.
        </Text>

        {loading && (
          <Progress value={uploadProgress} size="sm" animated />
        )}

        <FileButton
          onChange={handleFilesSelected}
          accept=".csv,.tsv,.json"
          multiple
        >
          {(props) => (
            <Button
              {...props}
              leftSection={<IconUpload size={16} />}
              variant="light"
              disabled={loading}
            >
              Select Files
            </Button>
          )}
        </FileButton>

        {files.length > 0 && (
          <>
            <Alert icon={<IconCheck size={16} />} color="blue">
              {files.length} file{files.length > 1 ? 's' : ''} selected ({formatSize(totalSize)})
            </Alert>

            <List spacing="xs" size="sm">
              {files.map((fileData, index) => (
                <List.Item key={index}>
                  <Group justify="space-between">
                    <div>
                      <Text size="sm" fw={500}>{fileData.name}</Text>
                      <Group gap="xs">
                        <Text size="xs" c="dimmed">{fileData.file.name}</Text>
                        <Badge size="xs" variant="light">
                          {formatSize(fileData.file.size)}
                        </Badge>
                      </Group>
                    </div>
                    <ActionIcon
                      color="red"
                      variant="subtle"
                      onClick={() => handleRemoveFile(index)}
                      disabled={loading}
                    >
                      <IconX size={16} />
                    </ActionIcon>
                  </Group>
                </List.Item>
              ))}
            </List>
          </>
        )}

        {files.length === 0 && !loading && (
          <Alert icon={<IconAlertCircle size={16} />} color="gray">
            No files selected. Click "Select Files" to choose files to upload.
          </Alert>
        )}

        <Group justify="flex-end" mt="md">
          <Button
            variant="subtle"
            onClick={handleClose}
            disabled={loading}
          >
            Cancel
          </Button>
          <Button
            onClick={handleUpload}
            disabled={files.length === 0 || loading}
            loading={loading}
            leftSection={<IconUpload size={16} />}
          >
            Upload {files.length > 0 ? `${files.length} File${files.length > 1 ? 's' : ''}` : ''}
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}
