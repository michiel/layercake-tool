import { useEffect, useId, useState } from 'react'
import mermaid from 'mermaid'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { IconAlertCircle, IconX } from '@tabler/icons-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { ScrollArea } from '@/components/ui/scroll-area'

type MermaidPreviewDialogProps = {
  open: boolean
  onClose: () => void
  diagram: string
  title?: string
}

mermaid.initialize({ startOnLoad: false, securityLevel: 'loose' })

export const MermaidPreviewDialog = ({ open, onClose, diagram, title }: MermaidPreviewDialogProps) => {
  const [renderedSvg, setRenderedSvg] = useState('')
  const [error, setError] = useState<string | null>(null)
  const renderId = useId().replace(/[^a-zA-Z0-9_-]/g, '')

  useEffect(() => {
    if (!open) {
      setRenderedSvg('')
      setError(null)
      return
    }

    let cancelled = false

    const renderDiagram = async () => {
      try {
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
  }, [diagram, open, renderId])

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent className="max-w-[90vw] h-[90vh] flex flex-col">
        <DialogHeader className="flex flex-row items-center justify-between pr-10">
          <DialogTitle>{title || 'Mermaid Preview'}</DialogTitle>
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-4 top-4"
            onClick={onClose}
          >
            <IconX size={18} />
          </Button>
        </DialogHeader>
        <ScrollArea className="flex-1 w-full border rounded-lg bg-muted/40 p-4">
          {error ? (
            <Alert variant="destructive">
              <IconAlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : renderedSvg ? (
            <div
              className="mermaid-preview w-full"
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
