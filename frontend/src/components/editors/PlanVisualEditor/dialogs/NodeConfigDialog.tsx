import { useState, useEffect } from 'react'
import { Modal, Title, Button, Group, Stack, Text, TextInput, Select, Textarea, Alert, Badge, Card } from '@mantine/core'
import { useForm } from '@mantine/form'
import { IconAlertCircle, IconCheck, IconFile, IconPlus } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata, DataSourceNodeConfig } from '../../../../types/plan-dag'
import { DataSourceSelectionDialog } from './DataSourceSelectionDialog'
import { useQuery } from '@apollo/client/react'
import { GET_DATASOURCE, DataSource, getDataSourceTypeDisplayName } from '../../../../graphql/datasources'

interface NodeConfigDialogProps {
  opened: boolean
  onClose: () => void
  nodeId: string
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  projectId: number
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
  projectId,
  onSave
}: NodeConfigDialogProps) => {
  const [isValid, setIsValid] = useState(true)
  const [validationErrors, setValidationErrors] = useState<string[]>([])
  const [dataSourceDialogOpen, setDataSourceDialogOpen] = useState(false)
  const [selectedDataSource, setSelectedDataSource] = useState<DataSource | null>(null)

  // Query current DataSource if this is a DataSource node with a dataSourceId
  const currentDataSourceId = nodeType === PlanDagNodeType.DATA_SOURCE
    ? (config as DataSourceNodeConfig).dataSourceId
    : null

  const { data: dataSourceData } = useQuery(GET_DATASOURCE, {
    variables: { id: currentDataSourceId || 0 },
    skip: !currentDataSourceId,
    errorPolicy: 'ignore'
  })

  // Update selectedDataSource when dataSourceData changes
  useEffect(() => {
    if ((dataSourceData as any)?.dataSource) {
      setSelectedDataSource((dataSourceData as any).dataSource as DataSource)
    }
  }, [dataSourceData])

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
      case PlanDagNodeType.DATA_SOURCE: return 'Data Source'
      case PlanDagNodeType.GRAPH: return 'Graph'
      case PlanDagNodeType.TRANSFORM: return 'Transform'
      case PlanDagNodeType.MERGE: return 'Merge'
      case PlanDagNodeType.COPY: return 'Copy'
      case PlanDagNodeType.OUTPUT: return 'Output'
      default: return 'Unknown'
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
          {renderNodeSpecificFields(nodeType, form, {
            selectedDataSource,
            onSelectDataSource: () => setDataSourceDialogOpen(true)
          })}

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

      {/* DataSource Selection Dialog */}
      <DataSourceSelectionDialog
        opened={dataSourceDialogOpen}
        onClose={() => setDataSourceDialogOpen(false)}
        onSelect={(dataSource) => {
          setSelectedDataSource(dataSource)
          form.setFieldValue('dataSourceId', dataSource.id)
        }}
        currentDataSourceId={selectedDataSource?.id}
        projectId={projectId}
      />
    </Modal>
  )
}

