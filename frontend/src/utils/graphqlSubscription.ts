type GraphQLObserver<T> = {
  next?: (value: T) => void
  error?: (error: unknown) => void
  complete?: () => void
}

export type GraphQLSubscriptionHandle = {
  unsubscribe: () => void
}

export type GraphQLSubscribable<T> = {
  subscribe: (observer: GraphQLObserver<T>) => GraphQLSubscriptionHandle
}

export const asGraphQLSubscribable = <T>(value: unknown): GraphQLSubscribable<T> => {
  return value as GraphQLSubscribable<T>
}
