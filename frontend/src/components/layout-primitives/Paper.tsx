import React from 'react'
import { cn } from '@/lib/utils'

interface PaperProps extends React.HTMLAttributes<HTMLDivElement> {
  shadow?: 'none' | 'sm' | 'md' | 'lg' | 'xl'
  withBorder?: boolean
  radius?: 'none' | 'sm' | 'md' | 'lg'
  p?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  children?: React.ReactNode
}

const shadowMap = {
  none: '',
  sm: 'shadow-sm',
  md: 'shadow-md',
  lg: 'shadow-lg',
  xl: 'shadow-xl',
}

const radiusMap = {
  none: 'rounded-none',
  sm: 'rounded-sm',
  md: 'rounded-md',
  lg: 'rounded-lg',
}

const paddingMap = {
  xs: 'p-2',
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
  xl: 'p-8',
}

export const Paper = React.forwardRef<HTMLDivElement, PaperProps>(
  ({ shadow = 'sm', withBorder, radius = 'md', p, className, children, ...props }, ref) => {
    const shadowClass = shadowMap[shadow]
    const borderClass = withBorder ? 'border border-border' : ''
    const radiusClass = radiusMap[radius]
    const paddingClass = p ? paddingMap[p] : ''

    return (
      <div
        ref={ref}
        className={cn('bg-card text-card-foreground', shadowClass, borderClass, radiusClass, paddingClass, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Paper.displayName = 'Paper'
