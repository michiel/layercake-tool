import { test, expect } from '@playwright/test';

const STORYBOOK_URL = 'http://localhost:6006';

// List of critical stories to test visually
const CRITICAL_STORIES = [
  'ui-button--primary',
  'ui-button--secondary',
  'ui-card--default',
  'ui-input--default',
  'ui-modal--default',
  'graph-graphvisualization--default',
  'graph-graphvisualization--small-graph',
  'graph-enhanced-graphsettings--default-settings',
  'graph-enhanced-graphtoolbar--default-toolbar',
  'graph-enhanced-graphminimap--default-minimap',
  'graph-enhanced-graphinspector--default-inspector',
  'forms-projectform--default',
  'forms-planform--default',
];

test.describe('Storybook Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to Storybook
    await page.goto(STORYBOOK_URL);
    
    // Wait for Storybook to load
    await page.waitForSelector('#storybook-root');
    await page.waitForLoadState('networkidle');
  });

  CRITICAL_STORIES.forEach((storyId) => {
    test(`Visual regression: ${storyId}`, async ({ page }) => {
      // Navigate to specific story
      await page.goto(`${STORYBOOK_URL}?path=/story/${storyId}`);
      
      // Wait for story to load
      await page.waitForSelector('#storybook-root');
      await page.waitForLoadState('networkidle');
      
      // Wait for any animations to complete
      await page.waitForTimeout(1000);
      
      // Take screenshot of the story
      const storyElement = page.locator('#storybook-root');
      await expect(storyElement).toBeVisible();
      
      // Visual regression test
      await expect(storyElement).toHaveScreenshot(`${storyId}.png`, {
        // Ensure consistent screenshots
        animations: 'disabled',
        // Allow for slight differences in rendering
        threshold: 0.3,
        // Mask dynamic content
        mask: [
          // Mask any timestamps or dynamic content
          page.locator('[data-testid="timestamp"]'),
          page.locator('.dynamic-content')
        ],
      });
    });
  });

  test('Storybook navigation works correctly', async ({ page }) => {
    // Test that we can navigate between stories
    await page.click('[data-testid="ui-button--primary"]');
    await page.waitForLoadState('networkidle');
    
    await page.click('[data-testid="ui-card--default"]');
    await page.waitForLoadState('networkidle');
    
    // Verify navigation works
    await expect(page.locator('#storybook-root')).toBeVisible();
  });

  test('Storybook accessibility compliance', async ({ page }) => {
    // Test a few key stories for accessibility
    const accessibilityStories = [
      'ui-button--primary',
      'ui-input--default',
      'ui-modal--default',
    ];

    for (const storyId of accessibilityStories) {
      await page.goto(`${STORYBOOK_URL}?path=/story/${storyId}`);
      await page.waitForLoadState('networkidle');
      
      // Basic accessibility checks
      const storyElement = page.locator('#storybook-root');
      await expect(storyElement).toBeVisible();
      
      // Check for proper heading structure
      const headings = await page.locator('h1, h2, h3, h4, h5, h6').count();
      if (headings > 0) {
        // Verify heading hierarchy if headings exist
        const firstHeading = await page.locator('h1, h2, h3, h4, h5, h6').first();
        await expect(firstHeading).toBeVisible();
      }
      
      // Check for proper button accessibility
      const buttons = await page.locator('button').count();
      if (buttons > 0) {
        const button = page.locator('button').first();
        await expect(button).toBeVisible();
        
        // Check if button has accessible name
        const hasAccessibleName = await button.evaluate((el) => {
          return el.getAttribute('aria-label') || el.textContent?.trim() || el.getAttribute('title');
        });
        expect(hasAccessibleName).toBeTruthy();
      }
    }
  });
});