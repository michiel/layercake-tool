# Layercake tool

## Installation

### MacOS

```
xattr -d com.apple.quarantine ./layercake
```
## Usage

### Example

Using the sample project,

```
# Generate the sample project
$ layercake generate sample kvm_control_flow
INFO layercake::generate_commands: Sample project generated successfully at: "kvm_control_flow"

# Run the sample project with a plan, this will generate the output files
$ layercake run -p kvm_control_flow/plan.yaml 
INFO layercake: Running plan: kvm_control_flow/plan.yaml

# Run the sample project with a plan, re-run the plan on input changes
$ layercake run -p kvm_control_flow/plan.yaml -w
```


#### Example linux using inotifywait

```bash
while true; \
  do inotifywait -e close_write out/kvm_control_flow.dot && \
  dot -Tpng out/kvm_control_flow.dot -o out/kvm_control_flow.png; \
done
```

## Development

### Sample run

```
cargo run -- -p sample/kvm_control_flow_plan.yaml
```

## Rendered examples

_This tool only outputs text files, the following images are rendered using other tools._

### GML rendered with Gephi

![Sample](images/sample-gml-gephi.png)

### PlantUML rendered

![Sample](images/sample-puml.png)
