# Layercake Tool - Storybook Documentation

This document provides comprehensive information about the Storybook setup for the Layercake Tool frontend.

## Overview

Storybook is integrated into the Layercake Tool frontend to provide:
- Interactive component documentation
- Visual regression testing
- Accessibility testing
- Component development isolation
- Design system documentation

## Getting Started

### Prerequisites
- Node.js 18+
- npm 8+

### Installation
```bash
cd frontend
npm install
```

### Running Storybook
```bash
# Start Storybook development server
npm run storybook

# Build Storybook for production
npm run build-storybook
```

### Running Tests
```bash
# Run all tests including visual regression
npm run test

# Run Storybook interaction tests
npm run test-storybook

# Run CI version of Storybook tests
npm run test-storybook:ci
```

## Project Structure

```
frontend/
├── .storybook/               # Storybook configuration
│   ├── main.ts              # Main configuration
│   ├── preview.ts           # Preview configuration
│   ├── manager.ts           # Manager configuration
│   └── test-runner.ts       # Test runner configuration
├── src/
│   ├── components/          # Component library
│   │   ├── ui/             # Basic UI components
│   │   ├── graph/          # Graph visualization components
│   │   ├── projects/       # Project management components
│   │   └── plans/          # Plan management components
│   └── stories/            # Story files and mock data
│       ├── sampleData/     # Sample data for stories
│       └── mockData/       # Mock data utilities
├── tests/
│   └── storybook-visual-regression.spec.ts  # Visual regression tests
└── storybook-static/       # Built Storybook (generated)
```

## Component Categories

### 1. UI Components (`src/components/ui/`)
Basic building blocks for the application:
- **Button**: Primary, secondary, and utility buttons
- **Card**: Content containers with various layouts
- **Input**: Form input fields with validation
- **Modal**: Overlay dialogs and modals
- **Loading**: Loading indicators and states
- **ErrorMessage**: Error display components

### 2. Graph Components (`src/components/graph/`)
Advanced graph visualization and interaction:
- **GraphVisualization**: Main D3.js-based graph display
- **GraphSettings**: Configuration panel for graph display options
- **GraphToolbar**: Interactive toolbar for graph operations
- **GraphMinimap**: Overview navigation for large graphs
- **GraphInspector**: Side panel for element inspection and editing

### 3. Form Components (`src/components/projects/`, `src/components/plans/`)
Application-specific forms:
- **ProjectForm**: Project creation and editing
- **PlanForm**: Plan configuration and management

## Story Organization

Stories are organized by component type and follow this naming convention:
```
{Category}/{Component}--{Variant}
```

Examples:
- `UI/Button--Primary`
- `Graph/GraphVisualization--Default`
- `Forms/ProjectForm--Default`

## Visual Regression Testing

### Setup
Visual regression tests are automatically run in CI/CD and compare screenshots of components against baseline images.

### Running Visual Tests
```bash
# Run visual regression tests locally
npm run test tests/storybook-visual-regression.spec.ts

# Update visual regression baselines
npm run test tests/storybook-visual-regression.spec.ts -- --update-snapshots
```

### Critical Stories for Visual Testing
The following stories are automatically tested for visual regressions:
- All UI component variants
- Key graph visualization states
- Form components with various states
- Layout components

## Accessibility Testing

### Automated Accessibility Checks
- **addon-a11y**: Provides real-time accessibility feedback
- **Test runner**: Includes accessibility assertions
- **Visual regression**: Checks for proper heading structure and ARIA labels

### Manual Accessibility Testing
1. Navigate between components using keyboard only
2. Test with screen readers
3. Verify color contrast ratios
4. Check focus management

## CI/CD Integration

### GitHub Actions Workflows

#### 1. Frontend CI/CD (`.github/workflows/frontend-ci.yml`)
Runs on every push and pull request:
- Linting and type checking
- Build verification
- Playwright tests
- Storybook build and test
- Visual regression testing

#### 2. Visual Regression Update (`.github/workflows/visual-regression-update.yml`)
Manual workflow to update visual regression baselines:
- Triggered manually from GitHub Actions
- Updates screenshot baselines
- Commits changes back to repository

### Artifacts
CI/CD generates the following artifacts:
- `playwright-results`: Test results and screenshots
- `storybook-build`: Built Storybook static files
- `visual-regression-results`: Visual regression test results

## Configuration

### Storybook Configuration Files

#### `.storybook/main.ts`
- Story file patterns
- Addon configuration
- Framework settings
- TypeScript configuration

#### `.storybook/preview.ts`
- Global parameters
- Default backgrounds and viewports
- Accessibility configuration
- Theme settings

#### `.storybook/manager.ts`
- UI customization
- Custom theme
- Panel configuration

#### `.storybook/test-runner.ts`
- Test setup and teardown
- Custom assertions
- Accessibility checks

### Viewport Configuration
Pre-configured viewports for responsive testing:
- **Mobile**: 375x667px
- **Tablet**: 768x1024px
- **Desktop**: 1440x900px
- **Responsive**: 100% width/height

### Theme Configuration
Support for light and dark themes:
- Light theme (default)
- Dark theme
- Custom color schemes

## Development Workflow

### Adding New Components
1. Create component in appropriate directory
2. Add TypeScript interfaces
3. Create `.stories.ts` file
4. Include in visual regression tests if critical
5. Update documentation

### Story Development Best Practices
1. **Use realistic data**: Leverage `sampleData` utilities
2. **Cover edge cases**: Empty states, error states, loading states
3. **Test interactions**: Use `@storybook/addon-actions`
4. **Document props**: Use JSDoc comments
5. **Accessibility**: Include ARIA labels and proper semantics

### Testing Strategy
1. **Unit Tests**: Component logic and behavior
2. **Integration Tests**: Component interactions
3. **Visual Regression**: UI consistency
4. **Accessibility**: WCAG compliance
5. **Performance**: Large graph handling

## Troubleshooting

### Common Issues

#### Storybook Won't Start
- Check Node.js version (18+ required)
- Clear `node_modules` and `package-lock.json`
- Verify port 6006 is available

#### Visual Regression Failures
- Check if intentional UI changes were made
- Update baselines using update workflow
- Verify consistent test environment

#### Missing Stories
- Check file naming conventions
- Verify story exports
- Check `.storybook/main.ts` patterns

### Performance Optimization
- Use `lazy` loading for large datasets
- Implement virtualization for large lists
- Optimize D3.js rendering for graph components

## Contributing

### Pull Request Process
1. Add stories for new components
2. Update visual regression baselines if needed
3. Ensure accessibility compliance
4. Add documentation for new features

### Code Style
- Follow TypeScript best practices
- Use consistent naming conventions
- Add JSDoc comments for public APIs
- Follow component composition patterns

## Resources

- [Storybook Documentation](https://storybook.js.org/docs)
- [Playwright Testing](https://playwright.dev/)
- [Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Component Design Patterns](https://reactpatterns.com/)

---

For more information about the Layercake Tool architecture, see the main [DESIGN.md](../DESIGN.md) file.