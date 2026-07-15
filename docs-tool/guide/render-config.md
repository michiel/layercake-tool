# Guide: renderConfig by target

Artefact nodes carry a `config` JSON with `renderTarget`, `outputPath`, and a
`renderConfig` object. The accepted `renderConfig` keys depend on the target.
`config` is stored as a JSON string (it bypasses schema validation today), so
this guide is the source of truth for the keys.

## Graph artefacts (GraphArtefactNode / TreeArtefactNode)

`renderTarget`: `Mermaid`, `Graphviz`, `PlantUml`, `MermaidMindmap`,
`MermaidTreemap`, … (see `schema type RenderTarget`). `renderConfig` keys:

| key | type | effect |
|-----|------|--------|
| `containNodes` | bool | group child nodes under their partition (subgraphs/clusters) |
| `orientation` | `TB`/`LR`/… | flow direction |
| `applyLayers` | bool | emit per-layer styling (classDefs / cluster colours) from the project palette |
| `builtInStyles` | `none`/`light`/`dark` | a built-in theme |
| `addNodeCommentsAsNotes` | bool | render node comments as notes |
| `notePosition` | `Left`/`Right`/… | where node-comment notes attach |
| `useNodeWeight` / `useEdgeWeight` | bool | size by weight |
| `targetOptions` | object | per-target options (e.g. mermaid `look`/`displayMode`) |
| `layerSourceStyles` | array | per-dataset layer style overrides |

**`builtInStyles` vs `applyLayers` (review N6):** they're independent.
`applyLayers` emits your project palette's per-layer `classDef`/cluster colours;
`builtInStyles` selects a base theme. With `builtInStyles: "none"` and
`applyLayers: true` you get *no* base theme but *do* get palette colours — which
is usually what you want for branded output.

## Sequence artefacts (SequenceArtefactNode)

`renderTarget`: `MermaidSequence` or `PlantUmlSequence`. **`outputPath` must
match**: `.mmd`/`.md` for MermaidSequence, `.puml`/`.txt` for PlantUmlSequence
(mismatches are now rejected). `renderConfig` keys:

| key | type | effect |
|-----|------|--------|
| `containNodes` | `"one"`/`"all"` (or bool: true→one, false→all) | `one` = a box per partition; `all` = one box round everything |
| `builtInStyles` | `none`/`light`/`dark` | theme |
| `showNotes` | bool (default true) | emit edge notes. **Notes only render when this is true** |
| `renderAllSequences` | bool (default true) | render every sequence, or only `enabledSequenceIds` |
| `enabledSequenceIds` | [int] | which sequences to render when `renderAllSequences` is false |

`useStoryLayers` (bool, sibling of `renderConfig`) applies the story's layer
colours to participants/boxes.

### Notes: position and null (review N7)

Each sequence edge ref has a `notePosition`: `Source` | `Target` | `Both`.
**If `notePosition` is null/unset, it defaults to `Both`** — `Note over
source,target`. Set it explicitly to pin a note to one lifeline.

### Examples

```jsonc
// Mermaid sequence, per-partition boxes, notes on
{"renderTarget":"MermaidSequence","outputPath":"scenario.mmd",
 "renderConfig":{"containNodes":"one","showNotes":true},"useStoryLayers":true}

// PlantUML sequence, single box, dark theme
{"renderTarget":"PlantUmlSequence","outputPath":"scenario.puml",
 "renderConfig":{"containNodes":"all","builtInStyles":"dark"}}

// Graph, palette colours but no base theme
{"renderTarget":"Mermaid","outputPath":"graph.mmd",
 "renderConfig":{"applyLayers":true,"builtInStyles":"none","containNodes":true}}
```

## For agents

If a diagram "isn't doing what I expect", check: `showNotes` (sequence notes),
`applyLayers` vs `builtInStyles` (colours), and `containNodes` (grouping). Then
run `layercake doctor` — an empty diagram is usually an unresolved edge, not a
config problem.
