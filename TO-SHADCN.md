# Mantine to shadcn/ui Migration Plan

## Overview

This document outlines the plan to completely replace Mantine UI components with shadcn/ui in the frontend application. The migration will be performed incrementally to maintain application stability.

## Current Mantine Usage Analysis

### Packages to Remove
- `@mantine/core` (v8.3.5)
- `@mantine/dates` (v8.3.5)
- `@mantine/form` (v8.3.5)
- `@mantine/hooks` (v8.3.5)
- `@mantine/notifications` (v8.3.5)

### Files Using Mantine
Total of 63 TypeScript files import from `@mantine` packages.

### Core Component Inventory

#### Layout Components
- **AppShell** - Custom layout wrapper (no direct shadcn equivalent)
- **Container** - Layout container with size variants
- **Stack** - Vertical flex layout
- **Group** - Horizontal flex layout
- **Flex** - Flexible layout
- **Paper** - Card-like container with shadow
- **Card** - Content card with sections
- **Divider** - Horizontal/vertical divider
- **ScrollArea** - Custom scrollbar area
- **Center** - Centering container

#### Navigation Components
- **Button** - Primary interactive element
- **ActionIcon** - Icon-only button
- **Tooltip** - Hover tooltips
- **NavLink** - Navigation links
- **Menu** - Dropdown menus
- **Anchor** - Styled links
- **Breadcrumbs** - Navigation breadcrumbs

#### Form Components
- **TextInput** - Text input field
- **Textarea** - Multi-line text input
- **Select** - Dropdown select
- **Switch** - Toggle switch
- **Checkbox** - Checkbox input
- **Radio** - Radio button
- **FileButton** - File upload button
- **ColorPicker** - Colour picker
- **Slider** - Range slider

#### Feedback Components
- **Alert** - Alert/notification box
- **Badge** - Status badge
- **Loader** - Loading spinner
- **LoadingOverlay** - Full-screen loading
- **Modal** - Modal dialog
- **Notifications** - Toast notifications system
- **Progress** - Progress bar
- **HoverCard** - Hover popup card
- **Popover** - Popup container
- **Indicator** - Badge indicator

#### Data Display Components
- **Table** - Data table
- **Tabs** - Tabbed interface
- **Accordion** - Collapsible sections
- **Avatar** - User avatar
- **Timeline** - Timeline display
- **Blockquote** - Quoted text
- **List** - Styled lists

#### Typography Components
- **Text** - Styled text
- **Title** - Heading text
- **Mark** - Highlighted text

#### Special Components
- **Stepper** - Step-by-step wizard
- **SegmentedControl** - Segmented button group
- **ThemeIcon** - Icon with theme colours
- **Dialog** - Inline dialog

### Mantine Features Used
- **Theme System** - Custom theme configuration with shadcn-inspired colours
- **CSS Variables** - Custom CSS variable resolver
- **Form Hook** - `useForm()` for form state management
- **Hooks** - `useMantineColorScheme()` for theme switching
- **Notifications** - Global notification system

## shadcn/ui Setup

### Prerequisites
```bash
# Install dependencies
npm install tailwindcss@latest postcss@latest autoprefixer@latest
npm install class-variance-authority clsx tailwind-merge
npm install @radix-ui/react-slot
npm install lucide-react  # Icon library (alternative to @tabler/icons-react)
```

### Initialisation
```bash
# Initialise shadcn/ui (creates components.json and lib/utils.ts)
npx shadcn@latest init
```

**Configuration Options:**
- Style: Default
- Base colour: Slate
- CSS variables: Yes
- Tailwind CSS import alias: @
- Components import alias: @/components
- Utils import alias: @/lib/utils

