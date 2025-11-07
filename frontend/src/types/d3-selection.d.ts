declare module 'd3-selection' {
  export interface Selection<GElement extends Element = Element, Datum = unknown, PElement extends Element | null = null, PDatum = unknown> {
    graphviz(): any
  }

  export function select<GElement extends Element = Element>(selector: string): Selection<GElement>
  export function selectAll<GElement extends Element = Element>(selector: string): Selection<GElement>
}
