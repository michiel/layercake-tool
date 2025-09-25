import { useState, useEffect } from 'react'
import { Modal, Title, Button, Group, Stack, Text, TextInput, Select, Textarea, Alert } from '@mantine/core'
import { useForm } from '@mantine/form'
import { IconAlertCircle, IconCheck } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag'

interface NodeConfigDialogProps {
  opened: boolean
  onClose: () => void
  nodeId: string
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  onSave: (nodeId: string, config: NodeConfig, metadata: NodeMetadata) => void
}

interface FormData {
  label: string
  description: string
  // Config fields will be dynamically added based on node type
  [key: string]: any
}

export const NodeConfigDialog = ({
  opened,
  onClose,
  nodeId,
  nodeType,
  config,
  metadata,
  onSave
}: NodeConfigDialogProps) => {
  const [isValid, setIsValid] = useState(true)
  const [validationErrors, setValidationErrors] = useState<string[]>([])

  const form = useForm<FormData>({
    initialValues: {
      label: metadata.label || '',
      description: metadata.description || '',
      ...getConfigDefaults(nodeType, config),
    },
    validate: {
      label: (value) => (!value ? 'Label is required' : null),
    },
  })

  // Reset form when dialog opens with new data
  useEffect(() => {
    if (opened) {
      form.setValues({
        label: metadata.label || '',
        description: metadata.description || '',
        ...getConfigDefaults(nodeType, config),
      })
      setIsValid(true)
      setValidationErrors([])
    }
  }, [opened, nodeType, config, metadata])

  const handleSave = () => {
    const values = form.values
    const errors = validateNodeConfig(nodeType, values)

    if (errors.length > 0) {
      setIsValid(false)
      setValidationErrors(errors)
      return
    }

    const newMetadata: NodeMetadata = {
      label: values.label,
      description: values.description,
    }

    const newConfig = buildConfigFromForm(nodeType, values)

    onSave(nodeId, newConfig, newMetadata)
    onClose()
  }

  const getNodeTypeLabel = (type: PlanDagNodeType): string => {
    switch (type) {
      case PlanDagNodeType.DATA_SOURCE: return 'Data Source Node'
      case PlanDagNodeType.GRAPH: return 'Graph Node'
      case PlanDagNodeType.TRANSFORM: return 'Transform Node'
      case PlanDagNodeType.MERGE: return 'Merge Node'
      case PlanDagNodeType.COPY: return 'Copy Node'
      case PlanDagNodeType.OUTPUT: return 'Output Node'
      default: return 'Node'
    }
  }

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Group gap="sm">
          <Title order={4}>{getNodeTypeLabel(nodeType)} Configuration</Title>
          <Text size="sm" c="dimmed">({nodeId})</Text>
        </Group>
      }
      size="md"
      centered
    >
      <form onSubmit={form.onSubmit(handleSave)}>
        <Stack gap="md">
          {!isValid && validationErrors.length > 0 && (
            <Alert icon={<IconAlertCircle size="1rem" />} color="red" title="Configuration Error">
              <Stack gap="xs">
                {validationErrors.map((error, index) => (
                  <Text key={index} size="sm">{error}</Text>
                ))}
              </Stack>
            </Alert>
          )}

          {/* Basic metadata fields */}
          <TextInput
            label="Label"
            placeholder="Enter node label"
            required
            {...form.getInputProps('label')}
          />

          <Textarea
            label="Description"
            placeholder="Enter node description (optional)"
            rows={2}
            {...form.getInputProps('description')}
          />

          {/* Node-specific configuration fields */}
          {renderNodeSpecificFields(nodeType, form)}

          <Group justify="flex-end" mt="md">
            <Button variant="subtle" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" leftSection={<IconCheck size="1rem" />}>
              Save Configuration
            </Button>
          </Group>
        </Stack>
      </form>
    </Modal>
  )
}

// Get default config values for form initialization
function getConfigDefaults(nodeType: PlanDagNodeType, config: NodeConfig): Record<string, any> {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      const dataSourceConfig = config as any
      return {
        inputType: dataSourceConfig.inputType || 'CSVNodesFromFile',
        source: dataSourceConfig.source || '',
        dataType: dataSourceConfig.dataType || 'Nodes',
        outputGraphRef: dataSourceConfig.outputGraphRef || '',
      }

    case PlanDagNodeType.GRAPH:
      const graphConfig = config as any
      return {
        graphId: graphConfig.graphId || '',
        graphSource: graphConfig.graphSource || 'create',
      }

    case PlanDagNodeType.TRANSFORM:
      const transformConfig = config as any
      return {
        inputGraphRef: transformConfig.inputGraphRef || '',
        outputGraphRef: transformConfig.outputGraphRef || '',
        transformType: transformConfig.transformType || 'FilterNodes',
        transformConfig: JSON.stringify(transformConfig.transformConfig || {}, null, 2),
      }

    case PlanDagNodeType.MERGE:
      const mergeConfig = config as any
      return {
        inputGraphRefs: (mergeConfig.inputGraphRefs || []).join(', '),
        outputGraphRef: mergeConfig.outputGraphRef || '',
        mergeStrategy: mergeConfig.mergeStrategy || 'union',
      }

    case PlanDagNodeType.COPY:
      const copyConfig = config as any
      return {
        sourceGraphRef: copyConfig.sourceGraphRef || '',
        outputGraphRef: copyConfig.outputGraphRef || '',
        copyType: copyConfig.copyType || 'shallow',
      }

    case PlanDagNodeType.OUTPUT:
      const outputConfig = config as any
      return {
        sourceGraphRef: outputConfig.sourceGraphRef || '',
        renderTarget: outputConfig.renderTarget || 'DOT',
        outputPath: outputConfig.outputPath || '',
        renderConfig: JSON.stringify(outputConfig.renderConfig || {}, null, 2),
      }

    default:
      return {}
  }
}

