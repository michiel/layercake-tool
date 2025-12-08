import React, { useMemo } from 'react'
import { Badge } from '../ui/badge'
import { Separator } from '../ui/separator'
import { Stack, Group } from '../layout-primitives'

type AnalysisResult = {
  functions?: any[]
  imports?: any[]
  data_flows?: any[]
  call_edges?: any[]
  entry_points?: any[]
  env_vars?: any[]
  files?: string[]
  directories?: string[]
  infra?: {
    resources?: Record<string, any>
    partitions?: Record<string, any>
    edges?: any[]
    diagnostics?: string[]
  }
  infra_correlation?: {
    matches?: any[]
    unresolved?: string[]
    warnings?: string[]
  }
}

type Props = {
  resultJson?: string | null
}

export const AnalysisResultViewer: React.FC<Props> = ({ resultJson }) => {
  const parsed = useMemo<AnalysisResult | null>(() => {
    if (!resultJson) return null
    try {
      return JSON.parse(resultJson)
    } catch (err) {
      return null
    }
  }, [resultJson])

  if (!resultJson) {
    return <div className="text-sm text-muted-foreground">No stored analysis result. Run analysis to populate.</div>
  }

  if (!parsed) {
    return <div className="text-sm text-red-600">Unable to parse analysis result JSON.</div>
  }

  const counts = {
    functions: parsed.functions?.length ?? 0,
    imports: parsed.imports?.length ?? 0,
    dataFlows: parsed.data_flows?.length ?? 0,
    controlFlows: parsed.call_edges?.length ?? 0,
    entries: parsed.entry_points?.length ?? 0,
    envs: parsed.env_vars?.length ?? 0,
    files: parsed.files?.length ?? 0,
    dirs: parsed.directories?.length ?? 0,
    infraResources: parsed.infra ? Object.keys(parsed.infra.resources ?? {}).length : 0,
    infraEdges: parsed.infra?.edges?.length ?? 0,
    matches: parsed.infra_correlation?.matches?.length ?? 0,
    unresolved: parsed.infra_correlation?.unresolved?.length ?? 0,
  }

  return (
    <Stack gap="md">
      <Group gap="sm" wrap>
        <Badge variant="outline">Functions: {counts.functions}</Badge>
        <Badge variant="outline">Data flow edges: {counts.dataFlows}</Badge>
        <Badge variant="outline">Control flow edges: {counts.controlFlows}</Badge>
        <Badge variant="outline">Imports: {counts.imports}</Badge>
        <Badge variant="outline">Entry points: {counts.entries}</Badge>
        <Badge variant="outline">Env vars: {counts.envs}</Badge>
        <Badge variant="outline">Files: {counts.files}</Badge>
        <Badge variant="outline">Directories: {counts.dirs}</Badge>
        <Badge variant="outline">Infra resources: {counts.infraResources}</Badge>
        <Badge variant="outline">Infra edges: {counts.infraEdges}</Badge>
        <Badge variant="outline">Infra matches: {counts.matches}</Badge>
        <Badge variant="outline">Infra unresolved: {counts.unresolved}</Badge>
      </Group>

      <Separator />

      <div className="space-y-3 text-sm">
        <Section title="Functions" items={parsed.functions} labelKey="name" fallback="No functions captured." />
        <Section title="Imports" items={parsed.imports} labelKey="module" fallback="No imports captured." />
        <Section title="Data flows" items={parsed.data_flows} labelKey="variable" fallback="No data flows captured." />
        <Section title="Control flows" items={parsed.call_edges} labelKey="callee" fallback="No control flows captured." />
        <Section title="Entry points" items={parsed.entry_points} labelKey="condition" fallback="No entry points captured." />
        <Section title="Env vars" items={parsed.env_vars} labelKey="name" fallback="No env vars captured." />
        <Section
          title="Infra resources"
          items={parsed.infra ? Object.values(parsed.infra.resources ?? {}) : []}
          labelKey="name"
          fallback="No infra resources detected."
        />
        <Section
          title="Infra partitions"
          items={parsed.infra ? Object.values(parsed.infra.partitions ?? {}) : []}
          labelKey="label"
          fallback="No infra partitions detected."
        />
        <Section
          title="Infra edges"
          items={parsed.infra?.edges}
          labelKey="edge_type"
          fallback="No infra edges detected."
        />
        <Section
          title="Infra diagnostics"
          items={parsed.infra?.diagnostics?.map((d) => ({ label: d })) ?? []}
          labelKey="label"
          fallback="No infra diagnostics."
        />
        <Section
          title="Infra correlation"
          items={parsed.infra_correlation?.matches}
          labelKey="reason"
          fallback="No infra correlations."
        />
        <Section
          title="Correlation unresolved"
          items={parsed.infra_correlation?.unresolved?.map((d) => ({ label: d })) ?? []}
          labelKey="label"
          fallback="No unresolved items."
        />
        <Section
          title="Correlation warnings"
          items={parsed.infra_correlation?.warnings?.map((d) => ({ label: d })) ?? []}
          labelKey="label"
          fallback="No correlation warnings."
        />
      </div>

      <Separator />
      <div>
        <div className="text-xs text-muted-foreground mb-1">Raw JSON</div>
        <pre className="bg-muted text-xs p-3 rounded-md overflow-x-auto">{JSON.stringify(parsed, null, 2)}</pre>
      </div>
    </Stack>
  )
}

type SectionProps = {
  title: string
  items?: any[]
  labelKey: string
  fallback?: string
}

const Section: React.FC<SectionProps> = ({ title, items, labelKey, fallback }) => {
  const list = items ?? []
  return (
    <div>
      <div className="font-medium">{title}</div>
      {list.length === 0 ? (
        <div className="text-muted-foreground text-xs">{fallback ?? 'None'}</div>
      ) : (
        <ul className="list-disc pl-4 text-xs space-y-1">
          {list.slice(0, 10).map((item, idx) => (
            <li key={idx}>
              {typeof item === 'string'
                ? item
                : item[labelKey] ?? item.label ?? item.name ?? JSON.stringify(item)}
              {item.file_path ? <span className="text-muted-foreground ml-1">({item.file_path})</span> : null}
              {item.confidence !== undefined ? (
                <span className="text-muted-foreground ml-1">[{item.confidence}%]</span>
              ) : null}
            </li>
          ))}
          {list.length > 10 && <li className="text-muted-foreground">… {list.length - 10} more</li>}
        </ul>
      )}
    </div>
  )
}
