import { useEffect, useId, useState, useRef } from 'react'
import { graphviz } from 'd3-graphviz'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { IconAlertCircle, IconZoomIn, IconZoomOut, IconZoomScan, IconDownload, IconMaximize, IconMinimize } from '@tabler/icons-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

type DotPreviewDialogProps = {
  open: boolean
  onClose: () => void
  diagram: string
  title?: string
}

export const DotPreviewDialog = ({ open, onClose, diagram, title }: DotPreviewDialogProps) => {
  const [error, setError] = useState<string | null>(null)
  const [zoom, setZoom] = useState(1)
  const [isRendering, setIsRendering] = useState(false)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const graphvizRef = useRef<any>(null)
  const renderId = useId().replace(/[^a-zA-Z0-9_-]/g, '')

  useEffect(() => {
    if (!open) {
      setError(null)
      setZoom(1)
      setIsFullscreen(false)
      return
    }

    if (!diagram) {
      return
    }

    let cancelled = false
    setIsRendering(true)

    const renderDiagram = () => {
      // Wait for container to be ready
      if (!containerRef.current) {
        setTimeout(renderDiagram, 50)
        return
      }

      try {
        const selector = `#graphviz-${renderId}`
        const element = document.querySelector(selector)

        if (!element) {
          throw new Error(`Element not found: ${selector}`)
        }

        const viz = graphviz(selector)
          .fit(true)
          .zoom(false)
          .on('end', () => {
            if (!cancelled) {
              setError(null)
              setIsRendering(false)
            }
          })
          .renderDot(diagram)

        graphvizRef.current = viz
      } catch (err) {
        if (!cancelled) {
          console.error('Failed to render Graphviz diagram', err)
          setError(err instanceof Error ? err.message : 'Failed to render Graphviz diagram')
          setIsRendering(false)
        }
      }
    }

    // Start rendering with a small delay
    const timeoutId = setTimeout(renderDiagram, 100)

    return () => {
      cancelled = true
      clearTimeout(timeoutId)
    }
  }, [diagram, open, renderId])

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
    if (!containerRef.current) return
    const svgElement = containerRef.current.querySelector('svg')
    if (!svgElement) return

    const svgData = new XMLSerializer().serializeToString(svgElement)
    const blob = new Blob([svgData], { type: 'image/svg+xml' })
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
          <DialogTitle>{title || 'Graphviz Preview'}</DialogTitle>
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
              disabled={isRendering || !!error}
              title="Download SVG"
            >
              <IconDownload className="h-4 w-4 mr-1" />
              SVG
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownloadPng}
              disabled={isRendering || !!error}
              title="Download PNG"
            >
              <IconDownload className="h-4 w-4 mr-1" />
              PNG
            </Button>
          </div>
        </div>

        <ScrollArea className="flex-1 w-full border rounded-lg bg-muted/40 p-4">
          {error ? (
            <Alert variant="destructive">
              <IconAlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : (
            <div
              id={`graphviz-${renderId}`}
              ref={containerRef}
              className="graphviz-preview w-full flex items-center justify-center"
              style={{
                transform: `scale(${zoom})`,
                transformOrigin: 'center center',
                transition: 'transform 0.2s ease-out',
                minHeight: '400px',
              }}
            />
          )}
          {isRendering && !error && (
            <p className="text-sm text-muted-foreground">Rendering diagram…</p>
          )}
        </ScrollArea>
      </DialogContent>
    </Dialog>
  )
}
