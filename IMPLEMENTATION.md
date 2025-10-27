## Structured Error Adoption Plan

We are executing the first slice of the error-handling remediation highlighted in `error-review.md`.
This iteration focuses on bringing the GraphQL mutations over to the existing `StructuredError`
helpers in small, reviewable batches so we can prove out the approach before scaling to the rest of
the API. The first slice (library mutations) is done; we're now extending the effort to the
high-traffic data-source import/export mutations.

### Task Tracker

| Task | Description | Status | Notes |
| --- | --- | --- | --- |
| Baseline audit | Capture the current count/location of raw `Error::new` usage to confirm focus areas. | Completed | `rg -n "Error::new" layercake-core/src/graphql | wc -l` still reports 137 call sites. |
| Library mutation migration | Update `layercake-core/src/graphql/mutations/mod.rs` library resolvers to use `StructuredError` and contextual helpers. | Completed | Library create/update/delete/reprocess/import/seed mutations now use `StructuredError::{bad_request,validation,not_found,service}` and import the helper. |
| Verification & docs | Run formatting/tests (as possible), record results, and summarize updates here. | Completed | `cargo fmt` and `cargo test -p layercake-core` both fail because the repo contains **both** `layercake-core/src/common.rs` and `layercake-core/src/common/mod.rs` (Rust error E0761). Changes were verified visually; issue predates this work. |
| Data source mutation migration | Replace `Error::new` usages in the data-source import/export mutations with `StructuredError`, covering decode failures, unsupported formats, and service/database errors. | Completed | Export/import now map through `StructuredError::{service,bad_request,database}`. |
| Iteration 2 verification | Re-run fmt/tests (noting blockers) and update this log once the data-source slice is complete. | Completed | `cargo fmt` / `cargo test -p layercake-core` still fail because of the duplicate `layercake-core/src/common.rs` vs `layercake-core/src/common/mod.rs` modules (rustc E0761). |

We will update this table as each task progresses and commit once the slice is complete.
