```python
import pandas as pd
df = pd.read_csv('layers.csv')
print(df.head())
print(df.info())


```

```text
        id    label background_color text_color border_color                                                                                         comment
0  main_01  Main 01           EBF2FF     003B94       B2CCFF    'Lightest Blue: Background for the largest/outermost container, or positive/good component.'
1  main_02  Main 02           B2CCFF     003B94       598BFF  'Light Blue: Background for secondary containers (one level nested), or a standard component.'
2  main_03  Main 03           598BFF     FFFFFF       003B94                              'Mid Blue: Standard container fill, or a primary component focus.'
3  main_04  Main 04           003B94     FFFFFF       002157               'Darkest Blue: Border/outline for main elements, or background for deep nesting.'
4   alt_01   Alt 01           FFF5EB     944C00       FFCCB2  'Lightest Orange: Background for contrasting outer container or secondary positive component.'
<class 'pandas.core.frame.DataFrame'>
RangeIndex: 15 entries, 0 to 14
Data columns (total 6 columns):
 #   Column            Non-Null Count  Dtype 
---  ------            --------------  ----- 
 0   id                15 non-null     object
 1   label             15 non-null     object
 2   background_color  15 non-null     object
 3   text_color        15 non-null     object
 4   border_color      15 non-null     object
 5   comment           15 non-null     object
dtypes: object(6)
memory usage: 848.0+ bytes
None


```

This technical design outlines a 3D visualiser for layercake graphs using React and A-Frame. The goal is to represent complex hierarchical data and flow relationships in an immersive, structured environment.

### 1. Architectural overview

The application will be built as a React-based single-page application (SPA) that manages the state of the graph data and layout parameters. A-Frame provides the declarative 3D framework, allowing us to render nodes, partitions, and edges as 3D entities.

* **React**: Handles state management, CSV parsing, and the user interface for layout tweaking.
* **A-Frame**: Renders the 3D scene, handles camera controls, and manages the lifecycle of 3D objects.
* **D3-hierarchy**: Used to compute the nested spatial bounds for the `belongs_to` partition structure.

### 2. Data model and processing

The visualiser expects three CSV inputs: `nodes.csv`, `edges.csv`, and `layers.csv`.

#### Processing steps:

1. **Hierarchy construction**: Transform the flat node list into a tree structure using the `belongs_to` attribute.
2. **Layer mapping**: Index the `layers.csv` data to quickly access colour schemes for rendering.
3. **Validation**: Ensure all edges link to nodes where `is_partition` is `false`.

### 3. Layout algorithm

The layout uses a coordinate system where the y-axis represents layers, and the xz-plane represents the hierarchy.

#### Horizontal partitioning (XZ-plane)

A recursive partitioning algorithm (similar to a 3D treemap) calculates the footprint of each partition node.

* The root node occupies the total available area.
* Children of a partition are assigned sub-rectangles within their parent's bounds.
* Padding is applied between partitions to ensure visual separation.

#### Vertical slicing (Y-axis)

The y-position is determined by the layer index.

* Nodes are placed at y = \text{layerIndex} \times \text{layerHeight}.
* Partitions are rendered as semi-transparent "pillars" or stacked boxes that span the vertical range of their contained nodes.

#### Edge routing

To prevent edges from cutting through blocks, an orthogonal routing system is used:

1. **Exit point**: The edge starts at the centre of a node face.
2. **Waypoints**: The path follows the grid lines between partition boundaries.
3. **Visualisation**: Edges are rendered using A-Frame `meshline` or tube geometries to represent flow volume.

### 4. Interactive parameters

The following parameters will be exposed via a control panel for real-time tweaking:

* `layerSpacing`: Vertical distance between horizontal slices.
* `partitionPadding`: Horizontal gap between hierarchical groupings.
* `nodeScale`: The size of the individual flow blocks.
* `edgeWidth`: Thickness of the flow lines.
* `opacity`: Transparency of the partition containers.

### 5. Implementation code samples

#### A. Layout state management (React)

This component manages the parameters and triggers the layout recalculation.

