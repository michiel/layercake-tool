# Layercake

This is a Rust workspace for the Layercake project, a tool for layer-based visualization of complex systems.

## Crates

The workspace contains the following crates:

- **layercake-tool**: The core CLI tool for creating and managing layercake visualizations

## Development

Navigate to the layercake-tool directory and run the standard Rust commands:

```bash
cd layercake-tool
cargo build
cargo test
cargo run -- -p sample/ref/plan.yaml
```

Or use workspace-level commands:

```bash
cargo build -p layercake-tool
cargo test -p layercake-tool
cargo run -p layercake-tool -- -p layercake-tool/sample/ref/plan.yaml
```

## License

MIT