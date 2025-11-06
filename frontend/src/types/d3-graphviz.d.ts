declare module 'd3-graphviz' {
  export interface GraphvizOptions {
    fit?: boolean
    width?: number
    height?: number
    zoom?: boolean
    scale?: number
  }

  export interface Graphviz {
    renderDot(dotSrc: string): Promise<void>
    resetZoom(): Graphviz
    zoomBehavior(): any
  }

  export function graphviz(selector: string, options?: GraphvizOptions): Graphviz
}