### Install Required shadcn/ui Components
```bash
# Layout & Structure
npx shadcn@latest add card
npx shadcn@latest add separator
npx shadcn@latest add scroll-area

# Navigation
npx shadcn@latest add button
npx shadcn@latest add tooltip
npx shadcn@latest add dropdown-menu
npx shadcn@latest add breadcrumb
npx shadcn@latest add navigation-menu

# Forms
npx shadcn@latest add input
npx shadcn@latest add textarea
npx shadcn@latest add select
npx shadcn@latest add switch
npx shadcn@latest add checkbox
npx shadcn@latest add radio-group
npx shadcn@latest add slider
npx shadcn@latest add label
npx shadcn@latest add form  # Form components with react-hook-form

# Feedback
npx shadcn@latest add alert
npx shadcn@latest add badge
npx shadcn@latest add dialog
npx shadcn@latest add toast  # Notification system
npx shadcn@latest add popover
npx shadcn@latest add hover-card
npx shadcn@latest add progress
npx shadcn@latest add skeleton  # Loading states

# Data Display
npx shadcn@latest add table
npx shadcn@latest add tabs
npx shadcn@latest add accordion
npx shadcn@latest add avatar

# Other
npx shadcn@latest add sheet  # Slide-out panel
npx shadcn@latest add command  # Command palette (optional)
```

### Additional Libraries
```bash
# Form management (replaces @mantine/form)
npm install react-hook-form @hookform/resolvers zod

# Date handling (replaces @mantine/dates)
npm install date-fns
npx shadcn@latest add calendar
npx shadcn@latest add popover  # For date picker

# Toast notifications (replaces @mantine/notifications)
npm install sonner
```

## Migration Strategy

The migration will be performed in stages to minimise disruption and allow for testing at each stage.

### Stage 1: Setup & Infrastructure (Foundation) ✅ COMPLETED
**Goal:** Establish shadcn/ui infrastructure without breaking existing functionality
**Duration:** 1-2 days

#### Tasks:
1. ✅ Install Tailwind CSS and dependencies
2. ✅ Configure `tailwind.config.js` with custom colours from existing theme
3. ✅ Initialise shadcn/ui (`npx shadcn@latest init`)
4. ✅ Create colour palette mapping (Mantine colours → Tailwind colours)
5. ✅ Set up Tailwind in `main.tsx` (Mantine CSS kept temporarily)
6. ✅ Install all required shadcn/ui components (26 components)
7. ✅ Create utility components and wrappers as needed

#### Success Criteria:
- [x] Tailwind CSS compiles successfully
- [x] shadcn/ui components are available in `src/components/ui/`
- [x] Application still runs with Mantine
- [x] No styling conflicts
- [x] TypeScript type-checking passes

#### Components Installed:
- accordion, alert, avatar, badge, breadcrumb, button, card, checkbox
- dialog, dropdown-menu, form, hover-card, input, label, popover, progress
- scroll-area, select, separator, slider, sonner, switch, table, tabs, textarea, tooltip

#### Dependencies Added:
- Tailwind CSS v4.1.16, PostCSS, Autoprefixer
- shadcn/ui utilities: class-variance-authority, clsx, tailwind-merge
- React Hook Form v7.66.0, Zod v4.1.12
- Sonner v2.0.7 (toast notifications)
- next-themes v0.4.6 (dark mode)
- lucide-react v0.552.0 (icons)

---

### Stage 2: Core Layout Components (Structure) 🚧 IN PROGRESS
**Goal:** Replace Mantine layout components with custom Tailwind equivalents
**Duration:** 2-3 days

#### Components Created: ✅
```
src/components/layout-primitives/
  ├── Stack.tsx            # Vertical flex layout (✅ Created)
  ├── Group.tsx            # Horizontal flex layout (✅ Created)
  ├── Flex.tsx             # Generic flex container (✅ Created)
  ├── Container.tsx        # Max-width container (✅ Created)
  ├── Paper.tsx            # Card-like container (✅ Created)
  ├── Center.tsx           # Centering container (✅ Created)
  └── index.ts             # Barrel export (✅ Created)

src/components/ui/
  └── spinner.tsx          # Loading spinner with lucide-react (✅ Created)
```

