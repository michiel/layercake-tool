# Command: `layercake doc`

Print embedded agent-facing documentation. Docs are compiled into the binary,
so they work offline and match the binary's version.

## Usage

```bash
layercake doc list                     # list all workflows, commands, and guides
layercake doc workflow <name>          # print docs-tool/workflow/<name>.md
layercake doc command <name>           # print docs-tool/command/<name>.md
layercake doc guide <name>             # print docs-tool/guide/<name>.md
```

## Examples

```bash
layercake doc list
layercake doc workflow edit-a-plan
layercake doc command schema
layercake doc guide agent              # the AI agent query-interface guide
layercake doc guide model              # the graph model documentation
layercake doc guide node-types         # every plan DAG node type + its config
layercake doc guide graph-json         # the GraphJson schema for datasets
```

## For agents

Start with `layercake doc list` to discover available topics, then read the
workflow that matches your task. Workflows are end-to-end task guides; command
docs describe individual commands. Output is Markdown on stdout — pipe or parse
freely.
