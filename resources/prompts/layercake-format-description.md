---
title: The layercake graph format
role: context
--- 

## The layercake graph format

To define a graph, there are three data types,
- Nodes set, with a list of nodes
- Edges set, with a list of edges
- Layers set, with a list of layers that style the nodes and edges

The Nodes should have the following attributes:

- id: a unique identifier for the node, lowercase and starting with a letter
- label: a human-readable label for the node
- layer: the layer that the node belongs to
- is_partition: a boolean value, true if the node used to group other nodes.  Edges do not connect to partition nodes
- belongs_to: the id of the partition node that the node belongs to, this represents the parent node in the hierarchy, every node belongs to a partition node except root nodes (which are empty, but have is_partition:true)
- comment: an optional comment for the node, in quotes

The Edges should should have the following attributes:
- id: a unique identifier for the edge, lowercase and starting with 'e_'
- source: the id of the source node
- target: the id of the target node
- label: a human-readable label for the node
- layer: the layer that the node belongs to
- comment: an optional comment for the node, in quotes

The Layers should have the following attributes:
- id: a unique identifier for the layer
- label: a human-readable label for the layer
- background_color: the background color for the layer in hex format without the leading #
- text_color: the text color for the layer in hex format without the leading #
- border_color: the border color for the layer in hex format without the leading #

The Edges represent the flow between nodes. This can be data flow, control flow, etc. Hierarchical relationships are not represented here. Edges ONLY connect nodes that have the attribute is_partition:false

The Nodes have a belongs_to attribute. This represents the hierarchical relationship, if any. If there is no hierarchy, create a single root node (e.g. label:"Scope", is_partition:true) that every other node belongs to. Hierarchical nodes are called partition nodes (they have the attribute is_partition:true), and they do not have edges connecting to them. Hierarchical relationships often represent ownership, management structures, or operating models

nodes, edges and layers can be written as CSV files with a header row. Make sure comment strings are in quotes and that labels do not use special characers or commas. 

