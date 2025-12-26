# Transaction Patterns

This document describes the recommended transaction usage patterns for `layercake-core`
services during the crate separation effort.

## Core Rule

Services accept a `&DatabaseConnection`, which may be a pooled connection or a
transaction handle. Callers are responsible for transaction lifecycle.

## Example

```rust
use layercake::database::connection::establish_connection;
use layercake::services::ProjectService;

let db = establish_connection("sqlite::memory:").await?;
let tx = db.begin().await?;

let service = ProjectService::new(tx.clone());
service.update_project_name(project_id, "New Name").await?;
service.add_project_tag(project_id, "priority").await?;

tx.commit().await?;
```

## Notes

- Keep service calls side-effect free outside of `&DatabaseConnection`.
- For multi-step mutations, open one transaction and pass it to each service.
- Commit only after all steps succeed; rollback on error.
