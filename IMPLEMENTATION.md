## Structured Error Adoption Plan

We are executing the first slice of the error-handling remediation highlighted in `error-review.md`.
This iteration focuses on bringing the library-related GraphQL mutations over to the existing
`StructuredError` helpers so we can prove out the approach before scaling to the rest of the API.

### Task Tracker

| Task | Description | Status | Notes |
| --- | --- | --- | --- |
| Baseline audit | Capture the current count/location of raw `Error::new` usage to confirm focus areas. | Completed | `rg -n "Error::new" layercake-core/src/graphql | wc -l` still reports 137 call sites. |
| Library mutation migration | Update `layercake-core/src/graphql/mutations/mod.rs` library resolvers to use `StructuredError` and contextual helpers. | Completed | Library create/update/delete/reprocess/import/seed mutations now use `StructuredError::{bad_request,validation,not_found,service}` and import the helper. |
| Verification & docs | Run formatting/tests (as possible), record results, and summarize updates here. | Completed | `cargo fmt` and `cargo test -p layercake-core` both fail because the repo contains **both** `layercake-core/src/common.rs` and `layercake-core/src/common/mod.rs` (Rust error E0761). Changes were verified visually; issue predates this work. |

We will update this table as each task progresses and commit once the slice is complete.
