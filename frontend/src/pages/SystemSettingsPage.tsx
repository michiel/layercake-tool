import React, { useMemo, useState } from 'react'
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
import { Stack, Group } from '../components/layout-primitives'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '../components/ui/accordion'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent } from '../components/ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog'
import { Input } from '../components/ui/input'
import { Label } from '../components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
import { Skeleton } from '../components/ui/skeleton'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table'
import { Textarea } from '../components/ui/textarea'

export const SystemSettingsPage: React.FC = () => {
  const { data, loading, error, refetch } = useQuery<GetSystemSettingsResponse>(GET_SYSTEM_SETTINGS)
  const [updateSetting, { loading: saving }] = useMutation(UPDATE_SYSTEM_SETTING)
  const [selectedSetting, setSelectedSetting] = useState<SystemSetting | null>(null)
  const [value, setValue] = useState('')
  const [formError, setFormError] = useState<string | null>(null)

  const settingsByCategory = useMemo(() => {
    if (!data?.systemSettings) return new Map<string, SystemSetting[]>()

    const grouped = new Map<string, SystemSetting[]>()
    for (const setting of data.systemSettings) {
      const category = setting.category
      if (!grouped.has(category)) {
        grouped.set(category, [])
      }
      grouped.get(category)!.push(setting)
    }

    // Sort settings within each category
    for (const settings of grouped.values()) {
      settings.sort((a, b) => a.label.localeCompare(b.label))
    }

    return grouped
  }, [data])

  const providerCategories = ['OpenAI', 'Anthropic', 'Google Gemini', 'Ollama']
  const topLevelCategories = [
    { key: 'providers', label: 'Providers', categories: providerCategories },
    { key: 'chat', label: 'Chat', categories: ['Chat'] },
    { key: 'data-acquisition', label: 'Data Acquisition', categories: ['Data Acquisition'] },
    { key: 'rag', label: 'RAG', categories: ['RAG'] },
  ]

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
      if (Number.isNaN(Number(value)) || !Number.isInteger(Number(value))) {
        setFormError('Enter a whole number')
        return false
      }
    }
    if (selectedSetting.valueType === 'Float' && value.trim() !== '') {
      if (Number.isNaN(Number(value))) {
        setFormError('Enter a decimal number')
        return false
      }
    }
    if (selectedSetting.valueType === 'Boolean' && value.trim() !== '') {
      if (value !== 'true' && value !== 'false') {
        setFormError('Value must be "true" or "false"')
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

  const renderSettingsTable = (settings: SystemSetting[]) => {
    if (settings.length === 0) return null

    return (
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Setting</TableHead>
            <TableHead>Value</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {settings.map((setting) => (
            <TableRow key={setting.key}>
              <TableCell>
                <div className="font-semibold">{setting.label}</div>
                <div className="text-xs text-muted-foreground">
                  {setting.key}
                </div>
                {setting.isSecret && (
                  <Badge variant="secondary" className="mt-1">
                    Secret
                  </Badge>
                )}
              </TableCell>
              <TableCell>
                <div>
                  {setting.isSecret
                    ? '••••••'
                    : setting.value && setting.value.trim() !== ''
                      ? setting.value
                      : 'Not set'}
                </div>
                <div className="text-xs text-muted-foreground">
                  {formatValueType(setting.valueType)}
                </div>
              </TableCell>
              <TableCell>
                <Group gap="xs">
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => openEditor(setting)}
                    disabled={setting.isReadOnly}
                  >
                    Edit
                  </Button>
                  {setting.isReadOnly && (
                    <Badge variant="outline">
                      Read-only
                    </Badge>
                  )}
                </Group>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    )
  }

  const renderValueEditor = () => {
    if (!selectedSetting) return null
    const disabled = selectedSetting.isReadOnly

    switch (selectedSetting.valueType) {
      case 'Integer':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Input
              id="setting-value"
              type="number"
              step="1"
              placeholder="Enter a whole number"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              disabled={disabled}
            />
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
      case 'Float':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Input
              id="setting-value"
              type="number"
              step="0.01"
              placeholder="Enter a decimal number"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              disabled={disabled}
            />
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
      case 'Boolean':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Select
              value={value || undefined}
              onValueChange={(val) => setValue(val ?? '')}
              disabled={disabled}
            >
              <SelectTrigger id="setting-value">
                <SelectValue placeholder="Select true or false" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="true">true</SelectItem>
                <SelectItem value="false">false</SelectItem>
              </SelectContent>
            </Select>
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
      case 'Enum':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Select
              value={value || undefined}
              onValueChange={(val) => setValue(val ?? '')}
              disabled={disabled}
            >
              <SelectTrigger id="setting-value">
                <SelectValue placeholder={selectedSetting.allowedValues.length ? 'Select a value' : 'No options'} />
              </SelectTrigger>
              <SelectContent>
                {selectedSetting.allowedValues.map((option) => (
                  <SelectItem key={option} value={option}>
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
      case 'Text':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Textarea
              id="setting-value"
              rows={4}
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
              disabled={disabled}
            />
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
      case 'Secret':
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Input
              id="setting-value"
              type="password"
              placeholder="Enter a new value"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
              disabled={disabled}
            />
            <p className="text-sm text-muted-foreground">Leave blank to clear the stored value</p>
          </div>
        )
      default:
        return (
          <div className="space-y-2">
            <Label htmlFor="setting-value">Value</Label>
            <Input
              id="setting-value"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
              disabled={disabled}
            />
            {selectedSetting.description && (
              <p className="text-sm text-muted-foreground">{selectedSetting.description}</p>
            )}
          </div>
        )
    }
  }

  return (
    <PageContainer>
      <Stack gap="lg">
        <Group justify="between" align="start">
          <div>
            <h2 className="text-2xl font-bold">System Settings</h2>
            <p className="text-sm text-muted-foreground mt-1">
              Inspect and update runtime configuration values without restarting the backend.
            </p>
          </div>
          <Button variant="secondary" onClick={() => refetch()} disabled={loading}>
            <IconSettings className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </Group>

        {error && (
          <Alert variant="destructive">
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>Failed to load settings</AlertTitle>
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        )}

        <Card className="border">
          <CardContent className="pt-6">
            {loading ? (
              <Stack gap="sm">
                {Array.from({ length: 4 }).map((_, index) => (
                  <Skeleton key={index} className="h-12 rounded-sm" />
                ))}
              </Stack>
            ) : (
              <Accordion type="multiple" className="w-full">
                {topLevelCategories.map((topLevel) => {
                  const hasSettings = topLevel.categories.some((cat) => settingsByCategory.has(cat))
                  if (!hasSettings) return null

                  return (
                    <AccordionItem key={topLevel.key} value={topLevel.key}>
                      <AccordionTrigger className="text-lg font-semibold">
                        {topLevel.label}
                      </AccordionTrigger>
                      <AccordionContent>
                        {topLevel.key === 'providers' ? (
                          <Accordion type="multiple" className="w-full pl-4">
                            {providerCategories.map((provider) => {
                              const providerSettings = settingsByCategory.get(provider)
                              if (!providerSettings || providerSettings.length === 0) return null

                              return (
                                <AccordionItem key={provider} value={provider}>
                                  <AccordionTrigger className="text-base">
                                    {provider}
                                  </AccordionTrigger>
                                  <AccordionContent>
                                    {renderSettingsTable(providerSettings)}
                                  </AccordionContent>
                                </AccordionItem>
                              )
                            })}
                          </Accordion>
                        ) : (
                          topLevel.categories.map((category) => {
                            const categorySettings = settingsByCategory.get(category)
                            if (!categorySettings || categorySettings.length === 0) return null
                            return (
                              <div key={category}>
                                {renderSettingsTable(categorySettings)}
                              </div>
                            )
                          })
                        )}
                      </AccordionContent>
                    </AccordionItem>
                  )
                })}
              </Accordion>
            )}
          </CardContent>
        </Card>
      </Stack>

      <Dialog open={Boolean(selectedSetting)} onOpenChange={(open) => !open && closeEditor()}>
        <DialogContent className="sm:max-w-[600px]">
          {selectedSetting && (
            <>
              <DialogHeader>
                <DialogTitle>{selectedSetting.label}</DialogTitle>
              </DialogHeader>
              <Stack gap="md">
                <p className="text-sm text-muted-foreground">
                  {selectedSetting.description || 'Update the current value for this setting.'}
                </p>
                {renderValueEditor()}
                {formError && (
                  <Alert variant="destructive">
                    <IconAlertCircle className="h-4 w-4" />
                    <AlertDescription>{formError}</AlertDescription>
                  </Alert>
                )}
              </Stack>
              <DialogFooter>
                <Button variant="outline" onClick={closeEditor}>
                  Cancel
                </Button>
                <Button onClick={handleSubmit} disabled={saving}>
                  {saving ? 'Saving...' : 'Save'}
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
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
      return 'Integer'
    case 'Float':
      return 'Decimal'
    case 'Boolean':
      return 'Boolean'
    case 'Enum':
      return 'Choice'
    case 'Secret':
      return 'Secret'
    default:
      return type
  }
}
