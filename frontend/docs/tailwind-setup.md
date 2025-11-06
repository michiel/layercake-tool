# Tailwind CSS Setup

## Overview

The Layercake frontend uses **Tailwind CSS v4** for styling. This document explains the configuration and theming approach used in the project.

## Version

- **Tailwind CSS**: v4.1.16
- **@tailwindcss/postcss**: v4.1.16 (PostCSS plugin)

## Why Tailwind v4?

Tailwind v4 represents a major architectural shift from v3:

- Uses a new CSS-first configuration approach via `@import` and `@theme`
- Removes the traditional JavaScript config file dependency
- Improved performance through native CSS parsing
- Better integration with modern build tools

## Configuration Files

### PostCSS Configuration

**File**: `postcss.config.js`

```javascript
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
}
```

**Important**: Tailwind v4 requires `@tailwindcss/postcss` plugin, not the old `tailwindcss` PostCSS plugin. Attempting to use `tailwindcss` directly will result in an error.

### Tailwind Configuration

**File**: `tailwind.config.js`

This file is still used for:
- Extended colour palettes (slate, gray, zinc, cyan, etc.)
- Border radius utilities (`--radius`)
- Dark mode configuration (`darkMode: ["class"]`)
- Content paths for utility class scanning

**Note**: The v3-style JavaScript config is maintained for backwards compatibility with the extended colour palette, but core theme colours are defined in CSS.

## CSS Configuration

**File**: `src/index.css`

The main CSS entry point uses Tailwind v4's CSS-first approach:

```css
@import "tailwindcss";

/* Define CSS variables for theming */
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --primary: 187 100% 42%;
  /* ... other theme variables */
}

.dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
  /* ... dark mode overrides */
}

/* Define Tailwind theme colours that reference the CSS variables */
@theme inline {
  --color-background: hsl(var(--background));
  --color-foreground: hsl(var(--foreground));
  --color-primary: hsl(var(--primary));
  /* ... other colour mappings */
}
```

### Key Components

1. **`@import "tailwindcss"`**: Imports Tailwind v4's base styles and utilities
2. **CSS Variables** (`:root` and `.dark`): Define theme values using HSL colour space (H S% L% format)
3. **`@theme inline`**: Maps CSS variables to Tailwind utility classes (e.g., `bg-background`, `text-foreground`)

## Theming System

### shadcn/ui Colour Variables

The project uses shadcn/ui's theming system with the following colour tokens:

- `background` / `foreground`: Page background and primary text
- `card` / `card-foreground`: Card backgrounds and card text
- `popover` / `popover-foreground`: Popover/dropdown backgrounds and text
- `primary` / `primary-foreground`: Primary actions and button text
- `secondary` / `secondary-foreground`: Secondary actions and text
- `muted` / `muted-foreground`: Muted backgrounds and subtle text
- `accent` / `accent-foreground`: Accent elements and text
- `destructive` / `destructive-foreground`: Destructive actions and text
- `border`: Border colours
- `input`: Form input borders
- `ring`: Focus ring colours

### Dark Mode

Dark mode is implemented using:

1. **next-themes**: Manages theme state and applies `.dark` class to `<html>` element
2. **CSS Variable Overrides**: `.dark` class defines alternate colour values
3. **Class-based Toggle**: `darkMode: ["class"]` in `tailwind.config.js`

### How It Works

1. **Define semantic colours** in `:root` and `.dark` using HSL values (without `hsl()` wrapper)
2. **Map to Tailwind colours** in `@theme inline` using `hsl(var(--variable))` syntax
3. **Use utility classes** in components: `bg-background`, `text-foreground`, `border-border`, etc.
4. **Theme switching** automatically updates all colours when `.dark` class is toggled

## Usage Examples

### Component Styling

```tsx
// Background and text
<div className="bg-background text-foreground">
  Content
</div>

// Card with proper contrast
<div className="bg-card text-card-foreground border border-border">
  Card content
</div>

// Primary button
<button className="bg-primary text-primary-foreground hover:bg-primary/90">
  Click me
</button>

// Muted text
<p className="text-muted-foreground">
  Subtitle or helper text
</p>
```

### Custom Colours

The extended colour palette from `tailwind.config.js` is still available:

```tsx
<div className="bg-cyan-500 text-slate-900">
  Custom coloured element
</div>
```

## Migration from Tailwind v3

If migrating from Tailwind v3:

1. **Replace** `@tailwind base/components/utilities` directives with `@import "tailwindcss"`
2. **Remove** `@layer` wrappers around CSS variable definitions
3. **Add** `@theme inline` block to map CSS variables to Tailwind colours
4. **Update** PostCSS config to use `@tailwindcss/postcss` instead of `tailwindcss`
5. **Keep** the JavaScript config file for extended colours and other non-colour theme values

## Common Issues

### Utility Classes Not Generated

**Problem**: Classes like `bg-background` or `text-foreground` are not being generated.

**Solution**: Ensure you have both:
1. CSS variables defined in `:root`/`.dark`
2. Colour mappings in `@theme inline` block

### "Cannot apply unknown utility class" Error

**Problem**: Getting errors about unknown utility classes when using `@apply`.

**Solution**: Tailwind v4 with `@import` syntax requires `@theme inline` definitions. Don't use `@apply` with custom semantic colour names - use utility classes directly in HTML instead.

### PostCSS Plugin Error

**Problem**: "tailwindcss should be installed as @tailwindcss/postcss" error.

**Solution**: Use `'@tailwindcss/postcss': {}` in `postcss.config.js`, not `'tailwindcss': {}`.

## Performance

- Build output: ~128KB CSS (minified, before gzip)
- ~20KB gzipped
- Includes all generated utility classes, third-party component styles (react-flow, react-query-builder), and assistant-ui theme

## References

- [Tailwind CSS v4 Documentation](https://tailwindcss.com/docs)
- [shadcn/ui Theming](https://ui.shadcn.com/docs/theming)
- [next-themes Documentation](https://github.com/pacocoursey/next-themes)