#### Files to Update (Priority):
1. `src/App.tsx` - AppShell, layout structure (⏳ Pending)
2. `src/main.tsx` - Remove MantineProvider, add Tailwind (⏳ Pending)
3. `src/components/layout/TopBar.tsx` (⏳ Pending)
4. `src/components/layout/PageContainer.tsx` (⏳ Pending)

#### Migration Pattern:
```tsx
// Before (Mantine)
import { Stack, Group, Container } from '@mantine/core'

<Stack gap="md">
  <Group justify="space-between">
    <Container size="lg">
      {children}
    </Container>
  </Group>
</Stack>

// After (shadcn/Tailwind)
import { cn } from '@/lib/utils'

<div className="flex flex-col gap-4">
  <div className="flex items-center justify-between">
    <div className={cn("container max-w-4xl")}>
      {children}
    </div>
  </div>
</div>
```

#### Success Criteria:
- [ ] Application layout renders correctly
- [ ] Navigation sidebar works
- [ ] Page containers have correct max-widths
- [ ] Spacing is consistent

---

### Stage 3: Navigation & Interactive Components
**Goal:** Replace Mantine navigation and button components
**Duration:** 2-3 days

#### Components to Migrate:
- Button → shadcn Button
- ActionIcon → Button with icon variant
- Tooltip → shadcn Tooltip
- Menu → shadcn DropdownMenu
- Modal → shadcn Dialog
- Anchor → Custom Link component

#### Files to Update (Priority):
1. All navigation buttons in `src/App.tsx` (sidebar)
2. `src/components/common/Breadcrumbs.tsx`
3. `src/components/editors/PlanVisualEditor/components/ContextMenu.tsx`
4. `src/components/editors/PlanVisualEditor/components/AdvancedToolbar.tsx`
5. `src/components/editors/PlanVisualEditor/components/NodeToolbar.tsx`
6. All files using Modal (19 files)

#### Migration Pattern:
```tsx
// Before (Mantine)
import { Button, ActionIcon, Tooltip, Menu, Modal } from '@mantine/core'

<Button variant="filled" leftSection={<Icon />} onClick={...}>
  Click Me
</Button>

<Tooltip label="Info">
  <ActionIcon variant="subtle">
    <Icon />
  </ActionIcon>
</Tooltip>

<Modal opened={opened} onClose={...}>
  <Modal.Body>Content</Modal.Body>
</Modal>

// After (shadcn)
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Dialog, DialogContent, DialogHeader } from '@/components/ui/dialog'

<Button variant="default" className="gap-2" onClick={...}>
  <Icon className="h-4 w-4" />
  Click Me
</Button>

<TooltipProvider>
  <Tooltip>
    <TooltipTrigger asChild>
      <Button variant="ghost" size="icon">
        <Icon className="h-4 w-4" />
      </Button>
    </TooltipTrigger>
    <TooltipContent>Info</TooltipContent>
  </Tooltip>
</TooltipProvider>

<Dialog open={opened} onOpenChange={...}>
  <DialogContent>Content</DialogContent>
</Dialog>
```

#### Success Criteria:
- [ ] All buttons render and function correctly
- [ ] Tooltips appear on hover
- [ ] Modals open and close properly
- [ ] Menus work correctly
- [ ] Navigation remains functional

---

### Stage 4: Form Components
**Goal:** Replace Mantine form components with shadcn forms
**Duration:** 3-4 days

#### Form Management Migration:
- Replace `@mantine/form` with `react-hook-form` + `zod`
- Use shadcn's Form component pattern

#### Components to Migrate:
- TextInput → shadcn Input
- Textarea → shadcn Textarea
- Select → shadcn Select
- Switch → shadcn Switch
- Checkbox → shadcn Checkbox
- FileButton → Custom file input

