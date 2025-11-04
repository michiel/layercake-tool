import { useEffect, useMemo, useRef, useState } from 'react'
import {
  Alert,
  Badge,
  Button,
  Card,
  Checkbox,
  Divider,
  Group,
  ScrollArea,
  Stack,
  Text,
  TextInput,
  Title,
  Tooltip,
} from '@mantine/core'
import { IconAlertCircle, IconDownload, IconPlayerPlay, IconPlayerStop, IconTrash } from '@tabler/icons-react'
import {
  AppLogEntry,
  AppLogLevel,
  clearLogBuffer,
  getLogEntries,
  isTauriRuntime,
  subscribeToLogStream,
} from '../services/tauriLogStream'
import PageContainer from '../components/layout/PageContainer'

const LOG_LEVELS: AppLogLevel[] = ['TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR']

const levelColors: Record<AppLogLevel, string> = {
  TRACE: 'gray',
  DEBUG: 'blue',
  INFO: 'green',
  WARN: 'yellow',
  ERROR: 'red',
}

const formatTimestamp = (date: Date) =>
  `${date.toLocaleDateString()} ${date.toLocaleTimeString([], { hour12: false })}`

const downloadLogs = (entries: AppLogEntry[]) => {
  const lines = entries.map((entry) => {
    const ts = formatTimestamp(entry.timestamp)
    const target = entry.target ? ` ${entry.target}` : ''
    return `[${ts}] [${entry.level}]${target} ${entry.message}`
  })
  const file = new Blob([lines.join('\n')], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(file)
  const link = document.createElement('a')
  link.href = url
  link.download = `layercake-app-logs-${new Date().toISOString()}.log`
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

export const AppLogsPage: React.FC = () => {
  const [logs, setLogs] = useState<AppLogEntry[]>(() => getLogEntries())
  const [search, setSearch] = useState('')
  const [autoScroll, setAutoScroll] = useState(true)
  const [activeLevels, setActiveLevels] = useState<Set<AppLogLevel>>(new Set(LOG_LEVELS))
  const scrollViewportRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!isTauriRuntime()) {
      return
    }

    return subscribeToLogStream((entries) => {
      setLogs(entries)
    })
  }, [])

  useEffect(() => {
    if (!autoScroll) {
      return
    }
    const viewport = scrollViewportRef.current
    if (viewport) {
      viewport.scrollTo({ top: viewport.scrollHeight })
    }
  }, [logs, autoScroll])

  const filteredLogs = useMemo(() => {
    const needle = search.trim().toLowerCase()
    return logs.filter((entry) => {
      if (!activeLevels.has(entry.level)) {
        return false
      }
      if (!needle) {
        return true
      }
      return (
        entry.message.toLowerCase().includes(needle) ||
        (entry.target?.toLowerCase().includes(needle) ?? false)
      )
    })
  }, [logs, activeLevels, search])

  const toggleLevel = (level: AppLogLevel, checked: boolean) => {
    setActiveLevels((prev) => {
      const next = new Set(prev)
      if (checked) {
        next.add(level)
      } else {
        next.delete(level)
      }
      if (next.size === 0) {
        return new Set(LOG_LEVELS)
      }
      return next
    })
  }

  const isTauri = isTauriRuntime()

  return (
    <PageContainer>
      <Stack gap="lg" style={{ height: '100%' }}>
        <Group justify="space-between" align="flex-start">
          <div>
            <Title order={2}>Application Logs</Title>
            <Text c="dimmed" size="sm" mt="xs">
              Live stream of the embedded server and desktop shell logs. Use filters to focus on the
              information you need.
            </Text>
          </div>
          <Group gap="sm">
            <Tooltip label={autoScroll ? 'Stop auto scrolling' : 'Resume auto scrolling'}>
              <Button
                variant="light"
                leftSection={autoScroll ? <IconPlayerStop size={16} /> : <IconPlayerPlay size={16} />}
                onClick={() => setAutoScroll((value) => !value)}
                disabled={!isTauri}
              >
                {autoScroll ? 'Auto-scroll on' : 'Auto-scroll off'}
              </Button>
            </Tooltip>
            <Tooltip label="Clear current logs">
              <Button
                variant="light"
                color="red"
                leftSection={<IconTrash size={16} />}
                onClick={() => clearLogBuffer()}
                disabled={!isTauri || logs.length === 0}
              >
                Clear
              </Button>
            </Tooltip>
            <Tooltip label="Download logs as .log file">
              <Button
                variant="light"
                leftSection={<IconDownload size={16} />}
                onClick={() => downloadLogs(filteredLogs)}
                disabled={!isTauri || filteredLogs.length === 0}
              >
                Download
              </Button>
            </Tooltip>
          </Group>
        </Group>

        {!isTauri && (
          <Alert icon={<IconAlertCircle size={16} />} color="yellow" title="Desktop only">
            The application log stream is only available in the Tauri desktop build.
          </Alert>
        )}

        <Card withBorder padding="md" radius="md" style={{ flex: 1, display: 'flex' }}>
          <Stack gap="md" style={{ flex: 1 }}>
            <Group justify="space-between" align="flex-end" wrap="wrap">
              <Group gap="sm">
                {LOG_LEVELS.map((level) => (
                  <Checkbox
                    key={level}
                    label={level}
                    color={levelColors[level]}
                    checked={activeLevels.has(level)}
                    onChange={(event) => toggleLevel(level, event.currentTarget.checked)}
                    disabled={!isTauri}
                  />
                ))}
              </Group>
              <TextInput
                placeholder="Search logs"
                value={search}
                onChange={(event) => setSearch(event.currentTarget.value)}
                style={{ minWidth: 220 }}
                disabled={!isTauri}
              />
            </Group>

            <Divider />

            <ScrollArea.Autosize mah={600} style={{ flex: 1 }} viewportRef={scrollViewportRef} offsetScrollbars>
              <Stack gap={4} pb="sm">
                {filteredLogs.length === 0 ? (
                  <Text c="dimmed" ta="center" py="xl">
                    {isTauri ? 'No log entries match your filter.' : 'Logs are unavailable outside Tauri.'}
                  </Text>
                ) : (
                  filteredLogs.map((entry, index) => (
                    <LogRow entry={entry} key={`${entry.timestamp.getTime()}-${index}`} />
                  ))
                )}
              </Stack>
            </ScrollArea.Autosize>
          </Stack>
        </Card>
      </Stack>
    </PageContainer>
  )
}

const LogRow = ({ entry }: { entry: AppLogEntry }) => {
  return (
    <Group gap="sm" align="flex-start" wrap="nowrap">
      <Text size="xs" c="dimmed" style={{ width: 155, flexShrink: 0 }}>
        {formatTimestamp(entry.timestamp)}
      </Text>
      <Badge color={levelColors[entry.level]} variant="light" size="sm" style={{ flexShrink: 0 }}>
        {entry.level}
      </Badge>
      <Stack gap={2} style={{ flex: 1 }}>
        <Text size="sm" style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
          {entry.message}
        </Text>
        {entry.target && (
          <Text size="xs" c="dimmed">
            {entry.target}
          </Text>
        )}
      </Stack>
    </Group>
  )
}

export default AppLogsPage
