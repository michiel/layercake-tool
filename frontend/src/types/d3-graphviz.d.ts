declare module 'd3-graphviz' {
  export interface Graphviz {
    renderDot(dotSrc: string): Graphviz
    fit(enabled: boolean): Graphviz
    zoom(enabled: boolean): Graphviz
    width(width: number): Graphviz
    height(height: number): Graphviz
    scale(scale: number): Graphviz
    resetZoom(): Graphviz
    zoomBehavior(): any
    on(event: string, callback: () => void): Graphviz
  }

  export function graphviz(selector?: string, options?: any): Graphviz
}
