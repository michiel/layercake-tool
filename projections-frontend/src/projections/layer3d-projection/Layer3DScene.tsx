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
import SpriteText from 'three-spritetext'
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

    // Wait for scene to be ready, then add THREE.js text
    scene.addEventListener('loaded', () => {
      const sceneEl = scene as any
      const threeScene = sceneEl.object3D

      // DEBUG: Add visible THREE.js sprite text
      const debugText = new SpriteText('DEBUG TEXT', 10)
      debugText.color = '#FF0000'
      debugText.position.set(0, 5, -10)
      threeScene.add(debugText)
      console.log('[Layer3D] Added THREE.js DEBUG TEXT at 0, 5, -10')
    })

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
      // depthWrite: false allows nodes to render through transparent layers
      plane.setAttribute('material', `color: ${layer.backgroundColor}; opacity: 0.25; transparent: true; side: double; depthWrite: false`)
      container.appendChild(plane)
    })
  }, [layers, controls.canvasSize, controls.layerSpacing])

  // Update layer labels when layers change
  useEffect(() => {
    if (!containerRef.current) return

    const container = containerRef.current.querySelector('#layer3d-layer-labels')
    const sceneEl = containerRef.current.querySelector('a-scene') as any
    if (!container || !sceneEl || !sceneEl.object3D) return

    // Remove old layer labels
    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    // Add new layer labels - positioned to the side of the canvas
    const labelOffset = Number(controls.canvasSize) / 2 + 20 // Position outside canvas edge
    layers.forEach((layer, index) => {
      // Add background plane
      const bg = document.createElement('a-plane')
      bg.setAttribute('position', `${-labelOffset + 10} ${index * Number(controls.layerSpacing)} -0.5`)
      bg.setAttribute('width', '40')
      bg.setAttribute('height', '8')
      bg.setAttribute('color', layer.backgroundColor)
      bg.setAttribute('opacity', '0.3')
      bg.setAttribute('transparent', 'true')
      container.appendChild(bg)

      // Add label entity
      const labelEntity = document.createElement('a-entity')
      labelEntity.setAttribute('position', `${-labelOffset} ${index * Number(controls.layerSpacing)} 0`)
      container.appendChild(labelEntity)

      // Add THREE.js text after entity is ready
      requestAnimationFrame(() => {
        const entityEl = labelEntity as any
        if (entityEl.object3D) {
          const sprite = new SpriteText(layer.name.toUpperCase(), 8)
          sprite.color = '#FFFFFF'
          sprite.textHeight = 8
          sceneEl.object3D.add(sprite)
          sprite.position.set(-labelOffset, index * Number(controls.layerSpacing), 0)
          console.log('[Layer3D] Added THREE.js layer label:', layer.name, 'at', -labelOffset, index * Number(controls.layerSpacing))
        }
      })
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
    const sceneEl = containerRef.current.querySelector('a-scene') as any
    if (!container || !sceneEl || !sceneEl.object3D) return

    // Remove old nodes
    while (container.firstChild) {
      container.removeChild(container.firstChild)
    }

    // Add new nodes
    layout.nodes.forEach((node) => {
      console.log('[Layer3D] Rendering node:', node.label, 'isPartition:', node.isPartition, 'height:', node.height)
      let entity: HTMLElement

      if (node.isPartition) {
        // Partition nodes: Vertical containers spanning multiple layers
        entity = document.createElement('a-entity')
        entity.setAttribute('position', `${node.x} ${node.y} ${node.z}`)

        // Create semi-transparent fill box
        const fillBox = document.createElement('a-box')
        fillBox.setAttribute('width', String(node.width))
        fillBox.setAttribute('height', String(node.height))
        fillBox.setAttribute('depth', String(node.depth))
        fillBox.setAttribute('material', `color: ${node.color}; opacity: 0.08; transparent: true`)
        fillBox.setAttribute('shadow', 'receive: true')
        entity.appendChild(fillBox)

        // Create wireframe outline for emphasis
        const wireBox = document.createElement('a-box')
        wireBox.setAttribute('width', String(node.width))
        wireBox.setAttribute('height', String(node.height))
        wireBox.setAttribute('depth', String(node.depth))
        wireBox.setAttribute('material', `color: ${node.color}; opacity: 0.6; transparent: true; wireframe: true`)
        entity.appendChild(wireBox)

        console.log('[Layer3D] Partition node:', node.label, 'height:', node.height, 'at Y:', node.y)
      } else {
        // Flow nodes: Solid boxes at specific layers
        entity = document.createElement('a-box')
        entity.setAttribute('position', `${node.x} ${node.y} ${node.z}`)
        entity.setAttribute('width', String(node.width))
        entity.setAttribute('height', String(node.height))
        entity.setAttribute('depth', String(node.depth))
        entity.setAttribute('color', node.color)
        entity.setAttribute('material', `opacity: 0.95; transparent: true`)
        entity.setAttribute('shadow', 'cast: true; receive: true')

        console.log('[Layer3D] Flow node:', node.label, 'at layer Y:', node.y)
      }

      entity.setAttribute('data-node-id', node.id)
      entity.setAttribute('data-node-label', node.label)
      entity.setAttribute('layer3d-node-interaction', '')

      container.appendChild(entity)

      // Add THREE.js text label after entity is added
      requestAnimationFrame(() => {
        const entityEl = entity as any
        if (entityEl.object3D) {
          const textHeight = node.isPartition ? 7 : 5
          const sprite = new SpriteText(node.isPartition ? `[${node.label}]` : node.label, textHeight)
          sprite.color = node.isPartition ? node.color : '#FFFFFF'
          sprite.position.set(0, node.height / 2 + 3, 0)
          entityEl.object3D.add(sprite)
          console.log('[Layer3D] Added THREE.js text for:', node.label)
        }
      })
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
