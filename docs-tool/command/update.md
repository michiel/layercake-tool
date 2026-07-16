# layercake update

Update the installed `layercake` binary to the latest published release. The
updater reads GitHub release metadata from `michiel/layercake-tool`, downloads
the archive for the current platform, verifies its `.sha256` checksum, and
replaces the running binary in place.

## Usage

```bash
layercake update              # check, confirm, then install the latest release
layercake update --check      # only report whether an update is available
layercake update --force      # reinstall even if already up to date
layercake update --pre-release  # consider pre-release versions too
layercake update --backup     # back up the current binary before replacing it
layercake update --rollback   # restore the most recent backup
layercake update --dry-run    # show what would happen without changing anything
```

## How it fits

Plain `layercake update` is interactive: it reports the current and latest
versions, and only downloads when a newer release exists (use `--force` to
reinstall the same version). `--check` is automation-friendly — it just reports
status and does not install.

The updater resolves the correct asset for your platform by name
(`layercake-<version>-<os>-<arch>.<ext>`), skipping checksum/signature sidecars,
so it picks the archive rather than the `.sha256` file.

## Supported platforms

Pre-built release archives exist for:

- `linux-x86_64`
- `linux-aarch64`
- `macos-aarch64`
- `windows-x86_64`

On other platforms, build from source (see `BUILD.md`) — the updater will report
that no compatible asset was found.

## Operational notes

- Replacing the binary may require write permission to its install directory; on
  Unix, run with `sudo` if the binary lives in a system path.
- `--backup` keeps the previous binary so `--rollback` can restore it if a new
  release misbehaves.
- The download is checksum-verified when a `.sha256` sidecar is published
  alongside the archive.
