import { useState, useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Stack } from '@/components/layout-primitives'
import { ReactFlowEdge } from '../../../types/plan-dag'

// Zod schema for edge form validation
const edgeConfigSchema = z.object({
  label: z.string().min(1, 'Label is required').transform((val) => val.trim()),
  dataType: z.enum(['GRAPH_DATA', 'GRAPH_REFERENCE']),
})

type EdgeFormData = z.infer<typeof edgeConfigSchema>

interface EdgeConfigDialogProps {
  edge: ReactFlowEdge | null
  opened: boolean
  onClose: () => void
  onSave: (edgeId: string, updates: Partial<ReactFlowEdge>) => void
  readonly?: boolean
}

export const EdgeConfigDialog = ({
  edge,
  opened,
  onClose,
  onSave,
  readonly = false
}: EdgeConfigDialogProps) => {
  const [loading, setLoading] = useState(false)

  const form = useForm<EdgeFormData>({
    resolver: zodResolver(edgeConfigSchema),
    defaultValues: {
      label: '',
      dataType: 'GRAPH_DATA',
    },
  })

  // Update form when edge changes
  useEffect(() => {
    if (edge) {
      form.reset({
        label: edge.label || '',
        dataType: (edge.metadata?.dataType as 'GRAPH_DATA' | 'GRAPH_REFERENCE') || 'GRAPH_DATA'
      })
    }
  }, [edge, form])

  const handleSubmit = async (values: EdgeFormData) => {
    if (!edge) return

    setLoading(true)
    try {
      // Prepare edge updates
      const updates: Partial<ReactFlowEdge> = {
        label: values.label,
        metadata: {
          ...edge.metadata,
          dataType: values.dataType,
          label: values.label
        },
        style: {
          ...edge.style,
          stroke: values.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96'
        }
      }

      await onSave(edge.id, updates)
      onClose()
    } catch (error) {
      console.error('Failed to update edge:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    form.reset()
    onClose()
  }

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Edge Configuration</DialogTitle>
          {edge && (
            <DialogDescription>
              Connection: {edge.source} → {edge.target}
            </DialogDescription>
          )}
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)}>
            <Stack gap="lg">
              <FormField
                control={form.control}
                name="label"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Label</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="Enter edge label"
                        {...field}
                        disabled={readonly || loading}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="dataType"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Data Type</FormLabel>
                    <Select
                      value={field.value}
                      onValueChange={field.onChange}
                      disabled={readonly || loading}
                    >
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select data type" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        <SelectItem value="GRAPH_DATA">Graph Data</SelectItem>
                        <SelectItem value="GRAPH_REFERENCE">Graph Reference</SelectItem>
                      </SelectContent>
                    </Select>
                    <FormDescription>
                      <strong>Graph Data:</strong> Actual data flow between nodes (grey line)<br/>
                      <strong>Graph Reference:</strong> Reference to another graph (blue line)
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {!readonly && (
                <DialogFooter>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleClose}
                    disabled={loading}
                  >
                    Cancel
                  </Button>
                  <Button type="submit" disabled={loading}>
                    {loading ? 'Saving...' : 'Save Changes'}
                  </Button>
                </DialogFooter>
              )}
            </Stack>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}