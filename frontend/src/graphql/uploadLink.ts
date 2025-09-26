import { ApolloLink, Operation, NextLink, FetchResult, Observable } from '@apollo/client'
import { print } from 'graphql'

export const createUploadLink = (uri: string) => {
  return new ApolloLink((operation: Operation, forward?: NextLink) => {
    return new Observable<FetchResult>((observer) => {
      const { variables } = operation

      // Convert any File objects to base64 strings
      const processedVariables = convertFilesToBase64(variables)

      // Always send as JSON (no more multipart)
      fetch(uri, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: print(operation.query),
          variables: processedVariables,
          operationName: operation.operationName,
        }),
        credentials: 'omit',
      })
        .then(response => response.json())
        .then(data => {
          observer.next(data)
          observer.complete()
        })
        .catch(error => {
          observer.error(error)
        })
    })
  })
}

function convertFilesToBase64(obj: any): any {
  if (obj instanceof File) {
    // This will be converted to a promise-based approach in the calling code
    return { __isFile: true, file: obj }
  }

  if (Array.isArray(obj)) {
    return obj.map(convertFilesToBase64)
  }

  if (obj && typeof obj === 'object') {
    const result: any = {}
    Object.keys(obj).forEach(key => {
      result[key] = convertFilesToBase64(obj[key])
    })
    return result
  }

  return obj
}