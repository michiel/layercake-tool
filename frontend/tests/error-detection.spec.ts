import { test, expect } from '@playwright/test';

test.describe('Frontend Error Detection', () => {
  let consoleErrors: string[] = [];
  let jsErrors: string[] = [];

  test.beforeEach(async ({ page }) => {
    // Capture console errors
    consoleErrors = [];
    jsErrors = [];
    
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Capture JavaScript errors
    page.on('pageerror', (error) => {
      jsErrors.push(error.message);
    });
  });

  test('should load plans page without JavaScript errors', async ({ page }) => {
    // Navigate to the plans page
    await page.goto('http://localhost:3001/projects/2/plans');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
    
    // Wait a bit more for any async components to load
    await page.waitForTimeout(2000);
    
    // Check for JavaScript errors
    if (jsErrors.length > 0) {
      console.log('JavaScript Errors Found:');
      jsErrors.forEach((error, index) => {
        console.log(`${index + 1}. ${error}`);
      });
    }
    
    if (consoleErrors.length > 0) {
      console.log('Console Errors Found:');
      consoleErrors.forEach((error, index) => {
        console.log(`${index + 1}. ${error}`);
      });
    }
    
    // Fail the test if there are JavaScript errors
    expect(jsErrors, `JavaScript errors found: ${jsErrors.join(', ')}`).toHaveLength(0);
    
    // Also check for critical console errors (syntax errors, import errors, etc.)
    const criticalErrors = consoleErrors.filter(error => 
      error.includes('SyntaxError') || 
      error.includes('does not provide an export') ||
      error.includes('Cannot resolve module') ||
      error.includes('Failed to fetch') ||
      error.includes('TypeError')
    );
    
    expect(criticalErrors, `Critical console errors found: ${criticalErrors.join(', ')}`).toHaveLength(0);
  });

  test('should navigate to individual plan page without errors', async ({ page }) => {
    // Start at plans page
    await page.goto('http://localhost:3001/projects/2/plans');
    await page.waitForLoadState('networkidle');
    
    // Wait for plans to load
    await page.waitForSelector('[data-testid="plan-card"], .plan-card, [title="View plan"]', { timeout: 10000 });
    
    // Click on the first view plan button or plan card
    const viewButton = page.locator('[title="View plan"]').first();
    const planCard = page.locator('.plan-card, [data-testid="plan-card"]').first();
    
    // Try to click view button first, fallback to plan card
    if (await viewButton.count() > 0) {
      await viewButton.click();
    } else if (await planCard.count() > 0) {
      await planCard.click();
    } else {
      // If no interactive elements found, this might indicate a loading issue
      console.log('No plan view buttons or cards found. Page content:');
      console.log(await page.content());
      throw new Error('No plan view elements found to click');
    }
    
    // Wait for navigation and page load
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Check the URL changed to individual plan view
    const currentUrl = page.url();
    expect(currentUrl).toMatch(/\/projects\/\d+\/plans\/\d+/);
    
    // Check for JavaScript errors on the plan page
    if (jsErrors.length > 0) {
      console.log('JavaScript Errors on Plan Page:');
      jsErrors.forEach((error, index) => {
        console.log(`${index + 1}. ${error}`);
      });
    }
    
    expect(jsErrors, `JavaScript errors on plan page: ${jsErrors.join(', ')}`).toHaveLength(0);
    
    const criticalErrors = consoleErrors.filter(error => 
      error.includes('SyntaxError') || 
      error.includes('does not provide an export') ||
      error.includes('Cannot resolve module')
    );
    
    expect(criticalErrors, `Critical errors on plan page: ${criticalErrors.join(', ')}`).toHaveLength(0);
  });

  test('should check all main routes for errors', async ({ page }) => {
    const routes = [
      'http://localhost:3001/',
      'http://localhost:3001/projects',
      'http://localhost:3001/projects/2/plans',
      'http://localhost:3001/projects/2/plans/2'
    ];

    for (const route of routes) {
      console.log(`Testing route: ${route}`);
      
      // Reset error arrays for each route
      consoleErrors = [];
      jsErrors = [];
      
      await page.goto(route);
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1500);
      
      // Log any errors found for this route
      if (jsErrors.length > 0 || consoleErrors.length > 0) {
        console.log(`Errors found on ${route}:`);
        if (jsErrors.length > 0) {
          console.log('JavaScript Errors:', jsErrors);
        }
        if (consoleErrors.length > 0) {
          console.log('Console Errors:', consoleErrors);
        }
      }
      
      // Check for critical errors
      const criticalErrors = [
        ...jsErrors,
        ...consoleErrors.filter(error => 
          error.includes('SyntaxError') || 
          error.includes('does not provide an export') ||
          error.includes('Cannot resolve module') ||
          error.includes('TypeError')
        )
      ];
      
      if (criticalErrors.length > 0) {
        throw new Error(`Critical errors found on ${route}: ${criticalErrors.join(', ')}`);
      }
    }
  });

  test.afterEach(async ({ page }) => {
    // Print summary of any errors found
    if (jsErrors.length > 0 || consoleErrors.length > 0) {
      console.log('\n=== Error Summary ===');
      console.log(`JavaScript Errors: ${jsErrors.length}`);
      console.log(`Console Errors: ${consoleErrors.length}`);
      console.log('=====================\n');
    }
  });
});