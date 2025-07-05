import { test, expect } from '@playwright/test';
import { readFile } from 'fs/promises';
import { join } from 'path';

test.describe('Graph Components Build Verification', () => {
  
  test('should verify GraphSettings component exists and has correct structure', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/GraphSettings.tsx');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify component exports
      expect(content).toContain('export const GraphSettings');
      expect(content).toContain('export interface GraphLayoutSettings');
      expect(content).toContain('export interface GraphDisplaySettings');
      expect(content).toContain('export interface GraphFilterSettings');
      
      // Verify key functionality
      expect(content).toContain('Modal');
      expect(content).toContain('activeTab');
      expect(content).toContain('layoutSettings');
      expect(content).toContain('displaySettings');
      expect(content).toContain('filterSettings');
      
      console.log('✓ GraphSettings component structure verified');
    } catch (error) {
      throw new Error(`GraphSettings component verification failed: ${error}`);
    }
  });

  test('should verify GraphToolbar component exists and has correct structure', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/GraphToolbar.tsx');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify component exports
      expect(content).toContain('export const GraphToolbar');
      expect(content).toContain('export interface GraphToolbarProps');
      
      // Verify key functionality
      expect(content).toContain('zoomLevel');
      expect(content).toContain('searchQuery');
      expect(content).toContain('selectedNodeCount');
      expect(content).toContain('onZoomIn');
      expect(content).toContain('onZoomOut');
      
      console.log('✓ GraphToolbar component structure verified');
    } catch (error) {
      throw new Error(`GraphToolbar component verification failed: ${error}`);
    }
  });

  test('should verify GraphMinimap component exists and has correct structure', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/GraphMinimap.tsx');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify component exports
      expect(content).toContain('export const GraphMinimap');
      expect(content).toContain('export interface GraphMinimapProps');
      expect(content).toContain('export interface MinimapViewport');
      
      // Verify key functionality
      expect(content).toContain('canvas');
      expect(content).toContain('viewport');
      expect(content).toContain('nodes');
      expect(content).toContain('edges');
      expect(content).toContain('onViewportChange');
      
      console.log('✓ GraphMinimap component structure verified');
    } catch (error) {
      throw new Error(`GraphMinimap component verification failed: ${error}`);
    }
  });

  test('should verify GraphInspector component exists and has correct structure', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/GraphInspector.tsx');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify component exports
      expect(content).toContain('export const GraphInspector');
      expect(content).toContain('export interface GraphInspectorProps');
      expect(content).toContain('export interface NodeData');
      expect(content).toContain('export interface EdgeData');
      expect(content).toContain('export interface LayerData');
      
      // Verify key functionality
      expect(content).toContain('selectedNodes');
      expect(content).toContain('selectedEdges');
      expect(content).toContain('selectedLayers');
      expect(content).toContain('activeTab');
      expect(content).toContain('onNodeUpdate');
      expect(content).toContain('onEdgeUpdate');
      
      console.log('✓ GraphInspector component structure verified');
    } catch (error) {
      throw new Error(`GraphInspector component verification failed: ${error}`);
    }
  });

  test('should verify Storybook stories exist and have correct structure', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/EnhancedGraphControls.stories.ts');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify story imports
      expect(content).toContain('import type { Meta, StoryObj }');
      expect(content).toContain('GraphSettings');
      expect(content).toContain('GraphToolbar');
      expect(content).toContain('GraphMinimap');
      expect(content).toContain('GraphInspector');
      
      // Verify story exports
      expect(content).toContain('export const DefaultSettings');
      expect(content).toContain('export const DefaultToolbar');
      expect(content).toContain('export const DefaultMinimap');
      expect(content).toContain('export const DefaultInspector');
      
      // Verify story configurations
      expect(content).toContain('Graph/Enhanced/GraphSettings');
      expect(content).toContain('Graph/Enhanced/GraphToolbar');
      expect(content).toContain('Graph/Enhanced/GraphMinimap');
      expect(content).toContain('Graph/Enhanced/GraphInspector');
      
      console.log('✓ Storybook stories structure verified');
    } catch (error) {
      throw new Error(`Storybook stories verification failed: ${error}`);
    }
  });

  test('should verify component imports are correct', async () => {
    const components = [
      'GraphSettings.tsx',
      'GraphToolbar.tsx', 
      'GraphMinimap.tsx',
      'GraphInspector.tsx'
    ];
    
    for (const component of components) {
      const filePath = join(process.cwd(), 'src/components/graph', component);
      
      try {
        const content = await readFile(filePath, 'utf-8');
        
        // Verify React import
        expect(content).toContain('import React');
        
        // Verify UI component imports where applicable
        if (component !== 'GraphMinimap.tsx') {
          expect(content).toMatch(/import.*from ['"]\.\.\/ui\//);
        }
        
        console.log(`✓ ${component} imports verified`);
      } catch (error) {
        throw new Error(`Import verification failed for ${component}: ${error}`);
      }
    }
  });

  test('should verify TypeScript interfaces are properly typed', async () => {
    const filePath = join(process.cwd(), 'src/components/graph/GraphSettings.tsx');
    
    try {
      const content = await readFile(filePath, 'utf-8');
      
      // Verify interface definitions have proper typing
      expect(content).toContain('forceStrength: number');
      expect(content).toContain('enableZoom: boolean');
      expect(content).toContain('selectedLayers: string[]');
      expect(content).toContain('zoomExtent: [number, number]');
      expect(content).toContain('nodeColorScheme: \'layer\' | \'weight\' | \'custom\'');
      
      console.log('✓ TypeScript interfaces properly typed');
    } catch (error) {
      throw new Error(`TypeScript interface verification failed: ${error}`);
    }
  });

  test('should verify component props are comprehensive', async () => {
    const components = [
      { file: 'GraphSettings.tsx', props: ['layoutSettings', 'displaySettings', 'filterSettings', 'onLayoutChange', 'onDisplayChange', 'onFilterChange'] },
      { file: 'GraphToolbar.tsx', props: ['zoomLevel', 'onZoomIn', 'onZoomOut', 'searchQuery', 'onSearchChange', 'nodeCount', 'edgeCount'] },
      { file: 'GraphMinimap.tsx', props: ['nodes', 'edges', 'viewport', 'onViewportChange', 'graphBounds'] },
      { file: 'GraphInspector.tsx', props: ['selectedNodes', 'selectedEdges', 'selectedLayers', 'onNodeUpdate', 'onEdgeUpdate', 'onLayerUpdate'] }
    ];
    
    for (const { file, props } of components) {
      const filePath = join(process.cwd(), 'src/components/graph', file);
      
      try {
        const content = await readFile(filePath, 'utf-8');
        
        for (const prop of props) {
          expect(content).toContain(prop);
        }
        
        console.log(`✓ ${file} props comprehensive`);
      } catch (error) {
        throw new Error(`Props verification failed for ${file}: ${error}`);
      }
    }
  });
});