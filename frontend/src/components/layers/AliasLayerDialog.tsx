import { useState } from 'react'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Spinner } from '@/components/ui/spinner'
import { CREATE_LAYER_ALIAS, GET_PROJECT_LAYERS } from '@/graphql/layers'
import type { ProjectLayer } from '@/graphql/layers'

interface AliasLayerDialogProps {
  open: boolean
  onClose: () => void
  projectId: number
  missingLayerId: string
  onSuccess: () => void
}

export const AliasLayerDialog = ({
  open,
  onClose,
  projectId,
  missingLayerId,
  onSuccess,
}: AliasLayerDialogProps) => {
  const [selectedLayerId, setSelectedLayerId] = useState<number | null>(null)

  const { data: layersData, loading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !open,
  })

  const [createAlias, { loading: creating }] = useMutation(CREATE_LAYER_ALIAS, {
    onCompleted: () => {
      onSuccess()
      onClose()
      setSelectedLayerId(null)
    },
    refetchQueries: ['GetProjectLayers'],
  })

  const handleCreate = () => {
    if (!selectedLayerId) return

    createAlias({
      variables: {
        projectId,
        aliasLayerId: missingLayerId,
        targetLayerId: selectedLayerId,
      },
    })
  }

  const handleClose = () => {
    setSelectedLayerId(null)
    onClose()
  }

  const paletteLayers = ((layersData as any)?.projectLayers?.filter(
    (l: ProjectLayer) => l.enabled
  ) || []) as ProjectLayer[]

  return (
    <Dialog open={open} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            Alias '{missingLayerId}' to existing layer
          </DialogTitle>
        </DialogHeader>

        <div className="py-4">
          <p className="text-sm text-muted-foreground mb-4">
            Select a layer to use for '{missingLayerId}':
          </p>

          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Spinner size="md" />
            </div>
          ) : paletteLayers.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No enabled layers available. Please enable layers from the Palette tab first.
            </p>
          ) : (
            <div className="space-y-3 max-h-[400px] overflow-y-auto">
              {paletteLayers.map((layer) => (
                <div
                  key={layer.id}
                  className="flex items-center space-x-3 p-3 border rounded hover:bg-muted/50 cursor-pointer"
                  onClick={() => setSelectedLayerId(layer.id)}
                >
                  <input
                    type="radio"
                    checked={selectedLayerId === layer.id}
                    onChange={() => setSelectedLayerId(layer.id)}
                    id={`layer-${layer.id}`}
                    className="cursor-pointer"
                  />
                  <Label
                    htmlFor={`layer-${layer.id}`}
                    className="flex-1 cursor-pointer"
                  >
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-sm font-medium">{layer.layerId}</span>
                        <span className="text-sm text-muted-foreground">-</span>
                        <span className="text-sm">{layer.name}</span>
                      </div>
                      <div className="flex gap-2 mt-2">
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.backgroundColor }}
                            title="Background"
                          />
                          <span className="text-xs text-muted-foreground">BG</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.borderColor }}
                            title="Border"
                          />
                          <span className="text-xs text-muted-foreground">Border</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.textColor }}
                            title="Text"
                          />
                          <span className="text-xs text-muted-foreground">Text</span>
                        </div>
                      </div>
                    </Label>
                  </div>
                ))}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={handleClose} disabled={creating}>
            Cancel
          </Button>
          <Button
            onClick={handleCreate}
            disabled={!selectedLayerId || creating || paletteLayers.length === 0}
          >
            {creating ? 'Creating...' : 'Create Alias'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
