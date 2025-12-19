import { useEffect, useMemo, useRef, useState } from 'react'
import ForceGraph3D from '3d-force-graph'
import { Group, Mesh, MeshBasicMaterial, SphereGeometry } from 'three'
import SpriteText from 'three-spritetext'

interface Force3DViewerProps {
  graphData: {
    nodes: any[]
    links: any[]
  }
  showLinks: boolean
  showLabels: boolean
  nodeRelSize: number
  linkColor: string
  defaultNodeColor: string
  linkDistance: number
  chargeStrength: number
}

export default function Force3DViewer({
  graphData,
  showLinks,
  showLabels,
  nodeRelSize,
  linkColor,
  defaultNodeColor,
  linkDistance,
  chargeStrength,
}: Force3DViewerProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const fgRef = useRef<any>(null)
  const [graphInitialized, setGraphInitialized] = useState(false)
  const lastGraphDataRef = useRef<{ nodes: any[]; links: any[] } | null>(null)

  const safeNodeSize = useMemo(() => Number(nodeRelSize) || 4, [nodeRelSize])
  const safeLinkDistance = useMemo(() => Number(linkDistance) || 60, [linkDistance])
  const safeChargeStrength = useMemo(() => Number(chargeStrength) || -120, [chargeStrength])

  const cleanupForceGraph = () => {
    console.log('[ForceGraph] Cleanup')
    const instance = fgRef.current
    if (!instance) return
    try {
      if (typeof instance.pauseAnimation === 'function') {
        instance.pauseAnimation()
      }
      if (typeof instance.graphData === 'function') {
        instance.graphData({ nodes: [], links: [] })
      }
      if (containerRef.current) {
        containerRef.current.innerHTML = ''
      }
      fgRef.current = null
      setGraphInitialized(false)
      console.log('[ForceGraph] Instance destroyed.')
    } catch (error) {
      console.error('[ForceGraph] Error during cleanup:', error)
    }
  }

  // Initialize ForceGraph3D
  useEffect(() => {
    console.log('[ForceGraph] Init effect')
    const elem = containerRef.current
    if (!elem) {
      console.log('[ForceGraph] Skipping init: no container ref')
      return
    }

    if (!fgRef.current) {
      console.log('[ForceGraph] INITIALIZING new ForceGraph3D instance.')
      const width = elem.clientWidth
      const height = elem.clientHeight
      console.log('[ForceGraph] Container dimensions:', width, 'x', height)

      fgRef.current = (ForceGraph3D as any)()(elem)
        .forceEngine('d3')
        .backgroundColor('#0b1021')
        .showNavInfo(false)
        .width(width)
        .height(height)
        .graphData({ nodes: [], links: [] })

      setGraphInitialized(true)
      console.log('[ForceGraph] Initialized with size:', width, 'x', height)
    }

    return () => cleanupForceGraph()
  }, [])

  // Handle window resize
  useEffect(() => {
    if (!graphInitialized) return

    const handleResize = () => {
      const fg = fgRef.current
      if (fg && containerRef.current) {
        const width = containerRef.current.clientWidth
        const height = containerRef.current.clientHeight
        if (typeof fg.width === 'function' && typeof fg.height === 'function') {
          fg.width(width).height(height)
          console.log('[ForceGraph] Resized to', width, 'x', height)
        }
      }
    }

    window.addEventListener('resize', handleResize)
    const timeoutId = setTimeout(handleResize, 100)

    return () => {
      window.removeEventListener('resize', handleResize)
      clearTimeout(timeoutId)
    }
  }, [graphInitialized])

  // Apply data and control updates
  useEffect(() => {
    console.log('[ForceGraph] Update effect')
    const fg = fgRef.current
    if (!fg || !containerRef.current) {
      console.log('[ForceGraph] Skipping update: graph not initialized')
      return
    }

    if (graphData.nodes.length === 0 && graphData.links.length === 0) {
      console.log('[ForceGraph] Waiting for graph data before full update.')
      return
    }

    const hasGraphChanged = lastGraphDataRef.current !== graphData
    console.log(
      `[ForceGraph] UPDATING with ${graphData.nodes.length} nodes.`,
      { showLinks, showLabels, links: graphData.links.length, rebindData: hasGraphChanged }
    )

    if (hasGraphChanged) {
      fg.graphData(graphData)
      lastGraphDataRef.current = graphData
    }
    fg.linkVisibility(() => showLinks)
    fg.linkColor(() => (showLinks ? linkColor : 'rgba(0,0,0,0)'))
    fg.linkOpacity(showLinks ? 0.6 : 0)
    fg.linkWidth(showLinks ? 0.3 : 0)
    fg.linkDirectionalParticles((link: any) => (showLinks ? (link.weight || 1) : 0))
    fg.linkDirectionalParticleSpeed((link: any) => (link.weight || 1) * 0.001)
    fg.linkDirectionalArrowLength(3.5)
    fg.linkDirectionalArrowRelPos(1)
    fg.linkThreeObjectExtend(true)
    fg.linkThreeObject((link: any) => {
      if (!showLabels || !showLinks) return null
      const text = link.name
      if (!text) return null
      const sprite = new SpriteText(text)
      sprite.color = '#DDDDDD'
      sprite.textHeight = Math.max(3, safeNodeSize * 0.7)
      sprite.backgroundColor = 'rgba(0,0,0,0)'
      return sprite
    })
    fg.linkPositionUpdate((sprite: any, { start, end }: any) => {
      if (!sprite) return
      const middle = {
        x: start.x + (end.x - start.x) / 2,
        y: start.y + (end.y - start.y) / 2,
        z: start.z + (end.z - start.z) / 2,
      }
      sprite.position.set(middle.x, middle.y, middle.z)
    })
    fg.nodeLabel(showLabels ? () => '' : (n: any) => n.name || n.id)

    const linkForce = fg.d3Force('link')
    if (linkForce && typeof linkForce.distance === 'function' && Number.isFinite(safeLinkDistance)) {
      linkForce.distance(safeLinkDistance)
    }

    const chargeForce = fg.d3Force('charge')
    if (chargeForce && typeof chargeForce.strength === 'function' && Number.isFinite(safeChargeStrength)) {
      chargeForce.strength(safeChargeStrength)
    }

    fg.onNodeClick((node: any) => {
      const distance = 90
      const distRatio = 1 + distance / Math.max(Math.hypot(node.x || 0, node.y || 0, node.z || 0), 0.001)
      const newPos =
        node.x || node.y || node.z
          ? { x: (node.x || 0) * distRatio, y: (node.y || 0) * distRatio, z: (node.z || 0) * distRatio }
          : { x: 0, y: 0, z: distance }
      fg.cameraPosition(newPos, node, 3000)
    })

    fg
      .nodeLabel((n: any) => (showLabels ? '' : n.name || n.id))
      .nodeRelSize(safeNodeSize)
      .nodeColor((n: any) => n.color || defaultNodeColor)
      .nodeThreeObject((n: any) => {
        const group = new Group()

        const sphereGeom = new SphereGeometry(safeNodeSize * 0.8, 12, 12)
        const sphereMat = new MeshBasicMaterial({
          color: n.color || defaultNodeColor,
        })
        const sphere = new Mesh(sphereGeom, sphereMat)
        group.add(sphere)

        if (showLabels) {
          const label = n.name || n.id
          const sprite = new SpriteText(label)
          sprite.color = n.color || defaultNodeColor
          sprite.backgroundColor = 'rgba(0,0,0,0)'
          sprite.textHeight = Math.max(8, safeNodeSize * 2.2)
          sprite.center.set(0.5, 0)
          sprite.position.set(0, safeNodeSize * 1.6, 0)
          sprite.renderOrder = 10
          group.add(sprite)
        }
        return group
      })

    if (typeof fg.d3ReheatSimulation === 'function') {
      requestAnimationFrame(() => {
        if (fgRef.current) {
          fgRef.current.d3ReheatSimulation()
          console.log('[ForceGraph] Simulation reheated')
        }
      })
    }
  }, [
    graphData,
    showLinks,
    linkColor,
    defaultNodeColor,
    safeNodeSize,
    showLabels,
    safeLinkDistance,
    safeChargeStrength,
  ])

  return <div ref={containerRef} className="flex-1 w-full" />
}
