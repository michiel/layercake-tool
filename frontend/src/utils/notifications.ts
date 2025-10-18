import React from 'react'
import { notifications } from '@mantine/notifications'
import { IconAlertCircle, IconCheck, IconInfoCircle } from '@tabler/icons-react'

/**
 * Show an error notification toast
 */
export const showErrorNotification = (title: string, message?: string) => {
  notifications.show({
    title,
    message: message || '',
    color: 'red',
    icon: React.createElement(IconAlertCircle, { size: 18 }),
    autoClose: 5000,
  })
}

/**
 * Show a success notification toast
 */
export const showSuccessNotification = (title: string, message?: string) => {
  notifications.show({
    title,
    message: message || '',
    color: 'green',
    icon: React.createElement(IconCheck, { size: 18 }),
    autoClose: 3000,
  })
}

/**
 * Show an info notification toast
 */
export const showInfoNotification = (title: string, message?: string) => {
  notifications.show({
    title,
    message: message || '',
    color: 'blue',
    icon: React.createElement(IconInfoCircle, { size: 18 }),
    autoClose: 4000,
  })
}
