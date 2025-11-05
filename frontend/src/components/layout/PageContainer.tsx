import React from 'react'
import { cn } from '@/lib/utils'

interface PageContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode
}

/**
 * Shared page wrapper that stretches content to the full viewport width
 * while preserving consistent horizontal padding.
 */
export const PageContainer: React.FC<PageContainerProps> = ({
  children,
  className,
  ...props
}) => {
  return (
    <div
      className={cn('w-full px-8', className)}
      {...props}
    >
      {children}
    </div>
  )
}

export default PageContainer
