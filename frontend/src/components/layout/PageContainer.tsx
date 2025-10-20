import React from 'react'
import { Container, ContainerProps } from '@mantine/core'

type PageContainerProps = ContainerProps

/**
 * Shared page wrapper that stretches content to the full viewport width
 * while preserving consistent horizontal padding.
 */
export const PageContainer: React.FC<PageContainerProps> = ({
  children,
  fluid,
  px,
  maw,
  style,
  ...rest
}) => {
  return (
    <Container
      fluid={fluid ?? true}
      px={px ?? 'xl'}
      maw={maw ?? '100%'}
      style={{ width: '100%', ...style }}
      {...rest}
    >
      {children}
    </Container>
  )
}

export default PageContainer
