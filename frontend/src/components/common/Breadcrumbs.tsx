import React from 'react'
import { Breadcrumbs as MantineBreadcrumbs, Anchor, Text } from '@mantine/core'
import { IconHome, IconDatabase, IconGraph, IconSettings } from '@tabler/icons-react'

interface BreadcrumbItem {
  title: string
  href?: string
  icon?: React.ReactNode
}

interface BreadcrumbsProps {
  projectName?: string
  projectId?: number
  currentPage?: string
  onNavigate?: (path: string) => void
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({
  projectName,
  projectId,
  currentPage,
  onNavigate
}) => {

  const getIcon = (page: string) => {
    switch (page.toLowerCase()) {
      case 'home':
        return <IconHome size={14} />
      case 'projects':
        return <IconDatabase size={14} />
      case 'plan editor':
        return <IconGraph size={14} />
      case 'graph editor':
        return <IconGraph size={14} />
      default:
        return null
    }
  }

  const items: BreadcrumbItem[] = [
    {
      title: 'Home',
      href: 'home',
      icon: getIcon('home'),
    },
  ]

  // Add project breadcrumb if we have a project
  if (projectId && projectName) {
    items.push({
      title: projectName,
      href: `project-${projectId}`,
      icon: getIcon('projects'),
    })
  }

  // Add current page if specified and not home
  if (currentPage && currentPage !== 'Home') {
    items.push({
      title: currentPage,
      icon: getIcon(currentPage),
    })
  }

  const breadcrumbItems = items.map((item, index) => {
    const isLast = index === items.length - 1

    if (isLast || !item.href) {
      return (
        <Text key={index} size="sm" c="dimmed" style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          {item.icon}
          {item.title}
        </Text>
      )
    }

    return (
      <Anchor
        key={index}
        onClick={() => onNavigate?.(item.href!)}
        size="sm"
        style={{ display: 'flex', alignItems: 'center', gap: 4, cursor: 'pointer' }}
      >
        {item.icon}
        {item.title}
      </Anchor>
    )
  })

  return (
    <MantineBreadcrumbs separator="/" mb="sm">
      {breadcrumbItems}
    </MantineBreadcrumbs>
  )
}