import { useEffect, useId, useState, useRef } from 'react'
import { graphviz } from 'd3-graphviz'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { IconAlertCircle, IconZoomIn, IconZoomOut, IconZoomScan, IconDownload } from '@tabler/icons-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'

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
  const containerRef = useRef<HTMLDivElement>(null)
  const graphvizRef = useRef<any>(null)
  const renderId = useId().replace(/[^a-zA-Z0-9_-]/g, '')

  console.log('[DotPreview] Component render:', { open, diagramLength: diagram?.length, title })

  useEffect(() => {
    console.log('[DotPreview] useEffect triggered:', {
      open,
      diagramLength: diagram?.length,
      containerExists: !!containerRef.current,
      renderId
    })

    if (!open) {
      console.log('[DotPreview] Dialog not open, skipping')
      setError(null)
      setZoom(1)
      return
    }

    if (!diagram) {
      console.log('[DotPreview] No diagram content')
      return
    }

    if (!containerRef.current) {
      console.log('[DotPreview] Container ref not ready')
      return
    }

    let cancelled = false
    setIsRendering(true)

    const renderDiagram = () => {
      try {
        console.log('[DotPreview] Starting render', {
          renderId,
          selector: `#graphviz-${renderId}`,
          diagramLength: diagram.length,
          containerExists: !!containerRef.current
        })

        const selector = `#graphviz-${renderId}`
        const element = document.querySelector(selector)

        console.log('[DotPreview] Element found:', !!element)

        if (!element) {
          throw new Error(`Element not found: ${selector}`)
        }

        const viz = graphviz(selector)
          .fit(true)
          .zoom(false)
          .on('end', () => {
            console.log('[DotPreview] Render complete')
            if (!cancelled) {
              setError(null)
              setIsRendering(false)
            }
          })
          .renderDot(diagram)

        console.log('[DotPreview] Graphviz instance created:', !!viz)
        graphvizRef.current = viz
      } catch (err) {
        if (!cancelled) {
          console.error('[DotPreview] Failed to render Graphviz diagram', err)
          setError(err instanceof Error ? err.message : 'Failed to render Graphviz diagram')
          setIsRendering(false)
        }
      }
    }

    // Delay rendering slightly to ensure DOM is ready
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
      <DialogContent className="max-w-[90vw] h-[90vh] flex flex-col">
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
