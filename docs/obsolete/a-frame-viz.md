# Technical implementation guide: Layercake graph visualiser

This guide details the technical design and implementation of a 3D graph visualiser using React and A-Frame. The system represents complex hierarchical structures and data flows using a "layercake" metaphor, where verticality represents hierarchy and layers, and horizontal space represents logical grouping.

## 1. Data architecture

The system consumes three CSV datasets. Below is an analysis of the provided `layers.csv` which defines the visual scheme.

### 1.1 Layer definition analysis

```python
import pandas as pd
df = pd.read_csv('layers.csv')
print(df.head())

```

```text
        id    label background_color text_color border_color                                            comment
0  main_01  Main 01           EBF2FF     003B94       B2CCFF    'Lightest Blue: Background for the largest...'
1  main_02  Main 02           B2CCFF     003B94       598BFF  'Light Blue: Background for secondary containers...'
2  main_03  Main 03           598BFF     FFFFFF       003B94  'Mid Blue: Standard container fill...'
3  main_04  Main 04           003B94     FFFFFF       002157  'Darkest Blue: Border/outline for main elements...'
4   alt_01   Alt 01           FFF5EB     944C00       FFCCB2  'Lightest Orange: Background for contrasting...'

```

### 1.2 Data schemas

* **Nodes set**:
* `id`: Unique identifier.
* `label`: Human-readable label.
* `layer`: Foreign key to a layer ID.
* `is_partition`: Boolean; if true, the node groups others and does not accept edge connections.
* `belongs_to`: Parent partition ID; empty for root nodes.


* **Edges set**:
* `source` / `target`: Node IDs (where `is_partition` is false).
* `label`: Description of flow.
* `layer`: Layer ID determining the edge's vertical context.


* **Layers set**:
* `background_color`, `text_color`, `border_color`: Hex codes (without #).
* `id`: Unique identifier used for logical grouping.



## 2. Spatial layout algorithm

The visualiser maps the graph into a 3D coordinate system (x, y, z).

### 2.1 Vertical axis (y)

The vertical position is strictly determined by the layer index.



Partition nodes (hierarchical containers) are rendered as semi-transparent volumes spanning the vertical range of their children, effectively creating "pillars" through the horizontal slices.

### 2.2 Horizontal plane (xz)

The horizontal layout uses a recursive squarified treemap algorithm to partition space based on the `belongs_to` hierarchy:

1. The root partition occupies the total available area.
2. Each partition divides its allocated rectangle among its children (sub-partitions and flow nodes).
3. A `partitionPadding` parameter creates gutters between blocks, ensuring edges have clear paths to travel without intersecting blocks.

### 2.3 Edge routing

To prevent edges from cutting through intermediate blocks, an orthogonal waypoint system is used:

1. **Departure**: Move from the source node to the nearest gutter.
2. **Elevation**: Travel vertically along the y axis to the target layer height.
3. **Navigation**: Travel through the xz gutters to the target node's horizontal footprint.
4. **Arrival**: Connect to the target node face.

## 3. Implementation code samples

### 3.1 Layout engine (TypeScript)

This engine calculates the spatial bounds for every element.

```typescript
import { hierarchy, treemap } from 'd3-hierarchy';

export interface LayoutParams {
  layerSpacing: number;
  partitionPadding: number;
  canvasSize: number;
}

export const computeLayout = (nodes: any[], layers: any[], params: LayoutParams) => {
  const { layerSpacing, partitionPadding, canvasSize } = params;

  // 1. Build hierarchy tree
  const rootNode = nodes.find(n => n.is_partition && !n.belongs_to);
  const tree = hierarchy(rootNode, (d) => nodes.filter(n => n.belongs_to === d.id))
    .count();

  // 2. Generate XZ coordinates using squarified treemap
  const layoutGenerator = treemap()
    .size([canvasSize, canvasSize])
    .padding(partitionPadding);
    
  const rootLayout = layoutGenerator(tree);

  // 3. Map layers to indices
  const layerMap = new Map(layers.map((l, i) => [l.id, { ...l, index: i }]));

  // 4. Transform to 3D positions
  return rootLayout.descendants().map(d => {
    const node = d.data;
    const layer = layerMap.get(node.layer) || { index: 0 };
    
    return {
      ...node,
      coords: {
        x: (d.x0 + d.x1) / 2 - canvasSize / 2,
        y: layer.index * layerSpacing,
        z: (d.y0 + d.y1) / 2 - canvasSize / 2,
        width: d.x1 - d.x0,
        depth: d.y1 - d.y0
      }
    };
  });
};

```

### 3.2 Visualiser component (React + A-Frame)

Renders the layout reactively based on data updates.

```jsx
import React, { useMemo, useState } from 'react';
import 'aframe';
import { Entity, Scene } from 'aframe-react';
import { computeLayout } from './layoutEngine';

const NodeBlock = ({ node, layerInfo }) => (
  <Entity
    geometry={{ primitive: 'box', width: node.coords.width, height: 1, depth: node.coords.depth }}
    position={`${node.coords.x} ${node.coords.y} ${node.coords.z}`}
    material={{ 
      color: `#${layerInfo.background_color}`, 
      opacity: 0.9,
      transparent: true 
    }}
  >
    <Entity
      text={{ value: node.label, align: 'center', width: 3, color: `#${layerInfo.text_color}` }}
      position={`0 0.6 0`}
      rotation="-90 0 0"
    />
  </Entity>
);

