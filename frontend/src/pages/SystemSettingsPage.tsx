import React, { useMemo, useState } from 'react'
import {
  Badge,
  Button,
  Card,
  Group,
  Modal,
  NumberInput,
  PasswordInput,
  Select,
  Skeleton,
  Stack,
  Table,
  Text,
  Textarea,
  TextInput,
  Title,
  Alert,
} from '@mantine/core'
import { IconAlertCircle, IconSettings } from '@tabler/icons-react'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  GET_SYSTEM_SETTINGS,
  SystemSetting,
  SystemSettingValueType,
  UPDATE_SYSTEM_SETTING,
  GetSystemSettingsResponse,
} from '../graphql/systemSettings'
import { showErrorNotification, showSuccessNotification } from '../utils/notifications'
import PageContainer from '../components/layout/PageContainer'

export const SystemSettingsPage: React.FC = () => {
  const { data, loading, error, refetch } = useQuery<GetSystemSettingsResponse>(GET_SYSTEM_SETTINGS)
  const [updateSetting, { loading: saving }] = useMutation(UPDATE_SYSTEM_SETTING)
  const [selectedSetting, setSelectedSetting] = useState<SystemSetting | null>(null)
  const [value, setValue] = useState('')
  const [formError, setFormError] = useState<string | null>(null)

  const settings = useMemo(() => {
    if (!data?.systemSettings) return []
    return [...data.systemSettings].sort((a, b) => {
      if (a.category === b.category) {
        return a.label.localeCompare(b.label)
      }
      return a.category.localeCompare(b.category)
    })
  }, [data])

  const openEditor = (setting: SystemSetting) => {
    setSelectedSetting(setting)
    setValue(setting.value ?? '')
    setFormError(null)
  }

  const closeEditor = () => {
    setSelectedSetting(null)
    setValue('')
    setFormError(null)
  }

  const validateValue = (): boolean => {
    if (!selectedSetting) return false
    if (selectedSetting.valueType === 'Integer' && value.trim() !== '') {
      if (Number.isNaN(Number(value))) {
        setFormError('Enter a numeric value')
        return false
      }
    }
    setFormError(null)
    return true
  }

  const handleSubmit = async () => {
    if (!selectedSetting) return
    if (!validateValue()) return

    try {
      await updateSetting({
        variables: {
          input: {
            key: selectedSetting.key,
            value: value ?? '',
          },
        },
      })
      showSuccessNotification('Setting updated', `${selectedSetting.label} saved`)
      closeEditor()
      await refetch()
    } catch (mutationError) {
      showErrorNotification('Failed to update setting', (mutationError as Error).message)
    }
  }

  const renderValueEditor = () => {
    if (!selectedSetting) return null
    const common = {
      label: 'Value',
      description: selectedSetting.description ?? undefined,
      disabled: selectedSetting.isReadOnly,
    }

    switch (selectedSetting.valueType) {
      case 'Integer':
        return (
          <NumberInput
            {...common}
            placeholder="Enter a number"
            value={value === '' ? undefined : Number(value)}
            onChange={(val) => {
              if (typeof val === 'number') {
                setValue(String(val))
              } else if (typeof val === 'string') {
                setValue(val)
              } else {
                setValue('')
              }
            }}
          />
        )
      case 'Enum':
        return (
          <Select
            {...common}
            data={selectedSetting.allowedValues.map((option) => ({ value: option, label: option }))}
            value={value || null}
            onChange={(val) => setValue(val ?? '')}
            placeholder={selectedSetting.allowedValues.length ? 'Select a value' : 'No options'}
          />
        )
      case 'Text':
        return (
          <Textarea
            {...common}
            minRows={4}
            value={value}
            onChange={(event) => setValue(event.currentTarget.value)}
          />
        )
      case 'Secret':
        return (
          <PasswordInput
            {...common}
            placeholder="Enter a new value"
            value={value}
            onChange={(event) => setValue(event.currentTarget.value)}
            description="Leave blank to clear the stored value"
          />
        )
      default:
        return (
          <TextInput
            {...common}
            value={value}
            onChange={(event) => setValue(event.currentTarget.value)}
          />
        )
    }
  }

  return (
    <PageContainer>
      <Stack gap="lg">
        <Group justify="space-between" align="flex-start">
          <div>
            <Title order={2}>System Settings</Title>
            <Text c="dimmed" size="sm" mt="xs">
              Inspect and update runtime configuration values without restarting the backend.
            </Text>
          </div>
          <Button variant="light" leftSection={<IconSettings size={16} />} onClick={() => refetch()} disabled={loading}>
            Refresh
          </Button>
        </Group>

        {error && (
          <Alert icon={<IconAlertCircle size={16} />} color="red" title="Failed to load settings">
            {error.message}
          </Alert>
        )}

        <Card withBorder padding="lg" radius="md">
          {loading ? (
            <Stack gap="sm">
              {Array.from({ length: 4 }).map((_, index) => (
                <Skeleton key={index} height={48} radius="sm" />
              ))}
            </Stack>
          ) : (
            <Table highlightOnHover verticalSpacing="sm">
              <Table.Thead>
                <Table.Tr>
                  <Table.Th>Setting</Table.Th>
                  <Table.Th>Value</Table.Th>
                  <Table.Th>Category</Table.Th>
                  <Table.Th>Actions</Table.Th>
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {settings.map((setting) => (
                <Table.Tr key={setting.key}>
                    <Table.Td>
                      <Text fw={600}>{setting.label}</Text>
                      <Text size="xs" c="dimmed">
                        {setting.key}
                      </Text>
                      {setting.isSecret && (
                        <Badge color="gray" size="xs" mt={4}>
                          Secret
                        </Badge>
                      )}
                    </Table.Td>
                    <Table.Td>
                      <Text>
                        {setting.isSecret
                          ? '••••••'
                          : setting.value && setting.value.trim() !== ''
                            ? setting.value
                            : 'Not set'}
                      </Text>
                      <Text size="xs" c="dimmed">
                        {formatValueType(setting.valueType)}
                      </Text>
                    </Table.Td>
                    <Table.Td>
                      <Badge color="blue" variant="light">
                        {setting.category}
                      </Badge>
                    </Table.Td>
                    <Table.Td>
                      <Group gap="xs">
                        <Button
                          size="xs"
                          variant="light"
                          onClick={() => openEditor(setting)}
                          disabled={setting.isReadOnly}
                        >
                          Edit
                        </Button>
                        {setting.isReadOnly && (
                          <Badge color="gray" variant="outline" size="xs">
                            Read-only
                          </Badge>
                        )}
                      </Group>
                    </Table.Td>
                  </Table.Tr>
                ))}
                {settings.length === 0 && (
                  <Table.Tr>
                    <Table.Td colSpan={4}>
                      <Text c="dimmed" style={{ textAlign: 'center' }}>
                        No settings available.
                      </Text>
                    </Table.Td>
                  </Table.Tr>
                )}
              </Table.Tbody>
            </Table>
          )}
        </Card>
      </Stack>

      <Modal opened={Boolean(selectedSetting)} onClose={closeEditor} title={selectedSetting?.label} size="lg">
        {selectedSetting && (
          <Stack gap="md">
            <Text size="sm" c="dimmed">
              {selectedSetting.description || 'Update the current value for this setting.'}
            </Text>
            {renderValueEditor()}
            {formError && (
              <Alert icon={<IconAlertCircle size={16} />} color="red">
                {formError}
              </Alert>
            )}
            <Group justify="flex-end">
              <Button variant="default" onClick={closeEditor}>
                Cancel
              </Button>
              <Button onClick={handleSubmit} loading={saving}>
                Save
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>
    </PageContainer>
  )
}

const formatValueType = (type: SystemSettingValueType) => {
  switch (type) {
    case 'String':
      return 'String'
    case 'Text':
      return 'Text'
    case 'Url':
      return 'URL'
    case 'Integer':
      return 'Number'
    case 'Enum':
      return 'Choice'
    case 'Secret':
      return 'Secret'
    default:
      return type
  }
}
