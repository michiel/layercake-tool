# Chat Credential Management

Layercake stores provider secrets in the `chat_credentials` table. Migrations ship empty records for each built-in provider so credentials can be updated without manual SQL.

## Seeding

`cargo run -- chat-credentials list`

If you see entries for `ollama`, `openai`, `gemini`, and `claude`, the seeding migration has executed successfully. Empty cells indicate an unset API key or base URL.

## Editing Credentials

```
cargo run -- chat-credentials set openai --api-key sk-live-... --base-url https://api.openai.com
cargo run -- chat-credentials set claude --api-key sk-ant-...
cargo run -- chat-credentials clear gemini
```

The command accepts an optional `--database` flag if you manage a custom SQLite path.

## Environment Overrides

For quick experiments you can still rely on environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`, `OLLAMA_BASE_URL`). When both a database value and environment variable exist, the CLI/console uses the stored credential first.

### Example: Google AI Studio (Gemini)

1. Create an API key in [Google AI Studio](https://aistudio.google.com/). The dashboard lists a short label plus the full key (starts with `AIza`).
2. Configure the credential:

   ```bash
   cargo run -- chat-credentials set gemini \
     --api-key AIzaSy...YourKey...
   ```

   Gemini currently uses the hosted Google endpoint, so a custom `--base-url` is not required. If Google introduces regional endpoints later, set them with `--base-url https://...`.

3. Verify storage:

   ```bash
   cargo run -- chat-credentials list
   ```

   The `gemini` row should show `********` in the `api_key` column and `-` for `base_url`.

4. Console usage:

   Select a project in the REPL, then run `chat --provider gemini`. The session will use the stored key; if you unset the database credential, you can fall back to `GOOGLE_API_KEY` in the environment.