const ContextualCake = ({ nodes, edges, layers }) => {
  const [params, setParams] = useState({ layerSpacing: 8, partitionPadding: 2, canvasSize: 100 });
  const layout = useMemo(() => computeLayout(nodes, layers, params), [nodes, layers, params]);

  return (
    <div style={{ width: '100vw', height: '100vh' }}>
      <Scene shadow="type: pcfsoft">
        <Entity primitive="a-sky" color="#f0f0f0" />
        <Entity light={{ type: 'ambient', intensity: 0.5 }} />

        {layout.map(item => (
          !item.is_partition ? (
            <NodeBlock 
              key={item.id} 
              node={item} 
              layerInfo={layers.find(l => l.id === item.layer)} 
            />
          ) : (
            <Entity
              key={item.id}
              geometry={{ 
                primitive: 'box', 
                width: item.coords.width, 
                height: params.layerSpacing * layers.length, 
                depth: item.coords.depth 
              }}
              position={`${item.coords.x} ${(params.layerSpacing * layers.length) / 2} ${item.coords.z}`}
              material={{ color: '#cccccc', wireframe: true, opacity: 0.2, transparent: true }}
            />
          )
        ))}
        <Entity primitive="a-camera" position="0 20 40" look-controls orbit-controls />
      </Scene>
    </div>
  );
};

```

## 4. Interactive updating and considerations

### 4.1 Reactive structural updates

The design supports real-time editing of the graph structure:

* **Addition/Removal**: Modifying the `nodes` or `edges` state triggers an immediate recalculation of the hierarchy and treemap.
* **Visibility toggles**: Layers can be made invisible or transparent by filtering the `layout` array before rendering, allowing for "x-ray" inspection of specific horizontal slices.
* **Parameter tweaking**: Real-time adjustment of `layerSpacing` allows users to "explode" the diagram vertically to resolve edge-crossing visual clutter.

### 4.2 Visual styling and readability

* **Colour consistency**: Using the `layers.csv` data, blocks maintain consistent colouring regardless of their position in the hierarchy, aiding in the identification of service types (e.g., storage vs compute).
* **Transparency**: Partition volumes are rendered with low opacity or as wireframes to ensure internal flow nodes are visible from all angles.
* **Depth cues**: A subtle fog effect should be applied to the scene to help users perceive the vertical distance between layers.
* **Edge weight**: Flow edges are rendered as volumetric tubes; the radius of these tubes can be mapped to data volume or frequency to provide additional context.
