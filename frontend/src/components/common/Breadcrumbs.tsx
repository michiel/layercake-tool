import React from 'react'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'
import { IconHome, IconDatabase, IconGraph } from '@tabler/icons-react'

interface BreadcrumbItemData {
  title: string
  href?: string
  icon?: React.ReactNode
}

interface BreadcrumbsProps {
  projectName?: string
  projectId?: number
  currentPage?: string
  sections?: Array<{ title: string; href?: string }>
  onNavigate?: (path: string) => void
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({
  projectName,
  projectId,
  currentPage,
  sections,
  onNavigate
}) => {

  const getIcon = (page: string) => {
    switch (page.toLowerCase()) {
      case 'home':
        return <IconHome size={12} />
      case 'projects':
        return <IconDatabase size={12} />
      case 'plan editor':
        return <IconGraph size={12} />
      case 'graph editor':
        return <IconGraph size={12} />
      default:
        return null
    }
  }

  const items: BreadcrumbItemData[] = [
    {
      title: 'Home',
      href: '/',
      icon: getIcon('home'),
    },
  ]

  // Add project breadcrumb if we have a project
  if (projectId && projectName) {
    items.push({
      title: projectName,
      href: `/projects/${projectId}`,
      icon: getIcon('projects'),
    })
  }

  // Add optional intermediate sections
  if (sections && sections.length > 0) {
    sections.forEach(section => {
      items.push({
        title: section.title,
        href: section.href,
        icon: section.href ? getIcon(section.title) : undefined,
      })
    })
  }

  // Add current page if specified and not home
  if (currentPage && currentPage !== 'Home') {
    items.push({
      title: currentPage,
      icon: getIcon(currentPage),
    })
  }

  const handleBreadcrumbClick = (
    event: React.MouseEvent<HTMLAnchorElement>,
    href: string
  ) => {
    if (!onNavigate) {
      return
    }
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.altKey ||
      event.ctrlKey ||
      event.shiftKey
    ) {
      return
    }

    event.preventDefault()
    onNavigate(href)
  }

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {items.map((item, index) => {
          const isLast = index === items.length - 1

          return (
            <React.Fragment key={index}>
              <BreadcrumbItem>
                {isLast || !item.href ? (
                  <BreadcrumbPage className="flex items-center gap-1">
                    {item.icon}
                    <span>{item.title}</span>
                  </BreadcrumbPage>
                ) : (
                  <BreadcrumbLink
                    href={item.href!}
                    onClick={(event) => handleBreadcrumbClick(event, item.href!)}
                    className="flex items-center gap-1 cursor-pointer"
                  >
                    {item.icon}
                    <span>{item.title}</span>
                  </BreadcrumbLink>
                )}
              </BreadcrumbItem>
              {!isLast && <BreadcrumbSeparator />}
            </React.Fragment>
          )
        })}
      </BreadcrumbList>
    </Breadcrumb>
  )
}
