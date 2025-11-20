# TODO items

- Add `resources/prompts/` to library as a Prompt type
- Add a "Preview" button and dialog to text format library items
- Currently project exports with datasets with multiple types (nodes/edges/layers) will fail to export. Fix this by always exporting datasets as Graph (JSON) and importing them accordingly
  - The error that triggered this is : "Failed to import project: Service 'AppContext::import_project_archive' error: Failed to import dataset Eligibility Threats: Invalid combination: csv format cannot contain graph data"
- To all export nodes, add a Layer configuration to enable/disable list item that can enable or disable invidiual Layer Sources (not individual layer items!). Toggling a layer source off will enable the selection of the override style (Default/Light/Dark/Grey). This will allow rendering of a graph that emphasises a specific layer (or multiple layers). In the rendering, the layer data that is assembled and provided for the template is then set accordingly.
- Add a legend to the graph formats that support it see @legend.md
- To the Data set editor, add a button "Copy nodes" next to "Paste Nodes", "Copy edges" next to "Paste edges" and "Copy layers" next to "Paste layers", where each Copy operation copies the dataset (nodes, edges or layers) to the clipboard as CSV


