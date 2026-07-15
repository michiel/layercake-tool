# Command: `layercake schema dump`

Print the GraphQL API surface. Works **offline** — no running server or
database is required, because the type system does not depend on runtime data.

## Usage

```bash
layercake schema dump                 # full GraphQL SDL
layercake schema dump --json          # introspection JSON (for codegen/tooling)
layercake schema dump --only-inputs   # just the input object types
layercake schema dump --only-mutations# just the Mutation root
layercake schema type <Name>          # one type's SDL, e.g. SequenceEdgeRefInput
```

## Examples

```bash
# See the shape of a specific input type before a mutation
layercake schema dump | grep -A20 "input PlanDagInput"

# List all queries
layercake schema dump | sed -n '/^type Query {/,/^}/p'

# Feed introspection JSON to a GraphQL codegen tool
layercake schema dump --json > schema.json
```

## For agents

Run this first when you need to construct a query or mutation and are unsure of
field names or argument types. The SDL is authoritative for the running binary's
version. To execute an operation against a live server, see
`layercake doc command api` (`layercake api call`).
