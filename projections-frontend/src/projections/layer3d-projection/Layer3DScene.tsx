/**
 * Layer3D Scene Component
 *
 * Hybrid React + A-Frame 1.7 implementation
 * - React manages state and GraphQL data
 * - A-Frame manages 3D scene imperatively
 *
 * Phase 1: Simple grid layout with layer stratification
 */

import { useEffect, useMemo, useRef } from 'react'
import 'aframe'
import { useControls } from 'leva'
import { useLayercakeLayout } from './hooks/useLayercakeLayout'

interface Layer3DSceneProps {
  nodes: Array<{ id: string; label: string; layer: string; color?: string; labelColor?: string; weight?: number; attrs?: Record<string, any> }>
  edges: Array<{ id: string; source: string; target: string; label?: string; weight?: number; attrs?: Record<string, any> }>
  layers: Array<{
    layerId: string
    name: string
    backgroundColor: string
    textColor: string
    borderColor: string
  }>
  state?: any
  onSaveState?: (state: any) => void
}

export default function Layer3DScene({ nodes, edges, layers, state, onSaveState }: Layer3DSceneProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const sceneInitialized = useRef(false)
  const saveTimerRef = useRef<number | null>(null)
  const lastSavedLayoutRef = useRef<{ canvasSize: number; layerSpacing: number; partitionPadding: number } | null>(null)

  const initialLayoutState = useMemo(
    () => ({
      canvasSize: state?.layout?.canvasSize ?? 200,
      layerSpacing: state?.layout?.layerSpacing ?? 20,
      partitionPadding: state?.layout?.partitionPadding ?? 3,
    }),
    [state]
  )

  // Leva controls for layout configuration
  const controls = useControls('Layer3D Layout', {
    canvasSize: { value: initialLayoutState.canvasSize, min: 50, max: 400, step: 10, label: 'Canvas Size' },
    layerSpacing: { value: initialLayoutState.layerSpacing, min: 5, max: 50, step: 1, label: 'Layer Spacing' },
    partitionPadding: { value: initialLayoutState.partitionPadding, min: 0, max: 10, step: 0.5, label: 'Partition Padding' },
  })

  // Persist controls to projection state with debounce - only when values actually change
  useEffect(() => {
    if (!onSaveState) return

    const currentLayout = {
      canvasSize: Number(controls.canvasSize),
      layerSpacing: Number(controls.layerSpacing),
      partitionPadding: Number(controls.partitionPadding),
    }

    // Check if values have actually changed
    if (lastSavedLayoutRef.current) {
      const hasChanged =
        lastSavedLayoutRef.current.canvasSize !== currentLayout.canvasSize ||
        lastSavedLayoutRef.current.layerSpacing !== currentLayout.layerSpacing ||
        lastSavedLayoutRef.current.partitionPadding !== currentLayout.partitionPadding

      if (!hasChanged) {
        return // No change, don't save
      }
    }

    // Clear existing timer
    if (saveTimerRef.current) {
      window.clearTimeout(saveTimerRef.current)
    }

    // Schedule save
    saveTimerRef.current = window.setTimeout(() => {
      console.log('[Layer3D] Saving layout state:', currentLayout)
      onSaveState({
        layout: currentLayout,
      })
      lastSavedLayoutRef.current = currentLayout
      saveTimerRef.current = null
    }, 400)
  }, [controls.canvasSize, controls.layerSpacing, controls.partitionPadding])

  // Calculate layout using Phase 2 treemap algorithm
  const layout = useLayercakeLayout(nodes, edges, layers, {
    canvasSize: Number(controls.canvasSize),
    layerSpacing: Number(controls.layerSpacing),
    partitionPadding: Number(controls.partitionPadding),
  })

  // Register A-Frame components once
  useEffect(() => {
    const AFRAME = (window as any).AFRAME
    if (!AFRAME) {
      console.error('[Layer3D] A-Frame not loaded')
      return
    }

    // Check if components already registered (HMR guard)
    if (!AFRAME.components['layer3d-node-interaction']) {
      AFRAME.registerComponent('layer3d-node-interaction', {
        init() {
          this.el.addEventListener('click', () => {
            const nodeId = this.el.getAttribute('data-node-id')
            const label = this.el.getAttribute('data-node-label')
            console.log('[Layer3D] Node clicked:', { nodeId, label })
            // TODO Phase 2: Emit event for node selection
          })

          // Hover effect
          this.el.addEventListener('mouseenter', () => {
            this.el.setAttribute('scale', '1.1 1.1 1.1')
          })

          this.el.addEventListener('mouseleave', () => {
            this.el.setAttribute('scale', '1 1 1')
          })
        },
      })
    }

    // Register vertical movement component for Q/E keys
    if (!AFRAME.components['vertical-controls']) {
      AFRAME.registerComponent('vertical-controls', {
        schema: {
          speed: { default: 3 }
        },
        init() {
          this.keys = {}
          this.velocity = new AFRAME.THREE.Vector3()

          window.addEventListener('keydown', (e) => {
            this.keys[e.key.toLowerCase()] = true
          })
          window.addEventListener('keyup', (e) => {
            this.keys[e.key.toLowerCase()] = false
          })
        },
        tick(_time: number, delta: number) {
          const speed = this.data.speed * (delta / 1000)
          const position = this.el.object3D.position

          if (this.keys['q']) {
            position.y -= speed
          }
          if (this.keys['e']) {
            position.y += speed
          }
        }
      })
    }
  }, [])

  // Initialize A-Frame scene imperatively
  useEffect(() => {
    if (!containerRef.current || sceneInitialized.current) return
    sceneInitialized.current = true

    const container = containerRef.current

    // Create A-Frame scene
    const scene = document.createElement('a-scene')
    scene.setAttribute('embedded', '')
    scene.setAttribute('shadow', 'type: pcfsoft')
    scene.setAttribute('stats', '')
    scene.style.width = '100%'
    scene.style.height = '100%'
    scene.style.position = 'absolute'
    scene.style.top = '0'
    scene.style.left = '0'

    // Wait for scene to load, then log canvas dimensions
    scene.addEventListener('loaded', () => {
      const canvas = scene.querySelector('canvas')
      if (canvas) {
        console.log('[Layer3D] Canvas dimensions:', {
          width: canvas.width,
          height: canvas.height,
          clientWidth: canvas.clientWidth,
          clientHeight: canvas.clientHeight,
          containerWidth: container.clientWidth,
          containerHeight: container.clientHeight,
        })
      }
    })

    // Create camera with faster controls
    const camera = document.createElement('a-entity')
    camera.setAttribute('camera', '')
    camera.setAttribute('look-controls', 'pointerLockEnabled: false; touchEnabled: true; mouseEnabled: true')
    camera.setAttribute('wasd-controls', 'acceleration: 150; easing: 20')
    camera.setAttribute('vertical-controls', 'speed: 5') // Q/E for up/down
    camera.setAttribute('id', 'layer3d-camera')
    scene.appendChild(camera)

    // Create lighting
    const ambientLight = document.createElement('a-light')
    ambientLight.setAttribute('type', 'ambient')
    ambientLight.setAttribute('color', '#888888')
    ambientLight.setAttribute('intensity', '0.6')
    scene.appendChild(ambientLight)

    const directionalLight = document.createElement('a-light')
    directionalLight.setAttribute('type', 'directional')
    directionalLight.setAttribute('position', '10 20 10')
    directionalLight.setAttribute('intensity', '0.8')
    directionalLight.setAttribute('castShadow', 'true')
    directionalLight.setAttribute('shadow-camera-near', '0.5')
    directionalLight.setAttribute('shadow-camera-far', '50')
    directionalLight.setAttribute('shadow-camera-left', '-20')
    directionalLight.setAttribute('shadow-camera-right', '20')
    directionalLight.setAttribute('shadow-camera-top', '20')
    directionalLight.setAttribute('shadow-camera-bottom', '-20')
    scene.appendChild(directionalLight)

    const hemisphereLight = document.createElement('a-light')
    hemisphereLight.setAttribute('type', 'hemisphere')
    hemisphereLight.setAttribute('groundColor', '#444444')
    hemisphereLight.setAttribute('skyColor', '#AAAAAA')
    hemisphereLight.setAttribute('intensity', '0.4')
    scene.appendChild(hemisphereLight)

    // Create layer planes container
    const layerPlanesContainer = document.createElement('a-entity')
    layerPlanesContainer.setAttribute('id', 'layer3d-layer-planes')
    scene.appendChild(layerPlanesContainer)

    // Create layer labels container
    const layerLabelsContainer = document.createElement('a-entity')
    layerLabelsContainer.setAttribute('id', 'layer3d-layer-labels')
    scene.appendChild(layerLabelsContainer)

    // Create nodes container
    const nodesContainer = document.createElement('a-entity')
    nodesContainer.setAttribute('id', 'layer3d-nodes')
    scene.appendChild(nodesContainer)

    // Create edges container
    const edgesContainer = document.createElement('a-entity')
    edgesContainer.setAttribute('id', 'layer3d-edges')
    scene.appendChild(edgesContainer)

    // Create ground plane
    const ground = document.createElement('a-entity')
    ground.setAttribute('id', 'layer3d-ground')
    ground.setAttribute('geometry', `primitive: plane; width: ${controls.canvasSize}; height: ${controls.canvasSize}`)
    ground.setAttribute('material', 'color: #0b1021; opacity: 0.8; side: double')
    ground.setAttribute('rotation', '-90 0 0')
    ground.setAttribute('position', '0 -1 0')
    scene.appendChild(ground)

    // Add to DOM
    container.appendChild(scene)

    console.log('[Layer3D] Scene initialized')

    // Cleanup on unmount
    return () => {
      if (container && container.contains(scene)) {
        container.removeChild(scene)
      }
      sceneInitialized.current = false
    }
  }, [])

  // Update ground plane size when canvas changes
  useEffect(() => {
    const ground = containerRef.current?.querySelector('#layer3d-ground')
    if (ground) {
      ground.setAttribute('geometry', `primitive: plane; width: ${controls.canvasSize}; height: ${controls.canvasSize}`)
    }
  }, [controls.canvasSize])

  // Update layer planes when layers change
  useEffect(() => {
    if (!containerRef.current) return

    const container = containerRef.current.querySelector('#layer3d-layer-planes')
    if (!container) return

    // Remove old layer planes
    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    // Add new layer planes
    layers.forEach((layer, index) => {
      const plane = document.createElement('a-plane')
      plane.setAttribute('position', `0 ${index * Number(controls.layerSpacing) - 0.1} 0`)
      plane.setAttribute('rotation', '-90 0 0')
      plane.setAttribute('width', String(controls.canvasSize))
      plane.setAttribute('height', String(controls.canvasSize))
      plane.setAttribute('color', layer.backgroundColor)
      plane.setAttribute('opacity', '0.25')
      plane.setAttribute('transparent', 'true')
      plane.setAttribute('material', 'side: double') // Make visible from both sides
      container.appendChild(plane)
    })
  }, [layers, controls.canvasSize, controls.layerSpacing])

  // Update layer labels when layers change
  useEffect(() => {
    if (!containerRef.current) return

    const container = containerRef.current.querySelector('#layer3d-layer-labels')
    if (!container) return

    // Remove old layer labels
    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    // Add new layer labels - positioned to the side of the canvas
    const labelOffset = Number(controls.canvasSize) / 2 + 15 // Position outside canvas edge
    layers.forEach((layer, index) => {
      const label = document.createElement('a-text')
      label.setAttribute('value', layer.name.toUpperCase())
      label.setAttribute('align', 'left')
      label.setAttribute('anchor', 'left')
      label.setAttribute('baseline', 'center')
      label.setAttribute('color', '#FFFFFF')
      label.setAttribute('position', `${-labelOffset} ${index * Number(controls.layerSpacing)} 0`)
      label.setAttribute('scale', '8 8 8') // Large labels
      label.setAttribute('side', 'double')
      label.setAttribute('shader', 'msdf')
      label.setAttribute('font', 'https://cdn.aframe.io/fonts/Roboto-msdf.json')
      label.setAttribute('wrap-count', '15')

      // Add subtle background for readability
      const bg = document.createElement('a-plane')
      bg.setAttribute('position', `${-labelOffset + 8} ${index * Number(controls.layerSpacing)} -0.5`)
      bg.setAttribute('width', '30')
      bg.setAttribute('height', '10')
      bg.setAttribute('color', layer.backgroundColor)
      bg.setAttribute('opacity', '0.3')
      bg.setAttribute('transparent', 'true')

      container.appendChild(bg)
      container.appendChild(label)
    })
  }, [layers, controls.canvasSize, controls.layerSpacing])

  // Update camera position from bounding box
  useEffect(() => {
    if (!containerRef.current || layout.nodes.length === 0) return

    const camera = containerRef.current.querySelector('#layer3d-camera')
    if (!camera) return

    const { boundingBox } = layout
    const fov = 60 // degrees
    const fovRad = (fov * Math.PI) / 180

    // Calculate distance to fit entire graph with 30% padding for better overview
    const maxDimension = Math.max(boundingBox.sizeX, boundingBox.sizeY, boundingBox.sizeZ)
    const distance = (maxDimension / (2 * Math.tan(fovRad / 2))) * 1.3

    // Position camera at an angle to see all layers with better perspective
    const cameraX = boundingBox.centerX + distance * 0.6
    const cameraY = boundingBox.centerY + distance * 0.4
    const cameraZ = boundingBox.centerZ + distance * 0.8

    camera.setAttribute('position', `${cameraX} ${cameraY} ${cameraZ}`)
    ;(camera as any).object3D.lookAt({ x: boundingBox.centerX, y: boundingBox.centerY, z: boundingBox.centerZ })

    console.log('[Layer3D] Camera positioned:', {
      position: { x: cameraX, y: cameraY, z: cameraZ },
      lookAt: { x: boundingBox.centerX, y: boundingBox.centerY, z: boundingBox.centerZ },
      boundingBox,
    })
  }, [layout])

  // Update nodes imperatively when layout changes
  useEffect(() => {
    if (!containerRef.current) return

    const container = containerRef.current.querySelector('#layer3d-nodes')
    if (!container) return

    // Remove old nodes
    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    // Add new nodes
    layout.nodes.forEach((node) => {
      const entity = document.createElement('a-box')
      entity.setAttribute('position', `${node.x} ${node.y} ${node.z}`)
      entity.setAttribute('width', String(node.width))
      entity.setAttribute('height', String(node.height))
      entity.setAttribute('depth', String(node.depth))
      entity.setAttribute('data-node-id', node.id)
      entity.setAttribute('data-node-label', node.label)
      entity.setAttribute('layer3d-node-interaction', '')
      entity.setAttribute('shadow', 'cast: true; receive: true')

      // Partition nodes: wireframe pillars with transparency
      // Leaf nodes: solid boxes
      if (node.isPartition) {
        entity.setAttribute('material', `color: ${node.color}; opacity: 0.3; transparent: true; wireframe: true`)
      } else {
        entity.setAttribute('color', node.color)
        entity.setAttribute('material', `opacity: 0.9; transparent: true`)
      }

      // Add text label above the node
      const text = document.createElement('a-text')
      text.setAttribute('value', node.label)
      text.setAttribute('align', 'center')
      text.setAttribute('anchor', 'center')
      text.setAttribute('baseline', 'bottom')
      text.setAttribute('color', '#FFFFFF') // White text for visibility
      text.setAttribute('position', `0 ${node.height / 2 + 1} 0`)
      text.setAttribute('scale', '4 4 4') // Larger text
      text.setAttribute('side', 'double')
      text.setAttribute('shader', 'msdf')
      text.setAttribute('font', 'https://cdn.aframe.io/fonts/Roboto-msdf.json')
      entity.appendChild(text)

      container.appendChild(entity)
    })

    console.log('[Layer3D] Rendered', layout.nodes.length, 'nodes')
  }, [layout.nodes])

  // Render edges with simple orthogonal routing
  useEffect(() => {
    if (!containerRef.current) return
    const container = containerRef.current.querySelector('#layer3d-edges')
    if (!container) return

    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    const nodeMap = new Map(layout.nodes.map((n) => [n.id, n]))
    edges.forEach((edge) => {
      const source = nodeMap.get(edge.source)
      const target = nodeMap.get(edge.target)
      if (!source || !target) return

      const p1 = `${source.x} ${source.y} ${source.z}`
      const p2 = `${source.x} ${target.y} ${source.z}`
      const p3 = `${target.x} ${target.y} ${target.z}`

      const seg1 = document.createElement('a-entity')
      seg1.setAttribute('line', `start: ${p1}; end: ${p2}; color: #888; opacity: 0.6`)
      const seg2 = document.createElement('a-entity')
      seg2.setAttribute('line', `start: ${p2}; end: ${p3}; color: #888; opacity: 0.6`)
      container.appendChild(seg1)
      container.appendChild(seg2)
    })
  }, [edges, layout.nodes])

  // Handle WebGL context loss
  useEffect(() => {
    const handleContextLost = (event: Event) => {
      event.preventDefault()
      console.error('[Layer3D] WebGL context lost')
      alert('WebGL context lost. Please reload the page.')
    }

    const canvas = containerRef.current?.querySelector('canvas')
    if (canvas) {
      canvas.addEventListener('webglcontextlost', handleContextLost)
      return () => canvas.removeEventListener('webglcontextlost', handleContextLost)
    }
  }, [])

  // Show errors if validation failed
  if (layout.errors.length > 0) {
    return (
      <div className="flex h-full items-center justify-center flex-col gap-4 bg-slate-900 text-slate-100">
        <div className="text-6xl">⚠️</div>
        <div className="text-2xl font-bold">Validation Error</div>
        <div className="text-slate-400 max-w-md">
          <ul className="list-disc list-inside">
            {layout.errors.map((error, i) => (
              <li key={i}>{error}</li>
            ))}
          </ul>
        </div>
      </div>
    )
  }

  // Show warnings in console
  if (layout.warnings.length > 0) {
    console.warn('[Layer3D] Validation warnings:', layout.warnings)
  }

  return (
    <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%' }}>
      <div ref={containerRef} style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%' }} />

      {/* Controls help overlay */}
      <div style={{
        position: 'absolute',
        bottom: '20px',
        left: '20px',
        backgroundColor: 'rgba(15, 23, 42, 0.85)',
        color: '#E2E8F0',
        padding: '12px 16px',
        borderRadius: '8px',
        fontSize: '13px',
        fontFamily: 'monospace',
        pointerEvents: 'none',
        backdropFilter: 'blur(8px)',
        border: '1px solid rgba(148, 163, 184, 0.2)',
      }}>
        <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#94A3B8' }}>
          🎮 Camera Controls
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '4px 12px', fontSize: '12px' }}>
          <span style={{ color: '#CBD5E1' }}>W/A/S/D</span>
          <span style={{ color: '#94A3B8' }}>Move forward/left/back/right</span>
          <span style={{ color: '#CBD5E1' }}>Q/E</span>
          <span style={{ color: '#94A3B8' }}>Move down/up</span>
          <span style={{ color: '#CBD5E1' }}>Mouse</span>
          <span style={{ color: '#94A3B8' }}>Click + drag to look around</span>
          <span style={{ color: '#CBD5E1' }}>Shift</span>
          <span style={{ color: '#94A3B8' }}>Hold to move faster</span>
        </div>
      </div>
    </div>
  )
}
