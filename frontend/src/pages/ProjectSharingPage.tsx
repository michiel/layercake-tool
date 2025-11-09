import React from 'react'
import { gql } from '@apollo/client'
import { useQuery } from '@apollo/client/react'
import { useNavigate, useParams } from 'react-router-dom'
import { IconShare, IconLink, IconUsersGroup } from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Stack, Group } from '../components/layout-primitives'
import { Button } from '../components/ui/button'
import { Input } from '../components/ui/input'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

export const ProjectSharingPage: React.FC = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectNumericId = projectId ? parseInt(projectId, 10) : NaN

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{ id: number; name: string }>
  }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find(
    (p: any) => p.id === projectNumericId,
  )

  if (projectsLoading && !selectedProject) {
    return (
      <PageContainer>
        <p>Loading project…</p>
      </PageContainer>
    )
  }

  if (!selectedProject) {
    return (
      <PageContainer>
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to Projects
        </Button>
      </PageContainer>
    )
  }

  const shareLink = `${window.location.origin}/projects/${selectedProject.id}`

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        sections={[{ title: 'Sharing' }]}
        currentPage="Project access"
        onNavigate={(path) => navigate(path)}
      />

      <Stack gap="lg">
        <Stack gap="xs">
          <h1 className="text-2xl font-bold">Sharing</h1>
          <p className="text-muted-foreground max-w-2xl">
            Invite collaborators, publish read-only links, and control how your project is distributed outside the team.
          </p>
        </Stack>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconShare className="h-5 w-5 text-primary" />
              Invite collaborators
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Generate a link that grants edit access to trusted teammates. You can revoke access from the project settings page.
            </p>
            <div className="flex flex-col md:flex-row gap-2">
              <Input readOnly value={shareLink} />
              <Button
                onClick={() => {
                  navigator.clipboard.writeText(shareLink)
                  navigate(`/projects/${selectedProject.id}`)
                }}
              >
                Copy link & view project
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconLink className="h-5 w-5 text-primary" />
              Publish read-only snapshot
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Export the latest plan, datasets, and generated graphs as a static package that external reviewers can inspect offline.
            </p>
            <Button variant="outline" onClick={() => navigate(`/projects/${selectedProject.id}/plan`)}>
              Review and export plan
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconUsersGroup className="h-5 w-5 text-primary" />
              Manage collaborators
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Keep track of who has access to this project and adjust permissions directly from the project overview screen.
            </p>
            <Group gap="sm">
              <Button variant="secondary" onClick={() => navigate(`/projects/${selectedProject.id}`)}>
                Open project overview
              </Button>
              <Button variant="ghost" onClick={() => navigate(`/projects/${selectedProject.id}/edit`)}>
                Edit project details
              </Button>
            </Group>
          </CardContent>
        </Card>
      </Stack>
    </PageContainer>
  )
}

export default ProjectSharingPage
