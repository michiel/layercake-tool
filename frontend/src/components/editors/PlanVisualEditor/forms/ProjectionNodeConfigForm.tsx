import { useEffect, useState } from 'react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { ProjectionNodeConfig } from '../../../../types/plan-dag'
import { Stack } from '@/components/layout-primitives'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const LIST_PROJECTIONS = gql`
  query ListProjections($projectId: ID!) {
    projections(projectId: $projectId) {
      id
      name
      projectionType
      graphId
    }
  }
`

interface ProjectionNodeConfigFormProps {
  config: ProjectionNodeConfig
  setConfig: (config: ProjectionNodeConfig) => void
  setIsValid: (isValid: boolean) => void
  projectId: number
}

export const ProjectionNodeConfigForm: React.FC<ProjectionNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<ProjectionNodeConfig>({
    projectionId: config.projectionId,
  })

  const { data: projectionsData, loading: projectionsLoading } = useQuery(LIST_PROJECTIONS, {
    variables: { projectId: projectId.toString() },
    skip: !projectId,
  })

  const projections = (projectionsData as any)?.projections ?? []

  useEffect(() => {
    setConfig(localConfig)
  }, [localConfig, setConfig])

  useEffect(() => {
    setIsValid(!!localConfig.projectionId)
  }, [localConfig, setIsValid])

  return (
    <Stack gap="md">
      <div className="space-y-2">
        <Label htmlFor="projection-select">Select Projection</Label>
        <Select
          value={localConfig.projectionId?.toString() || ''}
          onValueChange={(value) =>
            setLocalConfig((prev) => ({
              ...prev,
              projectionId: parseInt(value, 10),
            }))
          }
          disabled={projectionsLoading}
        >
          <SelectTrigger id="projection-select">
            <SelectValue placeholder="Choose a projection..." />
          </SelectTrigger>
          <SelectContent>
            {projectionsLoading ? (
              <SelectItem value="loading" disabled>
                Loading projections...
              </SelectItem>
            ) : projections.length === 0 ? (
              <SelectItem value="none" disabled>
                No projections found
              </SelectItem>
            ) : (
              projections.map((projection: any) => (
                <SelectItem key={projection.id} value={projection.id.toString()}>
                  {projection.name} (#{projection.id}) - {projection.projectionType}
                </SelectItem>
              ))
            )}
          </SelectContent>
        </Select>
        <p className="text-sm text-muted-foreground">
          Select a projection to visualise from the connected graph
        </p>
      </div>
    </Stack>
  )
}