#### Files to Update (Priority):
1. `src/components/project/CreateProjectModal.tsx` (uses useForm)
2. `src/components/datasources/DataSourceUploader.tsx` (uses useForm, complex)
3. `src/components/datasources/DataSourceEditor.tsx` (uses useForm, complex)
4. `src/components/library/LibrarySourcesPage.tsx` (uses useForm)
5. All form components in `src/components/editors/PlanVisualEditor/forms/` (16 files)
6. `src/components/editors/PlanVisualEditor/EdgeConfigDialog.tsx` (uses useForm)
7. `src/components/graphs/NodePropertiesForm.tsx`

#### Migration Pattern:
```tsx
// Before (Mantine)
import { TextInput, Select, Switch } from '@mantine/core'
import { useForm } from '@mantine/form'

const form = useForm({
  initialValues: { name: '', type: 'option1', enabled: false },
  validate: {
    name: (value) => value.length < 2 ? 'Too short' : null
  }
})

<form onSubmit={form.onSubmit(handleSubmit)}>
  <TextInput label="Name" {...form.getInputProps('name')} />
  <Select label="Type" data={options} {...form.getInputProps('type')} />
  <Switch label="Enabled" {...form.getInputProps('enabled')} />
</form>

// After (shadcn + react-hook-form)
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import { Form, FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'

const formSchema = z.object({
  name: z.string().min(2, 'Too short'),
  type: z.string(),
  enabled: z.boolean()
})

const form = useForm<z.infer<typeof formSchema>>({
  resolver: zodResolver(formSchema),
  defaultValues: { name: '', type: 'option1', enabled: false }
})

<Form {...form}>
  <form onSubmit={form.handleSubmit(handleSubmit)}>
    <FormField
      control={form.control}
      name="name"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Name</FormLabel>
          <FormControl>
            <Input {...field} />
          </FormControl>
        </FormItem>
      )}
    />
    {/* Similar patterns for Select and Switch */}
  </form>
</Form>
```

#### Success Criteria:
- [ ] All forms submit correctly
- [ ] Validation works as expected
- [ ] Error messages display properly
- [ ] Form state management is functional

---

### Stage 5: Feedback Components
**Goal:** Replace Mantine feedback components
**Duration:** 2-3 days

#### Components to Migrate:
- Alert → shadcn Alert
- Badge → shadcn Badge
- Loader → Custom spinner or shadcn Skeleton
- LoadingOverlay → Custom overlay with Spinner
- Notifications → sonner toast library
- Progress → shadcn Progress

#### Notifications System:
Replace `@mantine/notifications` with `sonner`:

```tsx
// Before (Mantine)
import { notifications } from '@mantine/notifications'

notifications.show({
  title: 'Success',
  message: 'Operation completed',
  color: 'green'
})

// After (sonner)
import { toast } from 'sonner'

toast.success('Operation completed', {
  description: 'Success'
})
```

#### Files to Update:
1. `src/utils/notifications.ts` - Notification utility
2. `src/main.tsx` - Add Toaster component
3. All files using Alert (35+ files)
4. All files using Badge (25+ files)
5. All files using Loader (15+ files)

#### Success Criteria:
- [ ] Toast notifications appear correctly
- [ ] Alerts display with correct styling
- [ ] Badges render properly
- [ ] Loading states work

---

### Stage 6: Data Display Components
**Goal:** Replace Mantine data display components
**Duration:** 2-3 days

#### Components to Migrate:
- Table → shadcn Table
- Tabs → shadcn Tabs
- Accordion → shadcn Accordion
- ScrollArea → shadcn ScrollArea
- HoverCard → shadcn HoverCard
- Popover → shadcn Popover
- Avatar → shadcn Avatar

