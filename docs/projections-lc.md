This is a comprehensive technical design document consolidating the **Extensible Media Control Architecture** with the **Layercake Graph Visualizer**.

This system, referred to as the **"Data Narrative Engine,"** enables users to explore complex 3D data structures through two distinct modes: **Free Exploration** (using standard VR inputs) and **Guided Narratives** (using the media control system to follow pre-defined data flows).

---

# Technical Design: The Data Narrative Engine

## 1. High-Level Architecture

We utilize the **"One Brain, Two Bodies"** pattern. A single React State (`XRGraphStore`) drives two simultaneous render targets: the **DOM Overlay** (for 2D screens) and the **A-Frame Scene** (for VR headsets).

### Core Components

1. **Layout Engine (The Map):** Processes CSV data using `d3-hierarchy` to generate 3D coordinates (x, y, z) for every node.
2. **Narrative Engine (The Guide):** Maps "Stories" (sequences of camera positions) to the generated coordinates.
3. **Unified Control System (The Remote):** A shared interface for controlling playback, active narrative, and layout parameters.

---

## 2. Data Pipeline & Layout Engine

The visualization relies on transforming flat CSV data into a spatial "Layercake."

### 2.1 The Coordinate System

* **Vertical (Y):** Represents **Stratification** (e.g., Software Layers, Security Levels). Fixed height steps.
* **Horizontal (XZ):** Represents **Containment** (e.g., Sub-systems, VPCs). Calculated via Squarified Treemap.

### 2.2 The Layout Algorithm (React Hook)

We encapsulate the D3 logic into a memoized hook to prevent expensive recalculations on every render.

```javascript
// src/hooks/useLayercakeLayout.js
import { useMemo } from 'react';
import { hierarchy, treemap } from 'd3-hierarchy';

export const useLayercakeLayout = (nodes, layers, config) => {
  const { canvasSize, partitionPadding, layerSpacing } = config;

  return useMemo(() => {
    // 1. Hierarchy Construction
    // Find root (nodes with no parent)
    const rootData = { id: 'root', children: nodes.filter(n => !n.belongs_to) };
    
    const buildTree = (nodeId) => {
      const children = nodes.filter(n => n.belongs_to === nodeId);
      return children.length 
        ? { id: nodeId, children: children.map(c => buildTree(c.id)), ...nodes.find(n => n.id === nodeId) }
        : nodes.find(n => n.id === nodeId);
    };
    
    const d3Root = hierarchy(buildTree('root'))
      .sum(d => d.value || 1) // Size by 'value' or equal weight
      .sort((a, b) => b.value - a.value);

    // 2. 2D Treemap Calculation (XZ Plane)
    const layoutFn = treemap()
      .size([canvasSize, canvasSize])
      .paddingOuter(partitionPadding)
      .paddingInner(partitionPadding / 2);
      
    layoutFn(d3Root);

    // 3. 3D Projection (Adding Y)
    const layerMap = new Map(layers.map((l, i) => [l.id, { ...l, yIndex: i }]));
    
    return d3Root.descendants().map(d => {
      const layer = layerMap.get(d.data.layer) || { yIndex: 0, color: '888888' };
      
      return {
        id: d.data.id,
        isPartition: !!d.children, // Has children = container
        x: (d.x0 + d.x1) / 2 - canvasSize / 2, // Center on origin
        y: layer.yIndex * layerSpacing,
        z: (d.y0 + d.y1) / 2 - canvasSize / 2,
        width: d.x1 - d.x0,
        depth: d.y1 - d.y0,
        height: d.children ? (layers.length * layerSpacing) : 1, // Partitions span all layers
        color: `#${layer.background_color}`,
        label: d.data.label
      };
    }).filter(d => d.id !== 'root'); // Remove virtual root
  }, [nodes, layers, canvasSize, partitionPadding, layerSpacing]);
};

