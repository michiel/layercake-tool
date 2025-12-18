/**
 * Layer3D Scene Component
 *
 * Hybrid React + A-Frame 1.7 implementation
 * - React manages state and GraphQL data
 * - A-Frame manages 3D scene imperatively
 *
 * Phase 1: Simple grid layout with layer stratification
 */

import { useEffect, useRef } from 'react'
import 'aframe'
import { useLayercakeLayout } from './hooks/useLayercakeLayout'

interface Layer3DSceneProps {
  nodes: Array<{ id: string; label: string; layer: string; color?: string; labelColor?: string }>
  edges: Array<{ id: string; source: string; target: string; label?: string }>
  layers: Array<{
    layerId: string
    name: string
    backgroundColor: string
    textColor: string
    borderColor: string
  }>
}

export default function Layer3DScene({ nodes, edges, layers }: Layer3DSceneProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const sceneInitialized = useRef(false)

  // Calculate layout using Phase 1 grid algorithm
  const layout = useLayercakeLayout(nodes, edges, layers, {
    layerSpacing: 10,
    nodeSize: 2,
    gridSpacing: 3,
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

    // Create camera
    const camera = document.createElement('a-entity')
    camera.setAttribute('camera', '')
    camera.setAttribute('look-controls', 'pointerLockEnabled: false')
    camera.setAttribute('wasd-controls', 'acceleration: 20')
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

    // Create nodes container
    const nodesContainer = document.createElement('a-entity')
    nodesContainer.setAttribute('id', 'layer3d-nodes')
    scene.appendChild(nodesContainer)

    // Create ground plane
    const ground = document.createElement('a-entity')
    ground.setAttribute('geometry', 'primitive: plane; width: 100; height: 100')
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
      plane.setAttribute('position', `0 ${index * 10 - 0.1} 0`)
      plane.setAttribute('rotation', '-90 0 0')
      plane.setAttribute('width', '100')
      plane.setAttribute('height', '100')
      plane.setAttribute('color', layer.backgroundColor)
      plane.setAttribute('opacity', '0.1')
      plane.setAttribute('transparent', 'true')
      container.appendChild(plane)
    })
  }, [layers])

  // Update camera position from bounding box
  useEffect(() => {
    if (!containerRef.current || layout.nodes.length === 0) return

    const camera = containerRef.current.querySelector('#layer3d-camera')
    if (!camera) return

    const { boundingBox } = layout
    const fov = 60 // degrees
    const fovRad = (fov * Math.PI) / 180

    // Calculate distance to fit entire graph with 10% padding
    const maxDimension = Math.max(boundingBox.sizeX, boundingBox.sizeY, boundingBox.sizeZ)
    const distance = (maxDimension / (2 * Math.tan(fovRad / 2))) * 1.1

    // Position camera at an angle to see all layers
    const cameraX = boundingBox.centerX + distance * 0.5
    const cameraY = boundingBox.centerY + distance * 0.3
    const cameraZ = boundingBox.centerZ + distance

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
      entity.setAttribute('color', node.color)
      entity.setAttribute('shadow', 'cast: true; receive: true')
      entity.setAttribute('data-node-id', node.id)
      entity.setAttribute('data-node-label', node.label)
      entity.setAttribute('layer3d-node-interaction', '')

      // Add text label
      const text = document.createElement('a-text')
      text.setAttribute('value', node.label)
      text.setAttribute('align', 'center')
      text.setAttribute('color', node.labelColor)
      text.setAttribute('position', `0 ${node.height / 2 + 0.5} 0`)
      text.setAttribute('scale', '2 2 2')
      text.setAttribute('side', 'double')
      entity.appendChild(text)

      container.appendChild(entity)
    })

    console.log('[Layer3D] Rendered', layout.nodes.length, 'nodes')
  }, [layout.nodes])

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

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />
}
