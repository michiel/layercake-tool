## Console

The command "layercake console" starts a REPL, a subcommand "chat" is used to start a chat context using a configured LLM provider. The MCP functionality is exposed directly to the chat context.

 - https://crates.io/crates/clap-repl for building the repl, integrate with existing clap code
 - LLM interaction with https://crates.io/crates/llm - configure for ollama, openapi/codex, google/gemini, anthropic/claude by default. This chat interface will also be exposed over GraphQL for later use with the web frontend

### Chat sample

layercake > list-projects
id   label
123  My Project
456  My Project II

layercake > use-project(123)
layercake(123) > list-graphs
id   label
333  My Graph
667  My Graph II

layercake(123) > chat
>> "How can I help?"
<< How many nodes does Graph II have?
>> 34
<< Refresh Graph II
>> Done




