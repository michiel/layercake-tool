import { useEffect, useMemo, useRef } from 'react'
import ForceGraph3D from '3d-force-graph'
import SpriteText from 'three-spritetext'
import * as THREE from 'three'
import FlyControls from 'three-fly-controls'
import { GraphData } from './GraphPreview'

interface GraphPreview3DProps {
  data: GraphData | null
  width?: number
  height?: number
}

interface LayerStyle {
  nodeColor: string
  borderColor: string
  textColor: string
  linkColor: string
}

const DEFAULT_STYLE: LayerStyle = {
  nodeColor: '#4c6ef5',
  borderColor: '#364fc7',
  textColor: '#f8f9fa',
  linkColor: '#94a3b8'
}

export const GraphPreview3D = ({ data, width, height }: GraphPreview3DProps) => {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const graphRef = useRef<any>(null)
  const controlsRef = useRef<any>(null)
  const animationFrameRef = useRef<number | null>(null)
  const resizeObserverRef = useRef<ResizeObserver | null>(null)

  const layerStyles = useMemo(() => {
    const normalizeColor = (value?: string) => {
      if (!value) return undefined
      return value.startsWith('#') ? value : `#${value}`
    }

    const styles = new Map<string, LayerStyle>()
    data?.layers?.forEach((layer) => {
      const key = layer.layerId || layer.name || 'default'
      styles.set(key, {
        nodeColor: normalizeColor(layer.backgroundColor) || DEFAULT_STYLE.nodeColor,
        borderColor: normalizeColor(layer.borderColor) || DEFAULT_STYLE.borderColor,
        textColor: normalizeColor(layer.textColor) || DEFAULT_STYLE.textColor,
        linkColor: normalizeColor(layer.borderColor) || DEFAULT_STYLE.linkColor
      })
    })

    const getStyle = (layerId?: string) => {
      if (!layerId) return styles.get('default') || DEFAULT_STYLE
      return styles.get(layerId) || DEFAULT_STYLE
    }

    return { getStyle }
  }, [data])

  useEffect(() => {
    if (!containerRef.current || !data) {
      return
    }

    // Clean up previous instance
    if (graphRef.current) {
      graphRef.current._destructor?.()
      graphRef.current = null
    }

    const graphWidth = width || containerRef.current.clientWidth || 800
    const graphHeight = height || containerRef.current.clientHeight || 600

    const graphInstance: any = (ForceGraph3D as unknown as (container: HTMLElement) => any)(containerRef.current)

    const nodeRadius = 4.5

    graphInstance
      .width(graphWidth)
      .height(graphHeight)
      .backgroundColor('#0b1120')
      .graphData({
        nodes: data.nodes.map(node => ({ ...node })),
        links: data.links.map(link => ({ ...link }))
      })
      .nodeLabel((node: any) => node.name || node.id)
      .nodeOpacity(0.95)
      .nodeThreeObject((node: any) => {
        const style = layerStyles.getStyle(node.layer)
        const group = new THREE.Group()

        const sphereGeometry = new THREE.SphereGeometry(nodeRadius, 16, 16)
        const sphereMaterial = new THREE.MeshStandardMaterial({
          color: style.nodeColor,
          emissive: style.nodeColor,
          emissiveIntensity: 0.35,
          roughness: 0.4,
          metalness: 0.1
        })
        const sphere = new THREE.Mesh(sphereGeometry, sphereMaterial)
        group.add(sphere)

        const sprite = new SpriteText(node.name || node.id)
        sprite.color = '#e2e8f0'
        sprite.backgroundColor = 'rgba(15, 23, 42, 0.85)'
        sprite.padding = 4
        sprite.borderWidth = 0
        sprite.textHeight = nodeRadius * 2.2
        ;(sprite as any).material.depthWrite = false
        sprite.position.set(0, nodeRadius * 2.8, 0)
        group.add(sprite)

        return group
      })
      .nodeThreeObjectExtend(true)
      .linkColor((link: any) => layerStyles.getStyle(link.layer).linkColor)
      .linkOpacity(0.85)
      .linkDirectionalArrowColor((link: any) => layerStyles.getStyle(link.layer).linkColor)
      .linkDirectionalArrowLength(6)
      .linkDirectionalArrowRelPos(1)
      .linkDirectionalParticles(4)
      .linkDirectionalParticleWidth(1.5)
      .linkDirectionalParticleSpeed(() => 0.006)
      .linkThreeObjectExtend(true)

    graphInstance.linkThreeObject((link: any) => {
      if (!link.name) return undefined
      const sprite = new SpriteText(link.name)
      sprite.color = '#cbd5f5'
      sprite.backgroundColor = 'rgba(15, 23, 42, 0.75)'
      sprite.padding = 2
      sprite.borderWidth = 0
      sprite.textHeight = 3.5
      ;(sprite as any).material.depthWrite = false
      return sprite
    })

    graphInstance.linkPositionUpdate((sprite: any, { start, end }: any) => {
      if (!sprite) return
      const middlePos = {
        x: start.x + (end.x - start.x) / 2,
        y: start.y + (end.y - start.y) / 2,
        z: start.z + (end.z - start.z) / 2
      }
      sprite.position.set(middlePos.x, middlePos.y, middlePos.z)
    })

    graphRef.current = graphInstance

    requestAnimationFrame(() => {
      if (!graphRef.current) return
      const flyControls = new FlyControls(
        graphInstance.camera(),
        graphInstance.renderer()?.domElement || containerRef.current
      )
      flyControls.movementSpeed = 50
      flyControls.rollSpeed = 0.5
      flyControls.dragToLook = true
      flyControls.autoForward = false
      controlsRef.current = flyControls
      graphInstance.controls(flyControls)
    })

    const animate = () => {
      controlsRef.current?.update(0.016)
      animationFrameRef.current = requestAnimationFrame(animate)
    }
    animationFrameRef.current = requestAnimationFrame(animate)

    // Lighting tweaks for readability
    const scene = graphInstance.scene()
    if (scene) {
      const ambient = new THREE.AmbientLight(0xffffff, 0.6)
      const directional = new THREE.DirectionalLight(0xffffff, 0.6)
      directional.position.set(0, 0, 300)
      scene.add(ambient)
      scene.add(directional)
    }

    // Auto-zoom to fit
    graphInstance.zoomToFit(400, 10)

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
        animationFrameRef.current = null
      }
      controlsRef.current = null
      if (graphRef.current) {
        graphRef.current._destructor?.()
        graphRef.current = null
      }
    }
  }, [data, height, layerStyles, width])

  useEffect(() => {
    if (!containerRef.current) return

    if (resizeObserverRef.current) {
      resizeObserverRef.current.disconnect()
    }

    resizeObserverRef.current = new ResizeObserver(entries => {
      for (const entry of entries) {
        if (entry.target === containerRef.current && graphRef.current) {
          const newWidth = width || entry.contentRect.width
          const newHeight = height || entry.contentRect.height
          graphRef.current.width(newWidth)
          graphRef.current.height(newHeight)
        }
      }
    })

    resizeObserverRef.current.observe(containerRef.current)

    return () => {
      resizeObserverRef.current?.disconnect()
      resizeObserverRef.current = null
    }
  }, [height, width])

  if (!data) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: '#0b1120',
          color: '#94a3b8',
          fontFamily: 'Sans-Serif'
        }}
      >
        No preview data available.
      </div>
    )
  }

  return (
    <div
      ref={containerRef}
      style={{
        width: '100%',
        height: '100%',
        position: 'relative',
        background: '#0b1120',
        borderRadius: '8px',
        overflow: 'hidden'
      }}
    />
  )
}