```

---

## 3. The Visualizer Implementation (`GraphScene.js`)

This component renders the graph and integrates the "Dolly" movement system from our previous research.

### 3.1 Edge Routing Strategy

To prevent messy "spaghetti" lines, we implement the **Manhattan/Orthogonal Routing** described in your requirements:

* **Path:** Node A (Source) \to Gutter XZ \to Vertical Lift \to Node B (Target).

```javascript
// src/components/GraphScene.js
import React, { useRef } from 'react';
import { useLayercakeLayout } from '../hooks/useLayercakeLayout';
import { useNarrativeControl } from '../hooks/useNarrativeControl'; // See Section 4
import 'aframe';

const GraphScene = ({ data, config, activeStory }) => {
  // 1. Compute Layout
  const items = useLayercakeLayout(data.nodes, data.layers, config);
  
  // 2. Narrative Engine Hooks (Controls Camera)
  const { dollyRef, rigRef, currentLabel } = useNarrativeControl(activeStory, items);

  return (
    <a-scene embedded shadow="type: pcfsoft">
      {/* --- A. The Graph Nodes --- */}
      {items.map(item => (
        <a-entity key={item.id} position={`${item.x} ${item.y} ${item.z}`}>
          {item.isPartition ? (
             // Partition: Wireframe Pillar
             <a-box 
               width={item.width} height={item.height} depth={item.depth}
               material="color: #555; wireframe: true; opacity: 0.3"
               position={`0 ${item.height/2 - 0.5} 0`} // Pivot adjustment
             />
          ) : (
             // Leaf Node: Solid Block
             <a-box 
               width={item.width * 0.9} height={1} depth={item.depth * 0.9}
               material={`color: ${item.color}; opacity: 0.9`}
               class="clickable" // For Raycasting
             >
                {/* Billboard Label */}
                <a-entity position="0 1.2 0" rotation="0 0 0" 
                  text={`value: ${item.label}; align: center; width: 4; color: black; side: double`}
                  look-at="#rig" // Always face camera
                />
             </a-box>
          )}
        </a-entity>
      ))}

      {/* --- B. Orthogonal Edges --- */}
      {data.edges.map((edge, i) => {
        const source = items.find(n => n.id === edge.source);
        const target = items.find(n => n.id === edge.target);
        if(!source || !target) return null;
        
        // Simple 3-point orthogonal path for clarity
        const p1 = `${source.x} ${source.y} ${source.z}`;
        const p2 = `${source.x} ${target.y} ${source.z}`; // Vertical Lift
        const p3 = `${target.x} ${target.y} ${target.z}`;

        return (
          <a-entity key={i} line={`start: ${p1}; end: ${p2}; color: #555; opacity: 0.5`} >
             <a-entity line={`start: ${p2}; end: ${p3}; color: #555; opacity: 0.5`} />
          </a-entity>
        );
      })}

      {/* --- C. The Narrative Rig (Camera) --- */}
      <a-entity id="dolly" ref={dollyRef}>
          {/* Visual Marker for "Free Roam" users to see where the story is */}
          <a-sphere radius="0.3" color="red" visible={!config.isRideMode} opacity="0.5" />
      </a-entity>

      <a-entity id="rig" ref={rigRef}>
        <a-camera look-controls wasd-controls="enabled: false"> 
            <a-cursor color="yellow" raycaster="objects: .clickable" />
            
            {/* HUD: Head-Locked Info Panel */}
            <a-entity position="0 -0.5 -1" geometry="primitive: plane; width: 1.2; height: 0.15" material="color: #000; opacity: 0.8">
               <a-text value={currentLabel} align="center" color="#4CC3D9" width="2"></a-text>
            </a-entity>
        </a-camera>
        <a-entity laser-controls="hand: right" raycaster="objects: .clickable" />
      </a-entity>

    </a-scene>
  );
};

