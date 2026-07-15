# Guide: GraphJson schema

`updateDataSetGraphData` and a dataset's `graphJson` field use this JSON shape.
Fields marked **required** must be present or the call fails (e.g.
`missing field weight`, `missing field layers`).

```json
{
  "nodes": [ { …Node } ],
  "edges": [ { …Edge } ],
  "layers": [ { …Layer } ]
}
```

`nodes`, `edges`, and `layers` are all **required** array keys (use `[]` if empty).

## Node

| field | type | required | notes |
|-------|------|----------|-------|
| `id` | string | **yes** | unique within the dataset |
| `label` | string | **yes** | display label |
| `layer` | string | **yes** | layer id (should exist in `layers`) |
| `weight` | int | **yes** | 1 is a sensible default |
| `is_partition` | bool | no (default false) | partition/container node |
| `belongs_to` | string\|null | no | parent partition id |
| `comment` | string\|null | no | surfaced in some renders |
| `dataset` | int | no | source dataset id |
| `attributes` | object | no | free-form |

## Edge

| field | type | required | notes |
|-------|------|----------|-------|
| `id` | string | **yes** | unique; stories reference edges by this id |
| `source` | string | **yes** | node id |
| `target` | string | **yes** | node id |
| `label` | string | **yes** | message/edge label |
| `weight` | int | **yes** | |
| `layer` | string | **yes** | |
| `comment` | string\|null | no | appended to sequence messages as `label: comment` |
| `dataset` | int | no | |
| `attributes` | object | no | |

**Give every edge a stable `id`.** Stories store `edgeId` references; an edge
without an `id` is keyed by `source:target` and is fragile to reorder.

## Layer

| field | type | required | notes |
|-------|------|----------|-------|
| `id` | string | **yes** | |
| `label` | string | **yes** | |
| `background_color` | string | no (defaulted) | hex, no leading `#` |
| `text_color` | string | no (defaulted) | |
| `border_color` | string | no (defaulted) | |
| `alias` | string | no | |

## Tip

Dump an existing dataset to see a live example:

```bash
layercake api call --query 'query($id:Int!){ dataSet(id:$id){ graphJson } }' \
  --variables '{"id": 199}'
```