```javascript
import React, { useState, useMemo } from 'react';
import { Scene } from 'aframe-react';
import { computeLayout } from './layoutEngine';

const Visualiser = ({ nodes, edges, layers }) => {
  const [params, setParams] = useState({
    layerSpacing: 5,
    partitionPadding: 1.5,
    nodeScale: 1
  });

  // Re-calculate positions whenever data or params change
  const layout = useMemo(() => 
    computeLayout(nodes, edges, layers, params), 
    [nodes, edges, layers, params]
  );

  return (
    <div className="visualiser-container">
      <Scene>
        <a-assets>
          {/* Layer-based materials could be pre-defined here */}
        </a-assets>

        {layout.partitions.map(p => (
          <PartitionVolume key={p.id} data={p} />
        ))}

        {layout.nodes.map(n => (
          <NodeBlock key={n.id} data={n} />
        ))}

        {layout.edges.map(e => (
          <FlowEdge key={e.id} data={e} />
        ))}
        
        <a-entity camera look-controls orbit-controls position="0 10 20"></a-entity>
      </Scene>
      <Controls params={params} setParams={setParams} />
    </div>
  );
};

```

#### B. Node block component (A-Frame)

This component renders an individual node using styling from the `layers.csv` data.

```javascript
const NodeBlock = ({ data }) => {
  const { x, y, z, label, layerInfo } = data;
  
  return (
    <a-entity position={`${x} ${y} ${z}`}>
      <a-box 
        width="1" height="1" depth="1"
        color={`#${layerInfo.background_color}`}
        material={`transparent: false; opacity: 1`}
      >
        <a-text 
          value={label} 
          align="center" 
          position="0 0 0.6" 
          color={`#${layerInfo.text_color}`}
          width="4"
        ></a-text>
      </a-box>
      {/* Border representation */}
      <a-entity 
        line={`start: -0.5 -0.5 0.5; end: 0.5 -0.5 0.5; color: #${layerInfo.border_color}`}
        // ... additional lines for box wireframe ...
      ></a-entity>
    </a-entity>
  );
};

```

#### C. Layout engine logic (simplified)

The core logic for mapping the `belongs_to` hierarchy to 3D space.

```javascript
export const computeLayout = (nodes, edges, layers, params) => {
  const { layerSpacing, partitionPadding } = params;

  // 1. Assign layer indices
  const layerMap = new Map(layers.map((l, i) => [l.id, { ...l, index: i }]));

  // 2. Generate tree from belongs_to
  const hierarchy = buildHierarchy(nodes); // Returns d3-like tree

  // 3. Compute XZ footprints using a squarified treemap approach
  const treemap = d3.treemap()
    .size([50, 50])
    .padding(partitionPadding);
  
  const root = treemap(d3.hierarchy(hierarchy).count());

  // 4. Map nodes to coordinates
  const positionedNodes = nodes.filter(n => !n.is_partition).map(node => {
    const layer = layerMap.get(node.layer);
    const bounds = findBoundsInHierarchy(root, node.id);
    
    return {
      ...node,
      x: (bounds.x0 + bounds.x1) / 2,
      y: layer.index * layerSpacing,
      z: (bounds.y0 + bounds.y1) / 2,
      layerInfo: layer
    };
  });

  return { nodes: positionedNodes, partitions: root.descendants(), edges: routeEdges(edges, positionedNodes) };
};