#### Files to Update (Priority):
1. `src/components/visualization/DataPreview.tsx` (Table, ScrollArea)
2. `src/pages/ChatLogsPage.tsx` (Table)
3. `src/components/editors/GraphSpreadsheetEditor/GraphSpreadsheetEditor.tsx` (Tabs, Table)
4. `src/components/visualization/GraphPreviewDialog.tsx` (Tabs)
5. `src/components/graphs/PropertiesAndLayersPanel.tsx` (Accordion, Slider)
6. `src/components/collaboration/UserPresenceIndicator.tsx` (Avatar, HoverCard)
7. `src/components/graphs/LayerListItem.tsx` (Popover, ColorPicker)

#### Migration Pattern:
```tsx
// Before (Mantine)
import { Tabs, Table } from '@mantine/core'

<Tabs defaultValue="first">
  <Tabs.List>
    <Tabs.Tab value="first">First Tab</Tabs.Tab>
    <Tabs.Tab value="second">Second Tab</Tabs.Tab>
  </Tabs.List>
  <Tabs.Panel value="first">Content 1</Tabs.Panel>
  <Tabs.Panel value="second">Content 2</Tabs.Panel>
</Tabs>

// After (shadcn)
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

<Tabs defaultValue="first">
  <TabsList>
    <TabsTrigger value="first">First Tab</TabsTrigger>
    <TabsTrigger value="second">Second Tab</TabsTrigger>
  </TabsList>
  <TabsContent value="first">Content 1</TabsContent>
  <TabsContent value="second">Content 2</TabsContent>
</Tabs>
```

#### Success Criteria:
- [ ] Tables display data correctly
- [ ] Tabs switch properly
- [ ] Accordions expand/collapse
- [ ] Scroll areas work smoothly

---

### Stage 7: Typography & Misc Components
**Goal:** Replace remaining Mantine components
**Duration:** 2 days

#### Components to Migrate:
- Text → Standard HTML tags with Tailwind classes
- Title → h1-h6 tags with Tailwind
- Mark → Custom styled span
- Divider → shadcn Separator
- Paper → Card or custom div
- ColorPicker → Custom component or third-party library
- Slider → shadcn Slider

#### Typography Pattern:
```tsx
// Before (Mantine)
import { Title, Text } from '@mantine/core'

<Title order={1}>Heading</Title>
<Title order={2}>Subheading</Title>
<Text size="lg" fw={500} c="dimmed">Description</Text>

// After (Tailwind)
<h1 className="text-4xl font-semibold">Heading</h1>
<h2 className="text-2xl font-semibold">Subheading</h2>
<p className="text-lg font-medium text-muted-foreground">Description</p>
```

#### Files to Update:
- All 63 files using Text component
- All files using Title component
- `src/components/graphs/LayerListItem.tsx` (ColorPicker)
- `src/components/graphs/PropertiesAndLayersPanel.tsx` (Slider)

#### Success Criteria:
- [ ] Typography is consistent
- [ ] Headings have correct sizing
- [ ] Colour picker functionality works
- [ ] Sliders function correctly

---

### Stage 8: Theme & Dark Mode
**Goal:** Implement dark mode with shadcn/ui patterns
**Duration:** 2-3 days

#### Tasks:
1. Set up `next-themes` or custom theme provider
2. Migrate `useMantineColorScheme()` usage
3. Update CSS variables for dark mode
4. Implement theme toggle component
5. Test all components in both light and dark modes

#### Files to Update:
1. `src/components/layout/TopBar.tsx` (uses `useMantineColorScheme`)
2. `src/themes/shadcn-cyan/theme.ts` (remove, migrate to Tailwind config)
3. `src/themes/shadcn-cyan/cssVariableResolver.ts` (remove)

#### Theme Setup:
```tsx
// Install next-themes
npm install next-themes

// Create ThemeProvider wrapper
// src/components/theme-provider.tsx
import { ThemeProvider as NextThemesProvider } from "next-themes"

export function ThemeProvider({ children, ...props }) {
  return <NextThemesProvider {...props}>{children}</NextThemesProvider>
}

// In main.tsx
import { ThemeProvider } from './components/theme-provider'

root.render(
  <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
    <App />
  </ThemeProvider>
)
```

