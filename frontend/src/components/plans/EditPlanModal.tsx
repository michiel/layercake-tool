import { useEffect, useState } from 'react'
import { useMutation } from '@apollo/client/react'
import { UPDATE_PLAN } from '@/graphql/plans'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { Plan } from '@/types/plan'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { parsePlanTags } from './CreatePlanModal'

interface EditPlanModalProps {
  plan: Plan | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdated?: () => void
}

export const EditPlanModal = ({ plan, open, onOpenChange, onUpdated }: EditPlanModalProps) => {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [tagsInput, setTagsInput] = useState('')

  useEffect(() => {
    if (plan && open) {
      setName(plan.name)
      setDescription(plan.description || '')
      setTagsInput(plan.tags.join(', '))
    }
  }, [plan, open])

  const [updatePlan, { loading }] = useMutation(UPDATE_PLAN, {
    onCompleted: () => {
      showSuccessNotification('Plan updated', 'Plan details were updated successfully.')
      onUpdated?.()
      onOpenChange(false)
    },
    onError: (error: Error) => {
      console.error('Failed to update plan', error)
      showErrorNotification('Plan update failed', error.message)
    },
  })

  if (!plan) {
    return null
  }

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()

    if (!name.trim()) {
      showErrorNotification('Missing name', 'Please provide a plan name.')
      return
    }

    const tagValues = parsePlanTags(tagsInput)

    await updatePlan({
      variables: {
        id: plan.id,
        input: {
          name: name.trim(),
          description: description.trim().length ? description.trim() : null,
          tags: tagValues.length ? tagValues : null,
        },
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle>Edit plan</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="edit-plan-name">Name</Label>
            <Input
              id="edit-plan-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-plan-description">Description</Label>
            <Textarea
              id="edit-plan-description"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              rows={3}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-plan-tags">Tags</Label>
            <Input
              id="edit-plan-tags"
              value={tagsInput}
              onChange={(event) => setTagsInput(event.target.value)}
            />
            <p className="text-xs text-muted-foreground">Separate tags with commas.</p>
          </div>
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? 'Saving…' : 'Save changes'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