```

### 6. Visual style recommendations

To maintain readability:

* **Partition containers**: Use a low-opacity version of the `background_color` from `main_01` or `main_02` to show the bounding box of a grouping without obscuring internal nodes.
* **Typography**: Use a sans-serif font like `Roboto` or `System` with high contrast against the node background.
* **Depth cues**: Implement a slight fog effect in the A-Frame scene to help the user perceive distance between layers.
* **Interactive highlighting**: When a node is hovered, its ancestors (partitions) and connected edges should increase in brightness or change line weight to clarify relationships.





# Technical design: Contextual cake visualiser

This document outlines the architecture and implementation of a 3D graph visualiser using React and A-Frame. The system represents complex hierarchical structures and data flows using a "layercake" metaphor, where verticality represents hierarchy and layers, and horizontal space represents logical grouping.

## 1. Data architecture

The system consumes three distinct datasets, typically provided as CSV files.

### 1.1 Nodes

Nodes represent either logical containers (partitions) or functional units (flow nodes).

* `id`: unique string.
* `label`: display name.
* `layer`: reference to a layer ID.
* `is_partition`: boolean (true for hierarchy containers, false for flow blocks).
* `belongs_to`: reference to the parent partition ID.
* `comment`: optional metadata.

### 1.2 Edges

Edges represent flow between nodes.

* `source` / `target`: references to node IDs where `is_partition` is `false`.
* `label`: flow description.
* `layer`: the layer context of the flow.

### 1.3 Layers

Layers define the visual aesthetics and vertical grouping.

* `background_color`, `text_color`, `border_color`: hex codes (without #).
* `id`: unique identifier.

## 2. Spatial layout algorithm

The visualiser maps the graph into a Cartesian coordinate system (x, y, z).

### 2.1 The vertical axis (y)

The vertical position is strictly determined by the layer. Each layer ID is assigned an index i \in \{0, \dots, L-1\}.



Partition nodes are rendered as semi-transparent volumes that span the entire height of the "cake", or specifically the range of layers their children occupy.

### 2.2 The horizontal plane (xz)

The horizontal layout uses a recursive squarified treemap algorithm to partition space based on the `belongs_to` hierarchy.

1. The root partition (where `belongs_to` is empty) is assigned the maximum bounds.
2. Each partition divides its allocated rectangle among its children (both sub-partitions and flow nodes).
3. A `partitionPadding` constant creates "gutters" between blocks, ensuring edges have clear paths to travel without intersecting nodes.

### 2.3 Edge routing

To ensure edges do not cut through intermediate blocks, the visualiser uses an orthogonal waypoint system:

1. **Level 1**: Move from the source node to the nearest "gutter" (the padding area).
2. **Level 2**: Travel vertically (y) to the target layer.
3. **Level 3**: Travel through the XZ gutters to the target node's footprint.
4. **Level 4**: Connect to the target node.

## 3. Implementation

### 3.1 Layout engine (TypeScript)

This engine calculates the spatial bounds for every element.

```typescript
// layoutEngine.ts
import { hierarchy, treemap } from 'd3-hierarchy';

export interface LayoutConfig {
  layerSpacing: number;
  partitionPadding: number;
  canvasSize: number;
}

export const computeLayout = (nodes: any[], layers: any[], config: LayoutConfig) => {
  const { layerSpacing, partitionPadding, canvasSize } = config;

  // 1. Build hierarchy
  const rootData = nodes.find(n => n.is_partition && !n.belongs_to);
  const tree = hierarchy(rootData, (d) => nodes.filter(n => n.belongs_to === d.id))
    .count();

  // 2. Generate XZ coordinates
  const layoutGenerator = treemap()
    .size([canvasSize, canvasSize])
    .padding(partitionPadding);
    
  const rootLayout = layoutGenerator(tree);

  // 3. Map layers to Y indices
  const layerMap = new Map(layers.map((l, i) => [l.id, { ...l, index: i }]));

  // 4. Transform to 3D positions
  const processedNodes = rootLayout.descendants().map(d => {
    const node = d.data;
    const layer = layerMap.get(node.layer) || { index: 0 };
    
    return {
      ...node,
      coords: {
        x: (d.x0 + d.x1) / 2 - canvasSize / 2,
        y: layer.index * layerSpacing,
        z: (y0 + y1) / 2 - canvasSize / 2,
        width: d.x1 - d.x0,
        depth: d.y1 - d.y0
      }
    };
  });

  return processedNodes;
};

```

### 3.2 Visualiser component (React + A-Frame)

This component renders the calculated layout using A-Frame entities.

```jsx
// Visualiser.jsx
import React, { useMemo } from 'react';
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
      text={{ value: node.label, align: 'center', width: 3 }}
      position={`0 0.6 0`}
      rotation="-90 0 0"
    />
  </Entity>
);

const ContextualCake = ({ data, params }) => {
  const { nodes, edges, layers } = data;
  
  const layout = useMemo(() => computeLayout(nodes, layers, params), [nodes, layers, params]);

  return (
    <Scene shadow="type: pcfsoft">
      <a-assets>
        {/* Assets like textures or mixins go here */}
      </a-assets>

      <Entity primitive="a-sky" color="#f0f0f0" />
      <Entity light={{ type: 'ambient', intensity: 0.5 }} />
      <Entity light={{ type: 'directional', intensity: 0.8 }} position="10 10 5" />

      {layout.map(node => (
        !node.is_partition ? (
          <NodeBlock 
            key={node.id} 
            node={node} 
            layerInfo={layers.find(l => l.id === node.layer)} 
          />
        ) : (
          <Entity
            key={node.id}
            geometry={{ 
              primitive: 'box', 
              width: node.coords.width, 
              height: params.layerSpacing * layers.length, 
              depth: node.coords.depth 
            }}
            position={`${node.coords.x} ${(params.layerSpacing * layers.length) / 2} ${node.coords.z}`}
            material={{ 
              color: '#cccccc', 
              wireframe: true, 
              opacity: 0.2, 
              transparent: true 
            }}
          />
        )
      ))}

      {/* Edge rendering logic would iterate over edges here */}
      
      <Entity primitive="a-camera" position="0 20 40" look-controls orbit-controls />
    </Scene>
  );
};

