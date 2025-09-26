import React, { Component, ReactNode } from 'react'
import { Alert, Stack, Button, Text } from '@mantine/core'
import { IconAlertTriangle, IconRefresh } from '@tabler/icons-react'

interface Props {
  children: ReactNode
  fallback?: ReactNode
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void
}

interface State {
  hasError: boolean
  error?: Error
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
    this.props.onError?.(error, errorInfo)
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined })
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <Alert icon={<IconAlertTriangle size="1rem" />} title="Something went wrong" color="red">
          <Stack gap="md">
            <Text size="sm">
              {this.state.error?.message || 'An unexpected error occurred'}
            </Text>
            <Button
              leftSection={<IconRefresh size="1rem" />}
              variant="light"
              size="sm"
              onClick={this.handleReset}
            >
              Try Again
            </Button>
          </Stack>
        </Alert>
      )
    }

    return this.props.children
  }
}