// Render node-specific configuration fields
function renderNodeSpecificFields(nodeType: PlanDagNodeType, form: any) {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return (
        <>
          <Select
            label="Input Type"
            data={[
              { value: 'CSVNodesFromFile', label: 'CSV Nodes from File' },
              { value: 'CSVEdgesFromFile', label: 'CSV Edges from File' },
              { value: 'GraphMLFromFile', label: 'GraphML from File' },
              { value: 'JSONGraphFromFile', label: 'JSON Graph from File' },
            ]}
            {...form.getInputProps('inputType')}
          />
          <TextInput
            label="Source Path"
            placeholder="e.g., import/nodes.csv"
            {...form.getInputProps('source')}
          />
          <Select
            label="Data Type"
            data={[
              { value: 'Nodes', label: 'Nodes' },
              { value: 'Edges', label: 'Edges' },
              { value: 'Graph', label: 'Complete Graph' },
            ]}
            {...form.getInputProps('dataType')}
          />
          <TextInput
            label="Output Graph Reference"
            placeholder="e.g., graph_main"
            {...form.getInputProps('outputGraphRef')}
          />
        </>
      )

    case PlanDagNodeType.GRAPH:
      return (
        <>
          <TextInput
            label="Graph ID"
            placeholder="e.g., main_graph"
            {...form.getInputProps('graphId')}
          />
          <Select
            label="Graph Source"
            data={[
              { value: 'create', label: 'Create New' },
              { value: 'reference', label: 'Reference Existing' },
            ]}
            {...form.getInputProps('graphSource')}
          />
        </>
      )

    case PlanDagNodeType.TRANSFORM:
      return (
        <>
          <TextInput
            label="Input Graph Reference"
            placeholder="e.g., graph_main"
            {...form.getInputProps('inputGraphRef')}
          />
          <TextInput
            label="Output Graph Reference"
            placeholder="e.g., graph_filtered"
            {...form.getInputProps('outputGraphRef')}
          />
          <Select
            label="Transform Type"
            data={[
              { value: 'FilterNodes', label: 'Filter Nodes' },
              { value: 'FilterEdges', label: 'Filter Edges' },
              { value: 'TransformNodes', label: 'Transform Nodes' },
              { value: 'TransformEdges', label: 'Transform Edges' },
              { value: 'AddNodes', label: 'Add Nodes' },
              { value: 'RemoveNodes', label: 'Remove Nodes' },
            ]}
            {...form.getInputProps('transformType')}
          />
          <Textarea
            label="Transform Configuration (JSON)"
            placeholder='{"nodeFilter": "type = \\"important\\""}'
            rows={4}
            {...form.getInputProps('transformConfig')}
          />
        </>
      )

    case PlanDagNodeType.MERGE:
      return (
        <>
          <TextInput
            label="Input Graph References"
            placeholder="graph_1, graph_2, graph_3"
            description="Comma-separated list of graph references to merge"
            {...form.getInputProps('inputGraphRefs')}
          />
          <TextInput
            label="Output Graph Reference"
            placeholder="e.g., graph_merged"
            {...form.getInputProps('outputGraphRef')}
          />
          <Select
            label="Merge Strategy"
            data={[
              { value: 'union', label: 'Union (combine all)' },
              { value: 'intersection', label: 'Intersection (common only)' },
              { value: 'override', label: 'Override (later graphs win)' },
            ]}
            {...form.getInputProps('mergeStrategy')}
          />
        </>
      )

    case PlanDagNodeType.COPY:
      return (
        <>
          <TextInput
            label="Source Graph Reference"
            placeholder="e.g., graph_source"
            {...form.getInputProps('sourceGraphRef')}
          />
          <TextInput
            label="Output Graph Reference"
            placeholder="e.g., graph_copy"
            {...form.getInputProps('outputGraphRef')}
          />
          <Select
            label="Copy Type"
            data={[
              { value: 'shallow', label: 'Shallow Copy' },
              { value: 'deep', label: 'Deep Copy' },
              { value: 'reference', label: 'Reference Only' },
            ]}
            {...form.getInputProps('copyType')}
          />
        </>
      )

    case PlanDagNodeType.OUTPUT:
      return (
        <>
          <TextInput
            label="Source Graph Reference"
            placeholder="e.g., graph_final"
            {...form.getInputProps('sourceGraphRef')}
          />
          <Select
            label="Render Target"
            data={[
              { value: 'DOT', label: 'Graphviz DOT' },
              { value: 'GraphML', label: 'GraphML' },
              { value: 'JSON', label: 'JSON' },
              { value: 'CSV', label: 'CSV' },
              { value: 'PNG', label: 'PNG Image' },
              { value: 'SVG', label: 'SVG' },
            ]}
            {...form.getInputProps('renderTarget')}
          />
          <TextInput
            label="Output Path"
            placeholder="e.g., output/result.dot"
            {...form.getInputProps('outputPath')}
          />
          <Textarea
            label="Render Configuration (JSON)"
            placeholder='{"containNodes": true, "orientation": "TB"}'
            rows={4}
            {...form.getInputProps('renderConfig')}
          />
        </>
      )

    default:
      return null
  }
}