```

---

## 4. The Extensible Control Interface

This is where the previous "Media Control" work integrates. We define "Stories" not as arbitrary coordinates, but as sequences of **Node IDs**.

### 4.1 Narrative Configuration

Instead of hardcoding positions, we define stories semantically. The system calculates the camera path dynamically based on where the Layout Engine placed those nodes.

```javascript
// src/data/stories.js
export const STORIES = [
  {
    id: "trace_auth",
    title: "🔐 Authentication Flow",
    theme: "warning",
    steps: [
      { targetNode: "gateway_01", duration: 3000, label: "1. User hits API Gateway" },
      { targetNode: "auth_svc", duration: 2000, label: "2. Token Validation" },
      { targetNode: "db_users", duration: 4000, label: "3. User Record Lookup" }
    ]
  },
  {
    id: "incident_rep",
    title: "🔥 Incident Report: 10:42AM",
    theme: "error",
    steps: [
      { targetNode: "payment_api", duration: 3000, label: "Alert Triggered Here" },
      { targetNode: "legacy_db", duration: 5000, label: "Root Cause: High Latency" }
    ]
  }
];

```

### 4.2 The Unified Controller (`XRDashboard.js`)

Combines the overlay UI and the 3D control panel into one logic block.

```javascript
import { CONTROLS } from './controls.config'; // From previous design

const XRDashboard = ({ actions, playbackState }) => {
  return (
    <>
      {/* 1. DOM Overlay (2D) */}
      <div className="hud-overlay">
         {CONTROLS.map(c => (
           <button onClick={() => actions.handle(c.action)}>{c.icon}</button>
         ))}
         <div className="progress-bar" style={{ width: `${playbackState.progress}%` }} />
      </div>

      {/* 2. VR World UI (3D) */}
      {/* This renders inside the A-Frame scene */}
      <ScenePortal> 
        <a-entity position="0 1 -2" rotation="-20 0 0" id="vr-dashboard">
           <a-plane color="#222" width="2" height="0.6" />
           {/* Generate 3D buttons from same config */}
           {CONTROLS.map((c, i) => (
             <a-box position={`${(i-1)*0.4} 0 0.1`} width="0.3" height="0.2" depth="0.05"
                    color={c.color} onClick={() => actions.handle(c.action)} class="clickable">
                <a-text value={c.label} align="center" z-offset="0.06" scale="0.5 0.5 0.5"/>
             </a-box>
           ))}
        </a-entity>
      </ScenePortal>
    </>
  )
}

```

---

## 5. Implementation Considerations

### 5.1 Performance (The "Thousand Node" Problem)

* **Challenge:** Rendering 1000 `<a-box>` entities in React will kill the framerate (React reconciliation overhead).
* **Solution:** Use **Instanced Mesh** for the bulk of the nodes if the count exceeds 500.
* *Implementation:* In A-Frame, utilize `a-instanced-mesh`. Map the layout calculations to a single buffer geometry rather than individual entities.
* *Trade-off:* Instanced meshes are harder to make interactive individually. For <500 nodes, keep individual entities for easier click handlers.



### 5.2 Text Readability in VR

* Text is the hardest part of VR data viz.
* **Technique:** Use `look-at="#rig"` on all text labels so they always rotate to face the user.
* **Technique:** Only render labels for nodes within a certain radius of the camera (LOD - Level of Detail).

### 5.3 Edge Crossing

* The layercake design helps, but edges can still clutter.
* **Mitigation:** Add a `transparency` slider in the Dashboard controls. Users can fade out the "Block" geometry to see the "Edge" tubes inside, essentially switching between **Structural View** and **Flow View**.

---

## 6. Implementation Plan

| Phase | Task | Output |
| --- | --- | --- |
| **1** | **Data Core** | React app parsing CSVs into `d3-hierarchy` structures. `console.log` the X/Y/Z coords. |
| **2** | **Visuals** | A-Frame scene rendering static Blocks (Nodes) and Wireframes (Partitions) based on coords. |
| **3** | **Narrative** | Implement `dollyRef` system. Hardcode a path to test smooth camera movement between nodes. |
| **4** | **Controls** | Build the `XRInterface` with Leva/HTML overlay. Connect "Next/Prev" to the Narrative engine. |
| **5** | **Edges** | Implement the orthogonal line rendering. Add logic to calculate Gutter offsets. |
| **6** | **Polish** | Add lighting, shadows, VR controller interactions, and "Ride Mode" toggle. |
