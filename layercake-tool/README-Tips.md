# Tips and snippets

## File watcher examples

#### Example linux using inotifywait

```bash
while true; \
  do inotifywait -e close-write out/kvm-control-flow.dot && \
  dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png; \
done
```

#### Example MacOS using built-in stat

```bash
prev_mod_time=$(stat -f "%m" out/kvm-control-flow.dot)                                                                                                                                                                                                                                                                                                [0/1306]

while true; do                                                                         
  sleep 1                                                                              
  new_mod_time=$(stat -f "%m" out/kvm-control-flow.dot)
  if [ "$new_mod_time" -ne "$prev_mod_time" ]; then
    dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png
    prev_mod_time=$new_mod_time                                                                                                                                                                                                                                                                                                                               
  fi                                                                                                                                                                                                                                                                                                                                                          
done
```

#### Example MacOS using fswatch

```bash
while true; do
  fswatch -1 -e ".*" -i "out/kvm-control-flow.dot" out/ && \
  dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png
done
```

## LLM prompts

### Explain the graph format

```
To define a graph, there are three CSV files,
- A nodes.csv file, with a list of nodes
- An edges.csv file, with a list of edges
- A layers.csv file, with a list of layers that style the nodes and edges

The nodes.csv file should have the following columns:
- id: a unique identifier for the node
- label: a human-readable label for the node
- layer: the layer that the node belongs to
- is_partition: a boolean value, true if the node used to group other nodes. Edges do not connect to partition nodes
- belongs_to: the id of the partition node that the node belongs to, this represents the parent node in the hierarchy, every node belongs to a partition node except root nodes (which are empty, but have is_partition:true)
- comment: an optional comment for the node

The edges.csv file should have the following columns:
- id: a unique identifier for the node
- source: the id of the source node
- target: the id of the target node
- label: a human-readable label for the node
- layer: the layer that the node belongs to
- comment: an optional comment for the node

The layers.csv file should have the following columns:
- id: a unique identifier for the layer
- label: a human-readable label for the layer
- background_color: the background color for the layer in hex format without the leading #
- text_color: the text color for the layer in hex format without the leading #
- border_color: the border color for the layer in hex format without the leading #

```