// Build config object from form values
function buildConfigFromForm(nodeType: PlanDagNodeType, values: FormData): NodeConfig {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      return {
        inputType: values.inputType,
        source: values.source,
        dataType: values.dataType,
        outputGraphRef: values.outputGraphRef,
      }

    case PlanDagNodeType.GRAPH:
      return {
        graphId: values.graphId ? parseInt(values.graphId) : 0,
        isReference: values.isReference || false,
        metadata: {
          nodeCount: values.nodeCount ? parseInt(values.nodeCount) : undefined,
          edgeCount: values.edgeCount ? parseInt(values.edgeCount) : undefined,
        },
      }

    case PlanDagNodeType.TRANSFORM:
      let transformConfig
      try {
        transformConfig = JSON.parse(values.transformConfig || '{}')
      } catch {
        transformConfig = {}
      }
      return {
        inputGraphRef: values.inputGraphRef,
        outputGraphRef: values.outputGraphRef,
        transformType: values.transformType,
        transformConfig,
      }

    case PlanDagNodeType.MERGE:
      return {
        inputRefs: values.inputGraphRefs.split(',').map((ref: string) => ref.trim()).filter(Boolean),
        outputGraphRef: values.outputGraphRef,
        mergeStrategy: values.mergeStrategy,
        conflictResolution: values.conflictResolution || 'PreferFirst',
      }

    case PlanDagNodeType.COPY:
      return {
        sourceGraphRef: values.sourceGraphRef,
        outputGraphRef: values.outputGraphRef,
        copyType: values.copyType,
        preserveMetadata: values.preserveMetadata !== undefined ? values.preserveMetadata : true,
      }

    case PlanDagNodeType.OUTPUT:
      let renderConfig
      try {
        renderConfig = JSON.parse(values.renderConfig || '{}')
      } catch {
        renderConfig = {}
      }
      return {
        sourceGraphRef: values.sourceGraphRef,
        renderTarget: values.renderTarget,
        outputPath: values.outputPath,
        renderConfig,
      }

    default:
      return {} as NodeConfig
  }
}

// Validate node configuration
function validateNodeConfig(nodeType: PlanDagNodeType, values: FormData): string[] {
  const errors: string[] = []

  if (!values.label) {
    errors.push('Label is required')
  }

  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      if (!values.source) errors.push('Source path is required')
      if (!values.outputGraphRef) errors.push('Output graph reference is required')
      break

    case PlanDagNodeType.GRAPH:
      if (!values.graphId) errors.push('Graph ID is required')
      break

    case PlanDagNodeType.TRANSFORM:
      if (!values.inputGraphRef) errors.push('Input graph reference is required')
      if (!values.outputGraphRef) errors.push('Output graph reference is required')
      try {
        JSON.parse(values.transformConfig || '{}')
      } catch {
        errors.push('Transform configuration must be valid JSON')
      }
      break

    case PlanDagNodeType.MERGE:
      if (!values.inputGraphRefs) errors.push('Input graph references are required')
      if (!values.outputGraphRef) errors.push('Output graph reference is required')
      const refs = values.inputGraphRefs.split(',').map((ref: string) => ref.trim()).filter(Boolean)
      if (refs.length < 2) errors.push('At least 2 input graph references are required for merge')
      break

    case PlanDagNodeType.COPY:
      if (!values.sourceGraphRef) errors.push('Source graph reference is required')
      if (!values.outputGraphRef) errors.push('Output graph reference is required')
      break

    case PlanDagNodeType.OUTPUT:
      if (!values.sourceGraphRef) errors.push('Source graph reference is required')
      if (!values.outputPath) errors.push('Output path is required')
      try {
        JSON.parse(values.renderConfig || '{}')
      } catch {
        errors.push('Render configuration must be valid JSON')
      }
      break
  }

  return errors
}