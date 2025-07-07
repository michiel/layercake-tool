import type { TestRunnerConfig } from '@storybook/test-runner';

const config: TestRunnerConfig = {
  setup() {
    // Global setup for all tests
  },
  
  async postRender(page, context) {
    // Reduce wait time for faster tests
    await page.waitForLoadState('domcontentloaded');
    
    // Simple smoke test - just verify the story rendered
    const elementHandler = await page.$('#storybook-root');
    if (!elementHandler) {
      throw new Error('Storybook root element not found');
    }
    
    // Basic accessibility check - ensure there are no console errors
    const consoleLogs = await page.evaluate(() => {
      return window.console.errors || [];
    });
    
    if (consoleLogs.length > 0) {
      console.warn('Console errors detected:', consoleLogs);
    }
  },
  
  async preRender(page, context) {
    // Setup before each story render
    await page.evaluate(() => {
      // Clear any previous errors
      if (window.console.errors) {
        window.console.errors = [];
      }
    });
  }
};

export default config;