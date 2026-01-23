# Error Handling Guidance

- Prefer using `StructuredError` helpers over `Error::new` for GraphQL resolvers. The helpers ensure errors carry meaningful `extensions.code` values such as `NOT_FOUND`, `BAD_REQUEST`, `SERVICE_ERROR`, or `DATABASE_ERROR`.
- Wrap database calls with `StructuredError::database("Operation", err)` so the failing operation is encoded in the response.
- Use `StructuredError::service("Service::method", err)` for service-layer failures (e.g., other Rust services or external APIs).
- Validate inputs early and return `StructuredError::bad_request(...)` for malformed JSON, base64 decoding errors, or unsupported options to give clients actionable feedback.
- When a resource lookup fails, return `StructuredError::not_found("Resource", id)` instead of empty responses. This keeps clients informed about missing data.
- Internal unexpected conditions (e.g., missing context) should use `StructuredError::internal(...)` sparingly and include enough detail for investigation without leaking sensitive data.
