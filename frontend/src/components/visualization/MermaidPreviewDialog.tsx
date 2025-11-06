import { useEffect, useId, useState, useRef } from 'react'
import mermaid from 'mermaid'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { IconAlertCircle, IconZoomIn, IconZoomOut, IconZoomScan, IconDownload, IconSun, IconMoon } from '@tabler/icons-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'

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
  const containerRef = useRef<HTMLDivElement>(null)
  const renderId = useId().replace(/[^a-zA-Z0-9_-]/g, '')

  useEffect(() => {
    if (!open) {
      setRenderedSvg('')
      setError(null)
      setZoom(1)
      return
    }

    let cancelled = false

    const renderDiagram = async () => {
      try {
        mermaid.initialize({
          startOnLoad: false,
          securityLevel: 'loose',
          theme: theme
        })
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

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent className="max-w-[90vw] h-[90vh] flex flex-col">
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
