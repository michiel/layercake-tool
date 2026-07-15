# Workflow: Run a plan headless (no server)

Execute a plan file end-to-end from the CLI — useful for CI, batch runs, or
agents that just need outputs without a running server or UI.

## 1. Get a plan file

Create a starter plan, or generate a sample project:

```bash
layercake init --plan my-plan.yaml           # write a default plan YAML
layercake generate sample <sample> <dir>     # scaffold a sample project
layercake generate template <exporter>       # print an exporter template
```

## 2. Run it

```bash
layercake run --plan my-plan.yaml            # execute once
layercake run --plan my-plan.yaml --watch    # re-run on file changes
```

The plan defines the inputs (datasets), transforms, and exporters; running it
produces the configured outputs.

## Notes

- This path does not need `layercake serve`; it reads the plan and writes
  outputs directly.
- To instead build/edit plans interactively or via the API against a live
  instance, see `layercake doc workflow edit-a-plan` and
  `layercake doc workflow drive-via-api`.
