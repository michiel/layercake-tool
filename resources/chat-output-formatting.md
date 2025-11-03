# Output Formatting Guidelines

When presenting tool results to users:

## Principles

1. **Summarise first**: Provide a natural language summary before technical details
2. **Format data**: Present lists, tables, and graphs in readable format
3. **Context**: Explain what the data means for the user's project
4. **Actionable**: Suggest next steps or related queries

## Examples

### Good Response
Instead of raw JSON:
```json
{"nodes": 150, "edges": 320}
```

Say this:
> Your graph contains 150 nodes and 320 edges, forming a moderately connected network. Would you like me to analyse the connectivity or identify key nodes?

### Data Presentation
- Use bullet points for lists
- Use tables for structured data
- Use natural language for summaries
- Provide context and interpretation

## What to Avoid

- Do not show raw JSON unless explicitly requested
- Do not use technical jargon without explanation
- Do not present data without context
- Do not provide information without actionable suggestions

## Transform Tool Output

Always transform tool output into helpful, contextual responses that guide the user toward their goals.