// Get default config values for form initialization
function getConfigDefaults(nodeType: PlanDagNodeType, config: NodeConfig): Record<string, any> {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      const dataSourceConfig = config as DataSourceNodeConfig
      return {
        // New DataSource system
        dataSourceId: dataSourceConfig.dataSourceId || null,
        displayMode: dataSourceConfig.displayMode || 'summary',

        // Legacy support (backward compatibility)
        inputType: dataSourceConfig.inputType || 'CSVNodesFromFile',
        source: dataSourceConfig.source || '',
        dataType: dataSourceConfig.dataType || 'Nodes',
      }

    case PlanDagNodeType.GRAPH:
      const graphConfig = config as any
      return {
        graphSource: graphConfig.graphSource || 'create',
      }

    case PlanDagNodeType.TRANSFORM:
      const transformConfig = config as any
      return {
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
function renderNodeSpecificFields(
  nodeType: PlanDagNodeType,
  form: any,
  extraProps?: {
    selectedDataSource?: DataSource | null
    onSelectDataSource?: () => void
  }
) {
  switch (nodeType) {
    case PlanDagNodeType.DATA_SOURCE:
      const { selectedDataSource, onSelectDataSource } = extraProps || {}
      return (
        <>
          {/* New DataSource Selection */}
          <div>
            <Text size="sm" fw={500} mb="xs">Data Source</Text>
            {selectedDataSource ? (
              <Card withBorder p="sm" mb="sm">
                <Group justify="space-between" align="flex-start">
                  <div style={{ flex: 1 }}>
                    <Group gap="xs" mb="xs">
                      <IconFile size={16} />
                      <Text fw={500} size="sm">{selectedDataSource.name}</Text>
                      <Badge variant="outline" size="xs">
                        {getDataSourceTypeDisplayName(selectedDataSource.sourceType)}
                      </Badge>
                    </Group>

                    {selectedDataSource.description && (
                      <Text size="xs" c="dimmed" mb="xs">
                        {selectedDataSource.description}
                      </Text>
                    )}

                    <Text size="xs" c="dimmed" ff="monospace">
                      {selectedDataSource.filename}
                    </Text>
                  </div>

                  <Button
                    size="xs"
                    variant="light"
                    onClick={onSelectDataSource}
                  >
                    Change
                  </Button>
                </Group>
              </Card>
            ) : (
              <Button
                variant="light"
                fullWidth
                leftSection={<IconPlus size={16} />}
                onClick={onSelectDataSource}
                mb="sm"
              >
                Select Data Source
              </Button>
            )}
          </div>

          {/* Display Mode */}
          <Select
            label="Display Mode"
            data={[
              { value: 'summary', label: 'Summary' },
              { value: 'detailed', label: 'Detailed' },
              { value: 'preview', label: 'Preview' },
            ]}
            {...form.getInputProps('displayMode')}
          />

          {/* Output Graph Reference */}
          <TextInput
            label="Output Graph Reference"
            placeholder="e.g., imported_data"
            {...form.getInputProps('outputGraphRef')}
          />

          {/* Legacy Fields (for backward compatibility) */}
          {(!selectedDataSource && (form.values.source || form.values.inputType)) && (
            <Alert icon={<IconAlertCircle size={16} />} title="Legacy Configuration" color="orange" mb="md">
              <Text size="sm" mb="sm">
                This node uses legacy configuration. Consider selecting a DataSource for better integration.
              </Text>

              <Stack gap="sm">
                <Select
                  label="Input Type (Legacy)"
                  data={[
                    { value: 'CSVNodesFromFile', label: 'CSV Nodes from File' },
                    { value: 'CSVEdgesFromFile', label: 'CSV Edges from File' },
                    { value: 'CSVLayersFromFile', label: 'CSV Layers from File' },
                  ]}
                  {...form.getInputProps('inputType')}
                />
                <TextInput
                  label="Source Path (Legacy)"
                  placeholder="e.g., import/nodes.csv"
                  {...form.getInputProps('source')}
                />
                <Select
                  label="Data Type (Legacy)"
                  data={[
                    { value: 'Nodes', label: 'Nodes' },
                    { value: 'Edges', label: 'Edges' },
                    { value: 'Layers', label: 'Layers' },
                  ]}
                  {...form.getInputProps('dataType')}
                />
              </Stack>
            </Alert>
          )}
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
            label="Filename"
            placeholder="e.g., myproject.gml (optional)"
            description="Optional. If not specified, will use project name and file extension."
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
      const dataSourceConfig: DataSourceNodeConfig = {}

      // Add new DataSource system properties if available
      if (values.dataSourceId) {
        dataSourceConfig.dataSourceId = values.dataSourceId
        dataSourceConfig.displayMode = values.displayMode || 'summary'
      }

      // Add legacy properties for backward compatibility
      if (values.inputType || values.source) {
        dataSourceConfig.inputType = values.inputType
        dataSourceConfig.source = values.source
        dataSourceConfig.dataType = values.dataType
      }

      return dataSourceConfig

    case PlanDagNodeType.GRAPH:
      return {
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
        transformType: values.transformType,
        transformConfig,
      }

    case PlanDagNodeType.MERGE:
      return {
        mergeStrategy: values.mergeStrategy,
        conflictResolution: values.conflictResolution || 'PreferFirst',
      }

    case PlanDagNodeType.COPY:
      return {
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
      // Check if using new DataSource system or legacy system
      if (values.dataSourceId) {
        // New DataSource system validation
        if (!values.outputGraphRef) errors.push('Output graph reference is required')
      } else if (values.source || values.inputType) {
        // Legacy system validation
        if (!values.source) errors.push('Source path is required')
        if (!values.outputGraphRef) errors.push('Output graph reference is required')
      } else {
        // No configuration at all
        errors.push('Please select a DataSource or provide legacy source configuration')
      }
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
      // outputPath (filename) is optional
      try {
        JSON.parse(values.renderConfig || '{}')
      } catch {
        errors.push('Render configuration must be valid JSON')
      }
      break
  }

  return errors
}