import { useMutation, useQuery } from '@apollo/client/react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Spinner } from '@/components/ui/spinner'
import { REMOVE_LAYER_ALIAS, GET_LAYER_ALIASES } from '@/graphql/layers'
import { IconX } from '@tabler/icons-react'
import type { LayerAlias } from '@/graphql/layers'

interface ManageAliasesDialogProps {
  open: boolean
  onClose: () => void
  projectId: number
  targetLayerId: number
  layerName: string
}

export const ManageAliasesDialog = ({
  open,
  onClose,
  projectId,
  targetLayerId,
  layerName,
}: ManageAliasesDialogProps) => {
  const { data, loading, refetch } = useQuery(GET_LAYER_ALIASES, {
    variables: { projectId, targetLayerId },
    skip: !open,
  })

  const [removeAlias, { loading: removing }] = useMutation(REMOVE_LAYER_ALIAS, {
    onCompleted: () => {
      refetch()
    },
    refetchQueries: ['GetProjectLayers'],
  })

  const aliases = ((data as any)?.getLayerAliases || []) as LayerAlias[]

  const handleRemove = (aliasLayerId: string) => {
    if (confirm(`Remove alias '${aliasLayerId}'?`)) {
      removeAlias({
        variables: { projectId, aliasLayerId },
      })
    }
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Manage Aliases for '{layerName}'</DialogTitle>
        </DialogHeader>

        <div className="py-4">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Spinner size="md" />
            </div>
          ) : aliases.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No aliases for this layer.
            </p>
          ) : (
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground mb-3">
                These layers are aliased to '{layerName}':
              </p>
              {aliases.map((alias) => (
                <div
                  key={alias.id}
                  className="flex items-center justify-between p-2 border rounded"
                >
                  <span className="font-mono text-sm">{alias.aliasLayerId}</span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleRemove(alias.aliasLayerId)}
                    disabled={removing}
                  >
                    <IconX className="h-4 w-4 mr-1" />
                    Remove Alias
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