export default ContextualCake;

```

### 3.3 Interactive control panel

To allow interactive tweaking, wrap the visualiser in a parent component that manages state for the layout parameters.

```jsx
// App.jsx
const App = () => {
  const [params, setParams] = React.useState({
    layerSpacing: 8,
    partitionPadding: 2,
    canvasSize: 100
  });

  return (
    <div style={{ width: '100vw', height: '100vh' }}>
      <div className="controls" style={{ position: 'absolute', zIndex: 10, padding: 20 }}>
        <label>Layer spacing</label>
        <input 
          type="range" min="2" max="20" 
          value={params.layerSpacing} 
          onChange={e => setParams({...params, layerSpacing: parseFloat(e.target.value)})} 
        />
        <label>Partition padding</label>
        <input 
          type="range" min="0" max="10" 
          value={params.partitionPadding} 
          onChange={e => setParams({...params, partitionPadding: parseFloat(e.target.value)})} 
        />
      </div>
      <ContextualCake data={graphData} params={params} />
    </div>
  );
};

```

## 4. Visual styling and optimisations

* **Colour consistency**: The `layers.csv` definitions ensure that even if nodes move between partitions, their colour (representing service type or platform) remains consistent.
* **Transparency**: Partitions are rendered as wireframes or low-opacity shells to ensure that flow nodes on the interior are always visible from multiple camera angles.
* **Level of detail (LOD)**: For large graphs, text labels should only become visible when the camera is within a specific distance threshold to maintain performance.
* **Edge thickness**: Edges should have a thickness property tied to their flow volume, rendered using `a-tube` or `meshline` for a volumetric appearance.


# Technical design: Layercake graph visualiser

This document outlines the architecture and implementation of a 3D graph visualiser using React and A-Frame. The system represents complex hierarchical structures and data flows using a "layercake" metaphor, where verticality represents hierarchy and layers, and horizontal space represents logical grouping.

## 1. Data architecture

The system consumes three distinct datasets, typically provided as CSV files. A sample analysis of the `layers.csv` is provided below.

### 1.1 Sample data analysis (layers.csv)

```python
import pandas as pd
df = pd.read_csv('layers.csv')
print(df.head())
print(df.info())

```

```text
        id    label background_color text_color border_color                                            comment
0  main_01  Main 01           EBF2FF     003B94       B2CCFF    'Lightest Blue: Background for the largest...'
1  main_02  Main 02           B2CCFF     003B94       598BFF  'Light Blue: Background for secondary containers...'
2  main_03  Main 03           598BFF     FFFFFF       003B94  'Mid Blue: Standard container fill...'
3  main_04  Main 04           003B94     FFFFFF       002157  'Darkest Blue: Border/outline for main elements...'
4   alt_01   Alt 01           FFF5EB     944C00       FFCCB2  'Lightest Orange: Background for contrasting...'

<class 'pandas.core.frame.DataFrame'>
RangeIndex: 15 entries, 0 to 14
Data columns (total 6 columns):
 #   Column            Non-Null Count  Dtype 
---  ------            --------------  ----- 
 0   id                15 non-null     object
 1   label             15 non-null     object
 2   background_color  15 non-null     object
 3   text_color        15 non-null     object
 4   border_color      15 non-null     object
 5   comment           15 non-null     object
dtypes: object(6)

