import { useEffect, useState } from 'react'
import { useMutation } from '@apollo/client/react'
import { CREATE_PLAN } from '@/graphql/plans'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'

export interface CreatePlanModalProps {
  projectId: number
  open: boolean
  onOpenChange: (open: boolean) => void
  onCreated?: () => void
}

export const parsePlanTags = (value: string): string[] =>
  value
    .split(',')
    .map((tag) => tag.trim())
    .filter((tag) => Boolean(tag))

export const CreatePlanModal = ({ projectId, open, onOpenChange, onCreated }: CreatePlanModalProps) => {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [tagsInput, setTagsInput] = useState('')

  useEffect(() => {
    if (!open) {
      setName('')
      setDescription('')
      setTagsInput('')
    }
  }, [open])

  const [createPlan, { loading }] = useMutation(CREATE_PLAN, {
    onCompleted: () => {
      showSuccessNotification('Plan created', 'New plan was created successfully.')
      onCreated?.()
      onOpenChange(false)
    },
    onError: (error: Error) => {
      console.error('Failed to create plan', error)
      showErrorNotification('Plan creation failed', error.message)
    },
  })

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()

    if (!name.trim()) {
      showErrorNotification('Missing name', 'Please provide a plan name.')
      return
    }

    const tagValues = parsePlanTags(tagsInput)

    await createPlan({
      variables: {
        input: {
          projectId,
          name: name.trim(),
          description: description.trim().length ? description.trim() : null,
          tags: tagValues.length ? tagValues : null,
          yamlContent: '',
          dependencies: [],
        },
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle>Create plan</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="plan-name">Name</Label>
            <Input
              id="plan-name"
              placeholder="Main plan"
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="plan-description">Description</Label>
            <Textarea
              id="plan-description"
              placeholder="Optional description"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              rows={3}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="plan-tags">Tags</Label>
            <Input
              id="plan-tags"
              placeholder="analysis, staging"
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
              {loading ? 'Creating…' : 'Create plan'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
