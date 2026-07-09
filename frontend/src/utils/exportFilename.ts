/**
 * Build a download filename when an artefact export returns an empty filename
 * (the backend does this when the node has no configured outputPath).
 *
 * Uses the render target / node label as the base and an extension derived from
 * the export MIME type, so the browser never downloads a nameless file.
 */
const MIME_EXTENSIONS: Record<string, string> = {
  'text/vnd.graphviz': 'dot',
  'text/plain': 'txt',
  'application/json': 'json',
  'text/csv': 'csv',
  'image/svg+xml': 'svg',
  'text/x-plantuml': 'puml',
  'text/x-mermaid': 'mmd',
}

export function fallbackExportFilename(
  renderTarget: string | undefined,
  label: string | undefined,
  mimeType: string | undefined,
): string {
  const base =
    (renderTarget || label || 'export')
      .toString()
      .trim()
      .replace(/[^a-zA-Z0-9._-]+/g, '_')
      .replace(/^_+|_+$/g, '') || 'export'
  const ext = (mimeType && MIME_EXTENSIONS[mimeType]) || 'txt'
  return base.includes('.') ? base : `${base}.${ext}`
}