#### Success Criteria:
- [ ] Theme switching works
- [ ] All components render correctly in both modes
- [ ] Colour variables are correct
- [ ] No visual regressions

---

### Stage 9: Testing & Refinement
**Goal:** Ensure all functionality works correctly
**Duration:** 2-3 days

#### Testing Checklist:
- [ ] All pages load without errors
- [ ] All forms submit correctly
- [ ] All modals open and close
- [ ] All tooltips and popovers work
- [ ] Navigation is functional
- [ ] Data tables display correctly
- [ ] File uploads work
- [ ] Notifications appear
- [ ] Theme switching works
- [ ] Responsive design works on different screen sizes
- [ ] No console errors
- [ ] No styling conflicts

#### Performance Checks:
- [ ] Bundle size comparison (before/after)
- [ ] Page load times
- [ ] Interaction responsiveness

#### Refinement Tasks:
1. Fix any visual inconsistencies
2. Adjust spacing and sizing to match original design
3. Optimise custom components
4. Update any missing animations/transitions
5. Document any breaking changes or new patterns

#### Success Criteria:
- [ ] All tests pass
- [ ] No regressions in functionality
- [ ] Visual consistency maintained
- [ ] Performance is acceptable

---

### Stage 10: Cleanup & Documentation
**Goal:** Remove Mantine dependencies and document changes
**Duration:** 1 day

#### Tasks:
1. Remove Mantine packages from `package.json`:
   ```bash
   npm uninstall @mantine/core @mantine/dates @mantine/form @mantine/hooks @mantine/notifications
   ```

2. Remove Mantine CSS imports from `main.tsx`:
   ```tsx
   // Remove these lines:
   // import '@mantine/core/styles.css'
   // import '@mantine/dates/styles.css'
   // import '@mantine/notifications/styles.css'
   ```

3. Delete Mantine theme files:
   ```bash
   rm -rf src/themes/shadcn-cyan/
   ```

4. Update documentation:
   - Create `docs/MIGRATION-NOTES.md`
   - Document component mapping
   - List breaking changes
   - Update component usage examples

5. Update developer setup instructions in README

#### Documentation Deliverables:
- Component migration mapping table
- New form patterns documentation
- Theme customisation guide
- Common pitfalls and solutions

#### Success Criteria:
- [ ] No Mantine dependencies remain
- [ ] No Mantine imports in codebase
- [ ] Documentation is complete
- [ ] Team is trained on new patterns

---

## Component Mapping Reference

| Mantine Component | shadcn/ui Equivalent | Notes |
|-------------------|---------------------|-------|
| AppShell | Custom Layout | Create custom layout with Tailwind |
| Container | div with container class | Use `className="container max-w-*"` |
| Stack | div with flex-col | `className="flex flex-col gap-*"` |
| Group | div with flex-row | `className="flex items-center gap-*"` |
| Button | Button | Direct replacement |
| ActionIcon | Button (icon variant) | `<Button variant="ghost" size="icon">` |
| TextInput | Input | Direct replacement with FormField |
| Textarea | Textarea | Direct replacement with FormField |
| Select | Select | Different API, requires multiple sub-components |
| Switch | Switch | Direct replacement |
| Checkbox | Checkbox | Direct replacement |
| Modal | Dialog | Different API, DialogContent, DialogHeader, etc. |
| Menu | DropdownMenu | Different API, more verbose |
| Tooltip | Tooltip | Requires provider, more verbose |
| Alert | Alert | Direct replacement |
| Badge | Badge | Direct replacement |
| Tabs | Tabs | Different API but similar |
| Table | Table | Direct replacement |
| Accordion | Accordion | Direct replacement |
| Card | Card | Direct replacement |
| Text | p/span with classes | Use Tailwind typography |
| Title | h1-h6 with classes | Use Tailwind typography |
| Loader | Custom Spinner | Create custom loading component |
| notifications | toast (sonner) | Different API |
| useForm | useForm (react-hook-form) | Different API, use zod for validation |
| useMantineColorScheme | useTheme (next-themes) | Different API |

