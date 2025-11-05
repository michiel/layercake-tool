import React from 'react'
import { cn } from '@/lib/utils'

interface CenterProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode
  inline?: boolean
}

export const Center = React.forwardRef<HTMLDivElement, CenterProps>(
  ({ inline, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          inline ? 'inline-flex' : 'flex',
          'items-center justify-center',
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Center.displayName = 'Center'