```

### 1.2 Data definitions

* **Nodes**: Represent logical containers (partitions) or functional units (flow nodes).
* `id`: unique identifier.
* `label`: display name.
* `layer`: reference to a layer id.
* `is_partition`: boolean (true for hierarchy containers).
* `belongs_to`: reference to the parent partition id.


* **Edges**: Represent flow between nodes.
* `source` / `target`: references to node ids where `is_partition` is `false`.
* `layer`: the layer context of the flow.


* **Layers**: Define the visual aesthetics and vertical grouping (see sample above).

## 2. Architectural overview

The application is built as a React-based single-page application (SPA).

* **React**: Handles state management, CSV parsing, and the user interface for layout tweaking.
* **A-Frame**: Renders the 3D scene, handles camera controls, and manages the lifecycle of 3D objects.
* **D3-hierarchy**: Used to compute the nested spatial bounds for the `belongs_to` partition structure.

## 3. Spatial layout algorithm

The visualiser maps the graph into a Cartesian coordinate system (x, y, z).

### 3.1 The vertical axis (y)

The vertical position is determined by the layer index.



Partitions are rendered as semi-transparent "pillars" or stacked boxes that span the vertical range of their contained nodes.

### 3.2 The horizontal plane (xz)

The horizontal layout uses a recursive squarified treemap algorithm to partition space based on the `belongs_to` hierarchy.

1. The root partition is assigned the maximum bounds.
2. Each partition divides its allocated rectangle among its children.
3. A `partitionPadding` constant creates gutters between blocks for edge routing.

### 3.3 Edge routing

To prevent edges from cutting through blocks, an orthogonal routing system is used:

1. **Exit point**: Edge starts at the centre of a node face.
2. **Waypoints**: The path follows the grid lines between partition boundaries (gutters).
3. **Visualisation**: Rendered using A-Frame `meshline` or tube geometries.

## 4. Implementation code samples

### 4.1 Layout engine (TypeScript)

```typescript
// layoutEngine.ts
import { hierarchy, treemap } from 'd3-hierarchy';

export const computeLayout = (nodes, layers, config) => {
  const { layerSpacing, partitionPadding, canvasSize } = config;

  // Build tree from belongs_to
  const rootData = nodes.find(n => n.is_partition && !n.belongs_to);
  const tree = hierarchy(rootData, (d) => nodes.filter(n => n.belongs_to === d.id)).count();

  // Generate XZ coordinates
  const layoutGenerator = treemap().size([canvasSize, canvasSize]).padding(partitionPadding);
  const rootLayout = layoutGenerator(tree);

  const layerMap = new Map(layers.map((l, i) => [l.id, { ...l, index: i }]));

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

### 4.2 Visualiser component (React + A-Frame)

```jsx
// Visualiser.jsx
import React, { useMemo, useState } from 'react';
import 'aframe';
import { Entity, Scene } from 'aframe-react';
import { computeLayout } from './layoutEngine';

const NodeBlock = ({ node, layerInfo }) => (
  <Entity
    geometry={{ primitive: 'box', width: node.coords.width, height: 1, depth: node.coords.depth }}
    position={`${node.coords.x} ${node.coords.y} ${node.coords.z}`}
    material={{ color: `#${layerInfo.background_color}`, opacity: 0.9, transparent: true }}
  >
    <Entity text={{ value: node.label, align: 'center', width: 3 }} position="0 0.6 0" rotation="-90 0 0" />
  </Entity>
);

const ContextualCake = ({ data }) => {
  const [params, setParams] = useState({ layerSpacing: 8, partitionPadding: 2, canvasSize: 100 });
  const layout = useMemo(() => computeLayout(data.nodes, data.layers, params), [data, params]);

  return (
    <div style={{ width: '100vw', height: '100vh' }}>
      <Scene>
        <Entity primitive="a-sky" color="#f0f0f0" />
        {layout.map(node => (
          !node.is_partition ? (
            <NodeBlock key={node.id} node={node} layerInfo={data.layers.find(l => l.id === node.layer)} />
          ) : (
            <Entity
              key={node.id}
              geometry={{ primitive: 'box', width: node.coords.width, height: params.layerSpacing * 5, depth: node.coords.depth }}
              position={`${node.coords.x} 0 ${node.coords.z}`}
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

## 5. Visual style and interactivity

* **Interactive parameters**: Parameters like `layerSpacing` and `partitionPadding` are exposed via sliders to allow real-time structural tweaking.
* **Colour consistency**: Styling is derived from `layers.csv` to ensure consistent visual language across different hierarchy branches.
* **Transparency**: Partitions use low-opacity materials to ensure internal flow nodes remain visible.
* **Highlighting**: Hovering over a node triggers a brightness increase in its ancestors and connected edges to clarify the path within the hierarchy.