## Custom Components to Create

Some Mantine components don't have direct shadcn equivalents and will need custom implementations:

1. **LoadingOverlay** - Full-screen loading state
2. **Stepper** - Step-by-step wizard
3. **SegmentedControl** - Segmented button group
4. **Timeline** - Timeline display
5. **ColorPicker** - Colour picker (consider react-colorful)
6. **FileButton** - Styled file input
7. **Indicator** - Badge indicator on avatars

## Colour Palette Migration

The existing Mantine theme uses shadcn-inspired colours. Map these to Tailwind config:

```js
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        // Keep existing colour names from Mantine theme
        cyan: { /* existing cyan palette */ },
        slate: { /* existing slate palette */ },
        // ... etc
      }
    }
  }
}
```

## Form Validation Migration

Replace Mantine's inline validation with Zod schemas:

```tsx
// Before (Mantine)
validate: {
  name: (value) => value.length < 2 ? 'Too short' : null
}

// After (Zod)
const schema = z.object({
  name: z.string().min(2, 'Too short')
})
```

## Icon Library Consideration

The project currently uses `@tabler/icons-react`. You can:
1. Keep using Tabler icons (recommended for minimal changes)
2. Migrate to `lucide-react` (shadcn's default)

If keeping Tabler icons, no changes needed. If migrating to lucide:
- Find equivalent icons (most have 1:1 mapping)
- Update all icon imports
- Adjust icon sizes (lucide uses different sizing)

## Risk Mitigation

### Potential Issues:
1. **Form complexity** - react-hook-form has a different API
2. **Custom styling** - Some Mantine components have extensive custom styling
3. **Type safety** - Zod schemas may require type adjustments
4. **Bundle size** - Initially might increase, optimise later

### Mitigation Strategies:
1. Create wrapper components for complex forms during transition
2. Document styling patterns for custom components
3. Use proper TypeScript types from the start
4. Monitor bundle size at each stage
5. Keep Mantine and shadcn side-by-side temporarily during critical stages

## Timeline Estimate

| Stage | Duration | Dependencies |
|-------|----------|--------------|
| Stage 1: Setup | 1-2 days | None |
| Stage 2: Layout | 2-3 days | Stage 1 |
| Stage 3: Navigation | 2-3 days | Stage 1, 2 |
| Stage 4: Forms | 3-4 days | Stage 1, 2 |
| Stage 5: Feedback | 2-3 days | Stage 1 |
| Stage 6: Data Display | 2-3 days | Stage 1 |
| Stage 7: Typography | 2 days | Stage 1 |
| Stage 8: Theme | 2-3 days | All previous |
| Stage 9: Testing | 2-3 days | All previous |
| Stage 10: Cleanup | 1 day | All previous |

**Total Estimated Time:** 19-27 days

## Success Metrics

- [ ] All Mantine packages removed
- [ ] Zero Mantine imports in codebase
- [ ] All existing functionality works
- [ ] No visual regressions
- [ ] Dark mode works correctly
- [ ] Forms validate correctly
- [ ] Performance is maintained or improved
- [ ] Bundle size is reasonable
- [ ] Developer experience is good
- [ ] Documentation is complete

## Notes

- This is a significant refactor requiring careful testing
- Consider feature freeze during critical migration stages
- Incremental migration reduces risk but takes longer
- Some components may need temporary wrapper components
- Keep git commits small and focused for easy rollback

## Additional Resources

- [shadcn/ui Documentation](https://ui.shadcn.com/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [react-hook-form Documentation](https://react-hook-form.com/)
- [Zod Documentation](https://zod.dev/)
- [next-themes Documentation](https://github.com/pacocoursey/next-themes)
- [sonner Documentation](https://sonner.emilkowal.ski/)
