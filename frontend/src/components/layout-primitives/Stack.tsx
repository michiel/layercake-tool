import React from 'react'
import { cn } from '@/lib/utils'

interface StackProps extends React.HTMLAttributes<HTMLDivElement> {
  gap?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | string
  align?: 'stretch' | 'start' | 'center' | 'end' | 'baseline'
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
  children?: React.ReactNode
}

const gapMap = {
  xs: 'gap-1',
  sm: 'gap-2',
  md: 'gap-4',
  lg: 'gap-6',
  xl: 'gap-8',
  '2xl': 'gap-10',
  '3xl': 'gap-12',
  '4xl': 'gap-16',
}

const alignMap = {
  stretch: 'items-stretch',
  start: 'items-start',
  center: 'items-center',
  end: 'items-end',
  baseline: 'items-baseline',
}

const justifyMap = {
  start: 'justify-start',
  center: 'justify-center',
  end: 'justify-end',
  between: 'justify-between',
  around: 'justify-around',
  evenly: 'justify-evenly',
}

export const Stack = React.forwardRef<HTMLDivElement, StackProps>(
  ({ gap = 'md', align, justify, className, children, ...props }, ref) => {
    const gapClass = gapMap[gap as keyof typeof gapMap] || gap
    const alignClass = align ? alignMap[align] : ''
    const justifyClass = justify ? justifyMap[justify] : ''

    return (
      <div
        ref={ref}
        className={cn('flex flex-col', gapClass, alignClass, justifyClass, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Stack.displayName = 'Stack'
