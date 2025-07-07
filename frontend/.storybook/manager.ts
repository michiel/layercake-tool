import { addons } from '@storybook/addons';
import { create } from '@storybook/theming';

// Create custom theme
const customTheme = create({
  base: 'light',
  brandTitle: 'Layercake Tool',
  brandUrl: 'https://github.com/michiel/layercake-tool',
  brandImage: undefined,
  brandTarget: '_self',
  
  // UI colors
  colorPrimary: '#3b82f6',
  colorSecondary: '#10b981',
  
  // Typography
  fontBase: '"Nunito Sans", -apple-system, ".SFNSText-Regular", "San Francisco", BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Helvetica, Arial, sans-serif',
  fontCode: 'Monaco, Consolas, "Lucida Console", monospace',
});

// Configure Storybook addons
addons.setConfig({
  theme: customTheme,
  panelPosition: 'bottom',
  enableShortcuts: true,
  showNav: true,
  showPanel: true,
  showRoots: true,
  sidebar: {
    showRoots: true,
    collapsedRoots: [],
  },
});