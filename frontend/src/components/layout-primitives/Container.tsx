import React from 'react'
import { cn } from '@/lib/utils'

interface ContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full'
  fluid?: boolean
  children?: React.ReactNode
}

const sizeMap = {
  xs: 'max-w-xs',
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-4xl',
  xl: 'max-w-6xl',
  '2xl': 'max-w-7xl',
  full: 'max-w-full',
}

export const Container = React.forwardRef<HTMLDivElement, ContainerProps>(
  ({ size = 'lg', fluid, className, children, ...props }, ref) => {
    const sizeClass = fluid ? 'w-full' : sizeMap[size]

    return (
      <div
        ref={ref}
        className={cn('container mx-auto px-4', sizeClass, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Container.displayName = 'Container'
