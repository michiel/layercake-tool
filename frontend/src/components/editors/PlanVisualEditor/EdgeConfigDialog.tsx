import { useState, useEffect } from 'react'
import { Modal, TextInput, Select, Button, Group, Stack, Text } from '@mantine/core'
import { useForm } from '@mantine/form'
import { ReactFlowEdge } from '../../../types/plan-dag'

interface EdgeConfigDialogProps {
  edge: ReactFlowEdge | null
  opened: boolean
  onClose: () => void
  onSave: (edgeId: string, updates: Partial<ReactFlowEdge>) => void
  readonly?: boolean
}

interface EdgeFormData {
  label: string
  dataType: 'GRAPH_DATA' | 'GRAPH_REFERENCE'
}

export const EdgeConfigDialog = ({
  edge,
  opened,
  onClose,
  onSave,
  readonly = false
}: EdgeConfigDialogProps) => {
  const [loading, setLoading] = useState(false)

  const form = useForm<EdgeFormData>({
    initialValues: {
      label: '',
      dataType: 'GRAPH_DATA'
    },
    validate: {
      label: (value) => value.trim().length === 0 ? 'Label is required' : null
    }
  })

  // Update form when edge changes
  useEffect(() => {
    if (edge) {
      form.setValues({
        label: edge.label || '',
        dataType: edge.metadata?.dataType || 'GRAPH_DATA'
      })
    }
  }, [edge])

  const handleSubmit = async (values: EdgeFormData) => {
    if (!edge) return

    setLoading(true)
    try {
      // Prepare edge updates
      const updates: Partial<ReactFlowEdge> = {
        label: values.label,
        metadata: {
          ...edge.metadata,
          dataType: values.dataType,
          label: values.label
        },
        style: {
          ...edge.style,
          stroke: values.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96'
        }
      }

      await onSave(edge.id, updates)
      onClose()
    } catch (error) {
      console.error('Failed to update edge:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    form.reset()
    onClose()
  }

  return (
    <Modal
      opened={opened}
      onClose={handleClose}
      title="Edge Configuration"
      size="md"
    >
      <form onSubmit={form.onSubmit(handleSubmit)}>
        <Stack gap="md">
          {edge && (
            <Text size="sm" c="dimmed">
              Connection: {edge.source} → {edge.target}
            </Text>
          )}

          <TextInput
            label="Label"
            placeholder="Enter edge label"
            {...form.getInputProps('label')}
            disabled={readonly || loading}
            required
          />

          <Select
            label="Data Type"
            placeholder="Select data type"
            data={[
              { value: 'GRAPH_DATA', label: 'Graph Data' },
              { value: 'GRAPH_REFERENCE', label: 'Graph Reference' }
            ]}
            {...form.getInputProps('dataType')}
            disabled={readonly || loading}
            required
          />

          <Text size="xs" c="dimmed">
            <strong>Graph Data:</strong> Actual data flow between nodes (gray line)<br/>
            <strong>Graph Reference:</strong> Reference to another graph (blue line)
          </Text>

          {!readonly && (
            <Group justify="flex-end">
              <Button
                variant="subtle"
                onClick={handleClose}
                disabled={loading}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                loading={loading}
              >
                Save Changes
              </Button>
            </Group>
          )}
        </Stack>
      </form>
    </Modal>
  )
}