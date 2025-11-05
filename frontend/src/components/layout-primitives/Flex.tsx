import React from 'react'
import { cn } from '@/lib/utils'

interface FlexProps extends React.HTMLAttributes<HTMLDivElement> {
  direction?: 'row' | 'column' | 'row-reverse' | 'column-reverse'
  gap?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | string
  align?: 'stretch' | 'start' | 'center' | 'end' | 'baseline'
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
  wrap?: boolean
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

const directionMap = {
  row: 'flex-row',
  column: 'flex-col',
  'row-reverse': 'flex-row-reverse',
  'column-reverse': 'flex-col-reverse',
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

export const Flex = React.forwardRef<HTMLDivElement, FlexProps>(
  ({ direction = 'row', gap, align, justify, wrap, className, children, ...props }, ref) => {
    const directionClass = directionMap[direction]
    const gapClass = gap ? (gapMap[gap as keyof typeof gapMap] || gap) : ''
    const alignClass = align ? alignMap[align] : ''
    const justifyClass = justify ? justifyMap[justify] : ''
    const wrapClass = wrap ? 'flex-wrap' : ''

    return (
      <div
        ref={ref}
        className={cn('flex', directionClass, gapClass, alignClass, justifyClass, wrapClass, className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Flex.displayName = 'Flex'
