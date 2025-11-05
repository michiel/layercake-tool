import React from 'react'
import { toast } from 'sonner'
import { IconAlertCircle, IconCheck, IconInfoCircle } from '@tabler/icons-react'

/**
 * Show an error notification toast
 */
export const showErrorNotification = (title: string, message?: string) => {
  toast.error(title, {
    description: message || undefined,
    icon: React.createElement(IconAlertCircle, { size: 18 }),
    duration: 5000,
  })
}

/**
 * Show a success notification toast
 */
export const showSuccessNotification = (title: string, message?: string) => {
  toast.success(title, {
    description: message || undefined,
    icon: React.createElement(IconCheck, { size: 18 }),
    duration: 3000,
  })
}

/**
 * Show an info notification toast
 */
export const showInfoNotification = (title: string, message?: string) => {
  toast.info(title, {
    description: message || undefined,
    icon: React.createElement(IconInfoCircle, { size: 18 }),
    duration: 4000,
  })
}
