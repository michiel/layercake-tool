import { useEffect, useId, useState, useRef } from 'react'
import mermaid from 'mermaid'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { IconAlertCircle, IconZoomIn, IconZoomOut, IconZoomScan, IconDownload, IconSun, IconMoon, IconMaximize, IconMinimize, IconExternalLink, IconCopy } from '@tabler/icons-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

// Flag to track if experimental diagrams have been loaded
let experimentalDiagramsLoaded = false

type MermaidPreviewDialogProps = {
  open: boolean
  onClose: () => void
  diagram: string
  title?: string
}

type MermaidTheme = 'default' | 'dark'

export const MermaidPreviewDialog = ({ open, onClose, diagram, title }: MermaidPreviewDialogProps) => {
  const [renderedSvg, setRenderedSvg] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [zoom, setZoom] = useState(1)
  const [theme, setTheme] = useState<MermaidTheme>('default')
  const [isFullscreen, setIsFullscreen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const renderId = useId().replace(/[^a-zA-Z0-9_-]/g, '')

  useEffect(() => {
    if (!open) {
      setRenderedSvg('')
      setError(null)
      setZoom(1)
      setIsFullscreen(false)
      return
    }

    let cancelled = false

    const renderDiagram = async () => {
      try {
        // Initialize mermaid with configuration
        mermaid.initialize({
          startOnLoad: false,
          securityLevel: 'loose',
          theme: theme
        })

        // Ensure experimental diagrams are loaded (for treemap-beta, mindmap, etc.)
        if (!experimentalDiagramsLoaded) {
          try {
            // Force mermaid to load all diagram types including experimental ones
            await mermaid.parse(diagram)
            experimentalDiagramsLoaded = true
          } catch (parseErr) {
            // If parse fails, try to continue with render anyway
            console.warn('Mermaid parse warning:', parseErr)
          }
        }

        const { svg } = await mermaid.render(`mermaid-${renderId}`, diagram)
        if (!cancelled) {
          setRenderedSvg(svg)
          setError(null)
        }
      } catch (err) {
        if (!cancelled) {
          console.error('Failed to render Mermaid diagram', err)
          setError(err instanceof Error ? err.message : 'Failed to render Mermaid diagram')
        }
      }
    }

    renderDiagram()

    return () => {
      cancelled = true
    }
  }, [diagram, open, renderId, theme])

  const handleZoomIn = () => {
    setZoom(prev => Math.min(prev + 0.25, 3))
  }

  const handleZoomOut = () => {
    setZoom(prev => Math.max(prev - 0.25, 0.25))
  }

  const handleZoomFit = () => {
    setZoom(1)
  }

  const handleDownloadSvg = () => {
    if (!renderedSvg) return
    const blob = new Blob([renderedSvg], { type: 'image/svg+xml' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = 'diagram.svg'
    link.click()
    URL.revokeObjectURL(url)
  }

  const handleDownloadPng = () => {
    if (!containerRef.current) return
    const svgElement = containerRef.current.querySelector('svg')
    if (!svgElement) return

    const svgData = new XMLSerializer().serializeToString(svgElement)
    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const img = new Image()
    const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' })
    const url = URL.createObjectURL(svgBlob)

    img.onload = () => {
      canvas.width = img.width
      canvas.height = img.height
      ctx.drawImage(img, 0, 0)
      URL.revokeObjectURL(url)

      canvas.toBlob((blob) => {
        if (!blob) return
        const pngUrl = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = pngUrl
        link.download = 'diagram.png'
        link.click()
        URL.revokeObjectURL(pngUrl)
      })
    }

    img.src = url
  }

  const toggleTheme = () => {
    setTheme(prev => prev === 'default' ? 'dark' : 'default')
  }

  const handleCopyToClipboard = async () => {
    if (!containerRef.current) return
    const svgElement = containerRef.current.querySelector('svg')
    if (!svgElement) return

    try {
      const svgData = new XMLSerializer().serializeToString(svgElement)
      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      if (!ctx) return

      const img = new Image()
      const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' })
      const url = URL.createObjectURL(svgBlob)

      img.onload = async () => {
        canvas.width = img.width
        canvas.height = img.height
        ctx.drawImage(img, 0, 0)
        URL.revokeObjectURL(url)

        canvas.toBlob(async (blob) => {
          if (!blob) return
          try {
            await navigator.clipboard.write([
              new ClipboardItem({
                'image/png': blob
              })
            ])
          } catch (err) {
            console.error('Failed to copy image to clipboard', err)
          }
        })
      }

      img.src = url
    } catch (err) {
      console.error('Failed to copy to clipboard', err)
    }
  }

  const handleOpenInNewWindow = () => {
    if (!renderedSvg) return

    const newWindow = window.open('', '_blank')
    if (!newWindow) return

    newWindow.document.write(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>${title || 'Mermaid Preview'}</title>
          <style>
            body {
              margin: 0;
              padding: 20px;
              display: flex;
              justify-content: center;
              align-items: center;
              min-height: 100vh;
              background: #f5f5f5;
            }
            svg {
              max-width: 100%;
              height: auto;
              background: white;
              box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            }
          </style>
        </head>
        <body>
          ${renderedSvg}
        </body>
      </html>
    `)
    newWindow.document.close()
  }

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent
        className={cn(
          'max-w-[90vw] h-[90vh] flex flex-col',
          isFullscreen && 'max-w-[100vw] w-screen h-screen sm:rounded-none !left-0 !top-0 !translate-x-0 !translate-y-0'
        )}
      >
        <button
          type="button"
          onClick={() => setIsFullscreen(prev => !prev)}
          className="absolute right-12 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
          title={isFullscreen ? 'Exit full screen' : 'Enter full screen'}
          aria-label={isFullscreen ? 'Exit full screen' : 'Enter full screen'}
        >
          {isFullscreen ? <IconMinimize className="h-4 w-4" /> : <IconMaximize className="h-4 w-4" />}
        </button>
        <DialogHeader>
          <DialogTitle>{title || 'Mermaid Preview'}</DialogTitle>
        </DialogHeader>

        {/* Control Buttons */}
        <div className="flex items-center gap-2 pb-2 border-b">
          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="sm"
              onClick={handleZoomIn}
              disabled={zoom >= 3}
              title="Zoom in"
            >
              <IconZoomIn className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleZoomOut}
              disabled={zoom <= 0.25}
              title="Zoom out"
            >
              <IconZoomOut className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleZoomFit}
              title="Zoom to fit"
            >
              <IconZoomScan className="h-4 w-4" />
            </Button>
            <span className="text-xs text-muted-foreground px-2">
              {Math.round(zoom * 100)}%
            </span>
          </div>

          <div className="h-6 w-px bg-border" />

          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownloadSvg}
              disabled={!renderedSvg}
              title="Download SVG"
            >
              <IconDownload className="h-4 w-4 mr-1" />
              SVG
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownloadPng}
              disabled={!renderedSvg}
              title="Download PNG"
            >
              <IconDownload className="h-4 w-4 mr-1" />
              PNG
            </Button>
          </div>

          <div className="h-6 w-px bg-border" />

          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopyToClipboard}
              disabled={!renderedSvg}
              title="Copy image to clipboard"
            >
              <IconCopy className="h-4 w-4 mr-1" />
              Copy
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleOpenInNewWindow}
              disabled={!renderedSvg}
              title="Open in new window"
            >
              <IconExternalLink className="h-4 w-4 mr-1" />
              Open
            </Button>
          </div>

          <div className="h-6 w-px bg-border" />

          <Button
            variant="outline"
            size="sm"
            onClick={toggleTheme}
            title={`Switch to ${theme === 'default' ? 'dark' : 'light'} theme`}
          >
            {theme === 'default' ? (
              <IconMoon className="h-4 w-4" />
            ) : (
              <IconSun className="h-4 w-4" />
            )}
          </Button>
        </div>

        <ScrollArea className="flex-1 w-full border rounded-lg bg-muted/40 p-4">
          {error ? (
            <Alert variant="destructive">
              <IconAlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : renderedSvg ? (
            <div
              ref={containerRef}
              className="mermaid-preview w-full flex items-center justify-center"
              style={{
                transform: `scale(${zoom})`,
                transformOrigin: 'center center',
                transition: 'transform 0.2s ease-out',
              }}
              dangerouslySetInnerHTML={{ __html: renderedSvg }}
            />
          ) : (
            <p className="text-sm text-muted-foreground">Rendering diagram…</p>
          )}
        </ScrollArea>
      </DialogContent>
    </Dialog>
  )
}

export default MermaidPreviewDialog
