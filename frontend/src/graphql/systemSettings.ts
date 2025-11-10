import { gql } from '@apollo/client'

export type SystemSettingValueType = 'String' | 'Text' | 'Url' | 'Integer' | 'Float' | 'Boolean' | 'Enum' | 'Secret'

export interface SystemSetting {
  key: string
  label: string
  category: string
  description?: string | null
  value?: string | null
  valueType: SystemSettingValueType
  allowedValues: string[]
  isSecret: boolean
  isReadOnly: boolean
  updatedAt: string
}

export interface GetSystemSettingsResponse {
  systemSettings: SystemSetting[]
}

export interface UpdateSystemSettingResponse {
  updateSystemSetting: SystemSetting
}

export const GET_SYSTEM_SETTINGS = gql`
  query GetSystemSettings {
    systemSettings {
      key
      label
      category
      description
      value
      valueType
      allowedValues
      isSecret
      isReadOnly
      updatedAt
    }
  }
`

export const UPDATE_SYSTEM_SETTING = gql`
  mutation UpdateSystemSetting($input: SystemSettingUpdateInput!) {
    updateSystemSetting(input: $input) {
      key
      label
      category
      description
      value
      valueType
      allowedValues
      isSecret
      isReadOnly
      updatedAt
    }
  }
`
