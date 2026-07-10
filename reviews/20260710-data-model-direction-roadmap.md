# Layercake Data Model Direction and Roadmap Review

**Date:** 2026-07-10  
**Status:** Architecture review and recommendation  
**Scope:** Data model, semantic foundation, analysis products, presentation layers, UI, CLI, MCP, and agentic access  
**Repository:** `michiel/layercake-tool`

---

## Executive summary

Layercake already has a useful architectural skeleton:

- a Plan DAG for defining data-processing workflows;
- a unified `graph_data` persistence model for imported, computed, and manually authored graph-shaped data;
- generic nodes and edges with extensible attributes;
- project-level layer configuration and aliases;
- graph, projection, story, sequence, and artefact concepts;
- GraphQL, CLI, console, and MCP-facing code paths;
- planned query improvements for filtering, traversal, introspection, batch operations, validation, and graph analysis.

These are strong foundations for an analysis platform. The Plan DAG and GraphData distinction is particularly valuable: one describes how an analysis is produced, while the other stores data produced or consumed by that workflow.

The present model is nevertheless too graph-centric to become the long-term canonical knowledge model for Layercake's intended consulting use cases. Security assessments of complex enterprises require coherent modelling of applications, platforms, teams, roles, controls, risks, findings, evidence, lifecycle stages, operating-model responsibilities, SDLC pipelines, framework mappings, scenarios, and temporal states. These concepts can all be displayed as graphs, but they are not inherently graph records. They are typed records, classifications, and assertions that may be projected into graphs, matrices, stories, sequences, tables, scorecards, timelines, and report sections.

The principal recommendation is therefore:

> Introduce versioned datasets of typed records and reified assertions as Layercake's canonical semantic data layer; retain GraphData as a materialised graph and view representation; and evolve layers into presentation mappings over semantic classifications and dimensions.

This is an evolutionary recommendation, not a wholesale rewrite. Existing GraphData, Plan DAG, layer aliases, projections, and rendering capabilities should be retained and repositioned within a clearer architecture.

The desired long-term flow is:

```text
Sources
   ↓ ingestion
Dataset revisions
   ↓ schema mapping and identity reconciliation
Typed records, classifications, and assertions
   ↓ query and Plan DAG execution
Analysis frame
   ↓ projection
Graph / matrix / story / sequence / table / scorecard models
   ↓ theme and renderer
Interactive UI and exported artefacts
```

The most important near-term work is:

1. complete the existing single-schema graph cleanup;
2. define semantic identity, record types, and relationship types;
3. replace single-dataset source tracking with multi-source lineage;
4. separate classification from presentation layers;
5. introduce versioned view definitions and materialisations;
6. expose the same semantic application services through UI, GraphQL, CLI, and MCP;
7. build reusable composite definitions over those foundations.

This direction enables Layercake to support operating-model and control-ownership views without hardcoding each new consulting product as a bespoke data structure.

---

## 1. Review context and product intent

Layercake is intended to source data into project datasets, combine those datasets into analysis products, and maintain a consistent visual language across those products. The analysis outputs may include:

- graphs;
- matrices;
- visual maps;
- projections;
- timelines;
- sequence diagrams;
- stories and narratives;
- tables and scorecards;
- report sections;
- composite operating-model views.

The common use case is consulting work involving security assessment of complex enterprise platforms with varied:

- operating models;
- organisational structures;
- SDLC and delivery pipelines;
- application and platform estates;
- control frameworks;
- risk models;
- ownership and accountability arrangements;
- policies and standards;
- evidence sources;
- findings and recommendations;
- current-state and target-state designs.

The data foundation must support three equally important interaction modes:

1. **Human UI** — usable exploration, editing, composition, review, and visualisation.
2. **Programmatic interface** — CLI and API access suitable for repeatable workflows and automation.
3. **Agentic interface** — MCP or CLI access with schema discovery, validation, lineage, safe mutation, and explanation.

The model must therefore optimise not only for flexible storage, but also for semantic coherence, traceability, evolvability, explainability, and safe manipulation.

---

## 2. Current architecture: strengths worth preserving

### 2.1 Plan DAG and GraphData are separate concepts

The repository documents a two-level graph model:

- the **Plan DAG**, which defines data-processing workflows;
- the **Graph Data** model, which represents graph structures flowing through those workflows.

This is a sound architectural decision. It separates analysis definition from analysis content.

The Plan DAG currently models nodes such as:

- `DataSetNode`;
- `GraphNode`;
- `TransformNode`;
- `FilterNode`;
- `MergeNode`;
- `ProjectionNode`;
- `GraphArtefactNode`;
- `TreeArtefactNode`;
- `StoryNode`;
- `SequenceArtefactNode`.

That provides a natural basis for a reproducible analysis pipeline. It should remain the orchestration layer even as the underlying semantic model becomes richer.

The distinction is valuable for agentic workflows as well:

```text
Plan DAG                         Domain and analysis data
────────────────────────        ─────────────────────────
Defines operations              Contains records and assertions
Defines dependencies            Contains materialised results
Defines parameters              Contains lineage and evidence
Defines output products         Contains semantic identities
```

### 2.2 Unified GraphData is directionally correct

The `graph_data` entity unifies imported datasets, computed graphs, and manually created graphs. It replaces older parallel dataset and graph schemas. The repository's single-schema migration plan correctly identifies the operational defects caused by dual schemas and dual writes.

A unified persistence and service path reduces:

- inconsistent reads and writes;
- divergent GraphQL and service behaviour;
- duplicate editing code;
- incompatible identifiers;
- special cases in projections;
- agent confusion about which graph representation is authoritative.

This consolidation should be completed before another broad data-model expansion is introduced.

### 2.3 Internal surrogate IDs and external IDs are a sensible boundary

Graph nodes and edges use integer database keys alongside user-facing `external_id` values. This is a good pattern:

- internal IDs are efficient for persistence;
- external IDs are stable across import and export;
- users and agents can refer to meaningful identifiers;
- datasets can preserve source-native keys.

The pattern should be retained, but identity needs to be extended beyond a graph-local identifier into source-record, canonical-entity, and view-local identity.

### 2.4 JSON attributes are a useful extension mechanism

Nodes and edges support flexible JSON attributes. This is useful for:

- early domain experimentation;
- preserving source-specific fields;
- renderer hints;
- temporary extensions;
- avoiding premature schema promotion.

JSON should remain available, but it should not become the only place where domain meaning is represented. Core platform concepts such as semantic type, provenance, validity, confidence, revision, relationship semantics, and classification should be first-class.

### 2.5 Project layers and aliases align with the visual-language objective

The project layer model includes:

- a stable layer identifier;
- display name;
- background, text, and border colours;
- optional dataset association;
- aliases from source layer names to canonical project layers.

This is a strong start toward a coherent cross-product visual vocabulary. It allows heterogeneous datasets to map different source terms onto a common project concept:

```text
Dataset A: Application
Dataset B: Apps
Dataset C: Workload

          ↓ aliases

Project concept: application
```

The next step is to decouple semantic classification from literal presentation. Layers should become rules and theme roles over typed data, rather than a single scalar field embedded in every node and edge.

### 2.6 Planned query-interface improvements are well aligned

The existing query-interface plan identifies important gaps:

- filtering;
- graph traversal;
- batch operations;
- search;
- schema introspection;
- validation and dry-run;
- graph analysis;
- annotations;
- templates and cloning.

These are exactly the capabilities needed for productive agentic access. They should be implemented over shared application services so GraphQL, CLI, MCP, and the UI use the same validation and semantics.

---

## 3. Central architectural issue

The current design risks conflating four distinct concepts:

1. **source material**;
2. **canonical domain data**;
3. **graph or view representation**;
4. **rendered analysis product**.

Today, imported content, manually authored content, and computed graph content can all be stored as `graph_data`, with nodes and edges carrying labels, layers, weights, partition information, comments, a source dataset ID, and arbitrary attributes.

That is effective when the source and target are naturally graphs. It becomes limiting when the source is tabular, documentary, temporal, evidentiary, or multidimensional.

For example, the following are domain assertions:

```text
Application A is owned by Business Unit B
Control C is accountable to Role D
Control C applies during Lifecycle Stage E
Risk F is mitigated by Control C
Finding G is supported by Evidence H
Pipeline I deploys Application A
```

These assertions may be projected into:

- a responsibility graph;
- a RACI matrix;
- a lifecycle heatmap;
- a risk-to-control matrix;
- an evidence trace;
- a sequence diagram;
- a story;
- a report narrative.

The graph is one representation of the facts, not necessarily their canonical storage form.

### Recommended conceptual split

```text
Source and ingestion
  └── Source revisions and ingestion runs

Dataset layer
  └── Versioned typed records

Semantic layer
  ├── Canonical entities
  ├── Assertions and relationship types
  ├── Dimensions and classifications
  └── Provenance and evidence

Analysis layer
  └── Plan DAG and analysis definitions

View layer
  ├── Analysis frames
  ├── Graph views
  ├── Matrix views
  ├── Story views
  └── Sequence views

Presentation layer
  ├── Theme tokens
  ├── Layer rules
  └── Renderer profiles
```

GraphData should remain an important view and materialisation format within this architecture.

---

## 4. Detailed findings

## 4.1 Finding: nodes lack explicit semantic types

The current graph node model includes:

- `external_id`;
- optional label;
- optional layer;
- optional weight;
- partition state;
- parent partition reference;
- comment;
- source dataset ID;
- arbitrary attributes.

There is no first-class semantic type for concepts such as:

- application;
- platform;
- service;
- team;
- role;
- control;
- risk;
- threat;
- finding;
- evidence;
- lifecycle stage;
- pipeline;
- environment;
- policy;
- capability;
- recommendation.

In practice, the `layer` field or ad hoc JSON attributes are likely to be used as a proxy for type. That is unsafe because a layer may instead describe:

- visual grouping;
- security zone;
- organisational boundary;
- lifecycle phase;
- source dataset;
- control domain;
- assessment status.

These are independent dimensions.

### Recommendation

Introduce a first-class record or entity type:

```text
RecordType
  id
  namespace
  key
  name
  description
  schema
  identity_rules
  version
  default_style_role
```

Graph nodes that materialise records should carry `entity_type_id` or `record_type_id`.

Support three scopes:

- platform-provided common types;
- project-local types;
- source-specific imported types mapped to canonical types.

Do not require a single universal enterprise ontology. The model should permit controlled, project-local evolution.

---

## 4.2 Finding: relationships are freeform rather than semantic

Graph edges have labels, but no first-class relationship type. This makes semantically distinct assertions difficult to validate and query reliably:

```text
Role accountable_for Control
Team responsible_for Control
Control applies_to Application
Control mitigates Risk
Finding concerns Control
Evidence supports Finding
```

Freeform labels drift quickly:

```text
owns
owned by
owner
ownership
responsible for
accountable team
```

### Recommendation

Introduce controlled relationship types:

```text
RelationshipType
  id
  namespace
  key
  name
  inverse_key
  allowed_subject_types
  allowed_object_types
  cardinality
  symmetric
  transitive
  temporal
  schema
  version
```

Represent relationships as reified assertions rather than thin graph edges:

```text
Assertion
  id
  relationship_type_id
  subject_record_id
  object_record_id
  qualifiers
  status
  confidence
  valid_from
  valid_to
  created_by
  created_at
```

A GraphData edge can be materialised from an assertion, while the assertion remains the canonical semantic object.

This supports qualified statements such as:

```json
{
  "subject": "role:platform-security",
  "predicate": "accountable_for",
  "object": "control:secure-code-review",
  "qualifiers": {
    "businessUnit": "retail",
    "lifecycleStages": ["build", "test"],
    "implementationStatus": "documented"
  },
  "confidence": 0.85
}
```

---

## 4.3 Finding: provenance is limited to one dataset per element

Nodes and edges store a single optional `source_dataset_id`. Merge behaviour preserves one source using first-non-null semantics. This is useful for basic source colouring and filtering but is not sufficient for consulting-grade lineage.

A control, finding, or ownership assertion may be supported by several sources:

- a control catalogue;
- a framework mapping;
- an interview;
- a policy document;
- pipeline configuration;
- assessment evidence;
- agent inference;
- human review.

A derived conclusion may depend on several records and one or more transforms.

### Risks

Without multi-source provenance, Layercake cannot reliably answer:

- Which exact source rows support this relationship?
- Which document page supports this finding?
- Which transform created this derived entity?
- Were conflicting sources reconciled?
- Was the statement authored by a human or inferred by an agent?
- Can the analysis be regenerated from the same inputs?

### Recommendation

Introduce a many-to-many lineage model:

```text
ProvenanceLink
  id
  subject_kind
  subject_id
  source_kind
  source_id
  source_locator
  contribution_role
  confidence
  transform_run_id
  created_by
  created_at
```

Possible contribution roles include:

- asserted;
- observed;
- corroborated;
- contradicted;
- derived;
- inferred;
- reviewed.

A source locator should support:

- CSV row and column;
- JSON pointer;
- document page and paragraph;
- API object identifier;
- Git commit and path;
- interview note location.

The existing `source_dataset_id` may remain as `primary_source_dataset_id` for compatibility and efficient filtering, but it should not be the authoritative lineage model.

---

## 4.4 Finding: source files, datasets, and computed data are coupled

The current `graph_data` entity stores source file metadata and blob content alongside graph lifecycle and computed-graph metadata.

This becomes problematic as ingestion expands:

- one source file may produce several datasets;
- one dataset may combine several files;
- one source may be an API, connector, Git repository, interview, questionnaire, or agent output;
- source revisions need immutable retention;
- files may move to external object storage;
- source bytes and parsed data may have different access controls;
- a dataset may be regenerated without replacing the source artefact.

### Recommendation

Separate source capture from parsed datasets:

```text
Source
  id
  project_id
  source_type
  name
  configuration
  sensitivity

SourceRevision
  id
  source_id
  content_hash
  object_uri
  captured_at
  metadata

IngestionDefinition
  id
  source_id
  parser_type
  mapping_config
  schema_id

IngestionRun
  id
  ingestion_definition_id
  source_revision_id
  status
  warnings
  started_at
  completed_at

Dataset
  id
  project_id
  name
  schema_id

DatasetRevision
  id
  dataset_id
  content_hash
  status
  created_at
```

This provides reproducible imports, clearer retention, and cleaner security boundaries.

---

## 4.5 Finding: `source_type` combines unrelated dimensions

The discriminator values `dataset`, `computed`, and `manual` represent different dimensions:

- `dataset` is a resource category;
- `computed` is a derivation mechanism;
- `manual` is an authorship mechanism.

These are not mutually exclusive. A manually curated dataset can later be computed. A computed product may receive curated overrides. An API-imported dataset is neither simply file-based nor manual.

### Recommendation

Use orthogonal fields:

```text
resource_kind:
  dataset
  materialized_view
  graph
  matrix
  story
  sequence
  table

origin_kind:
  imported
  computed
  authored
  reconciled

mutability:
  immutable_revision
  editable
  generated

materialization_state:
  stale
  building
  ready
  failed
```

This prevents the source discriminator from becoming increasingly overloaded.

---

## 4.6 Finding: a single layer cannot represent multidimensional consulting data

The current model gives nodes and edges one optional layer. The target use cases are inherently multidimensional.

A control may simultaneously have:

- control domain: identity;
- lifecycle stage: build;
- accountable function: platform security;
- framework reference: NIST;
- risk domain: data protection;
- maturity: defined;
- criticality: high;
- geography: global.

A single scalar layer cannot represent these classifications.

### Recommendation

Introduce dimensions and classifications:

```text
Dimension
  id
  project_id
  key
  name
  value_type

DimensionValue
  id
  dimension_id
  key
  name
  order
  parent_id

Classification
  id
  record_id
  dimension_value_id
  provenance
  confidence
```

Then define presentation layers as selectors over semantic data:

```text
LayerDefinition
  id
  project_id
  key
  selector
  style_role
  priority
```

Example:

```text
selector: entity_type = control AND control_domain = identity
style_role: control.identity
```

This separates domain truth from presentation:

```text
Domain fact: lifecycle_stage = build
Theme role: lifecycle.build
Renderer style: colour, border, icon, pattern
```

The existing layer alias mechanism can be retained as a mapping from imported terms to canonical dimension values or style roles.

---

## 4.7 Finding: partitions combine presentation containment and domain hierarchy

The current graph model uses `is_partition` and `belongs_to`, and prohibits partitions from being edge endpoints. This is useful for renderer containers but conflates display grouping with domain relationships.

An application can genuinely belong to a platform or business unit. A trust zone can be a first-class domain entity. Those objects may legitimately participate in relationships.

### Recommendation

Separate domain hierarchy from view containment:

```text
Domain assertion:
  Application belongs_to Platform

View containment:
  Application node is rendered inside cluster Platform
```

Introduce view-specific containment:

```text
ViewElement
  id
  view_revision_id
  record_id
  parent_view_element_id
  layout
  style_override
```

The same semantic record may then appear:

- inside a business-unit container in an operating-model view;
- inside a trust-zone container in a security architecture view;
- ungrouped in a risk graph;
- as a row in a matrix.

No canonical entity needs to be mutated to support a different presentation.

---

## 4.8 Finding: graph-specific columns are promoted ahead of semantic requirements

Fields such as `weight`, `is_partition`, and `belongs_to` are first-class, while major consulting concepts are left in JSON attributes.

Cross-cutting concepts more deserving of first-class treatment include:

- semantic type;
- revision;
- canonical identity;
- validity interval;
- scenario;
- confidence;
- authorship;
- classification;
- provenance;
- evidence status;
- security marking;
- lifecycle state.

### Recommendation

Move graph-layout and graph-rendering properties toward view-specific models. Promote fields based on platform-wide semantic, governance, query, and lifecycle needs.

---

## 4.9 Finding: identity reconciliation is not explicit

Enterprise datasets refer to the same object differently:

```text
Azure DevOps
ADO
AzDO
Corporate Azure DevOps Platform
dev.azure.com/example
```

A graph-local external ID does not solve reconciliation across source datasets.

### Recommendation

Distinguish:

- source record identity;
- canonical entity identity;
- view-local identity.

Suggested model:

```text
CanonicalEntity
  id
  project_id
  entity_type_id
  canonical_key
  canonical_label

RecordEntityLink
  id
  record_id
  canonical_entity_id
  match_method
  confidence
  review_status
  reviewed_by
```

Match methods may include:

- exact key;
- configured alias;
- deterministic mapping;
- fuzzy match;
- agent proposal;
- human confirmation.

Merge should not silently collapse source objects. Reconciliation should be a visible, reviewable operation with preserved lineage.

---

## 4.10 Finding: temporal and scenario semantics are underdeveloped

Security consulting commonly compares:

- current state;
- target state;
- transitional state;
- historical snapshots;
- alternative organisational models;
- business-unit variants;
- pre- and post-remediation states.

Creation timestamps alone do not support these needs.

### Recommendation

Support:

```text
valid_from
valid_to
observed_at
scenario_id
baseline_id
supersedes_id
```

This enables:

- current versus target operating models;
- before/after remediation;
- assessment deltas;
- future-state projections;
- ownership changes over time;
- time-sequenced stories.

---

## 4.11 Finding: claims, evidence, observations, findings, and recommendations need distinction

Security assessment content includes several semantically different artefact types:

- claim or assertion;
- direct observation;
- evidence;
- inference;
- finding;
- recommendation;
- decision;
- exception.

Treating these as generic nodes with comments makes review and agentic usage less safe.

### Recommendation

At minimum, introduce assertion status and evidence semantics:

```text
asserted
observed
inferred
disputed
accepted
rejected
superseded
```

An agent-created statement should be representable as:

```text
status: inferred
confidence: 0.72
evidence: [source locators]
author: agent run
review status: pending
```

A human can then confirm or reject it without destroying the original derivation trail.

---

## 5. Recommended target model

The target model should be organised into bounded areas rather than one universal graph table.

## 5.1 Sources and ingestion

Core resources:

```text
Source
SourceRevision
IngestionDefinition
IngestionRun
SourceLocator
```

Responsibilities:

- capture origin;
- retain immutable revisions;
- configure parsing and mapping;
- reproduce imports;
- capture quality warnings;
- connect parsed values to source locations.

## 5.2 Datasets and records

Core resources:

```text
Dataset
DatasetRevision
RecordType
Record
FieldDefinition
SchemaVersion
```

Datasets should not need to be graph-shaped. Examples include:

- controls;
- applications;
- roles;
- lifecycle stages;
- findings;
- interview observations;
- pipeline inventory;
- framework catalogue;
- risk register.

A pragmatic first implementation can store typed record values in JSONB validated against a versioned schema:

```text
Record
  id
  dataset_revision_id
  external_id
  record_type_id
  values JSONB
  content_hash
  created_at
```

## 5.3 Assertions and relationships

Core resources:

```text
RelationshipType
Assertion
AssertionQualifier
AssertionEvidence
```

Assertions provide relationship semantics, validity, confidence, source lineage, and review state.

## 5.4 Dimensions, taxonomies, and classifications

Core resources:

```text
Dimension
DimensionValue
Classification
TaxonomyMapping
```

Use this for:

- lifecycle stages;
- control domains;
- frameworks;
- risk categories;
- organisational units;
- maturity;
- severity;
- geography;
- environment;
- security zones.

## 5.5 Views and materialisations

Core resources:

```text
ViewDefinition
ViewRevision
ViewParameter
AnalysisFrame
Materialization
ViewElement
ViewRelation
```

Supported product kinds should be peers:

```text
graph
matrix
table
timeline
story
sequence
heatmap
scorecard
report_section
dashboard
```

A view definition selects semantic data and maps it into a product structure.

## 5.6 Themes and semantic visual roles

Core resources:

```text
Theme
ThemeToken
StyleRole
LayerRule
RendererProfile
```

Use semantic tokens rather than literal colours as the primary contract:

```text
entity.application
entity.control
entity.role
risk.critical
status.gap
lifecycle.build
ownership.accountable
```

A theme maps tokens to colours, borders, icons, shapes, and patterns.

---

## 6. GraphData's recommended role

GraphData should remain, but its role should become explicit:

> GraphData is a graph-shaped materialisation or editable graph view, not the universal canonical representation of all domain knowledge.

GraphData is well suited to:

- graph renderer input;
- graph editing sessions;
- cached graph projections;
- graph transform pipelines;
- compatibility with existing projections;
- import/export of graph-native formats;
- graph-specific analytics.

GraphData nodes and edges should be able to reference canonical semantic objects:

```text
GraphDataNode
  ...existing fields...
  record_id optional
  assertion_id optional
  entity_type_id optional

GraphDataEdge
  ...existing fields...
  assertion_id optional
  relationship_type_id optional
```

View-local properties such as layout, containment, and style overrides can remain with GraphData or a related view-element table.

This approach avoids discarding existing implementation while preventing GraphData from accumulating every future domain concern.

---

## 7. Analysis frame and product projections

A key architectural addition should be an intermediate **AnalysisFrame**.

An AnalysisFrame is a resolved set of:

- records;
- assertions;
- dimensions;
- classifications;
- metrics;
- provenance;
- parameters;
- scenarios.

Example:

```text
AnalysisFrame: Control Operating Model
  controls
  roles
  teams
  lifecycle stages
  ownership assertions
  accountability assertions
  risk assertions
  evidence assertions
  classifications
```

Product projectors then consume the same frame:

```text
GraphProjector → GraphData
MatrixProjector → MatrixView
StoryProjector → StoryView
SequenceProjector → SequenceView
TableProjector → TableView
```

This avoids each renderer independently querying and interpreting raw data. It also ensures that coordinated products use the same selected facts and parameters.

---

## 8. Composite objects

Composite consulting objects should be definitions over foundational records and relationships, not monolithic copied data structures.

A composite should contain:

- input references;
- expected record and relationship types;
- query and selection rules;
- dimensions;
- product mappings;
- parameters;
- presentation rules;
- optional curated overrides;
- materialisation metadata.

### Example: control operating model across lifecycle stages

```yaml
kind: operating-model-control-ownership
version: 1

inputs:
  controls: dataset:controls
  roles: dataset:roles
  teams: dataset:teams
  lifecycleStages: taxonomy:sdlc-stages
  ownership: relationship:responsible-for
  accountability: relationship:accountable-for
  risks: dataset:risks
  findings: dataset:findings

frame:
  entityTypes:
    - control
    - role
    - team
    - risk
    - finding
  relationships:
    - responsible-for
    - accountable-for
    - applies-during
    - mitigates
    - concerns

dimensions:
  rows:
    entityType: control
  columns:
    taxonomy: sdlc-stages

cell:
  primary:
    relationship: responsible-for
  secondary:
    relationship: accountable-for
  indicators:
    - relationship: mitigates
    - relatedEntityType: finding

presentation:
  layerBy: control-domain
  statusBy: implementation-status
  theme: project-default
```

This one composite can generate:

- ownership matrix;
- responsibility graph;
- lifecycle story;
- control-gap heatmap;
- sequence view;
- summary metrics;
- report narrative.

All generated products should reference the same canonical record and assertion IDs.

---

## 9. Plan DAG evolution

The Plan DAG should remain the execution and composition model, but its vocabulary and port contracts should broaden beyond GraphData.

### 9.1 Suggested node categories

```text
Source nodes
  SourceNode
  DatasetNode
  QueryNode

Semantic operations
  MapSchemaNode
  ReconcileIdentityNode
  ClassifyNode
  JoinNode
  AssertRelationshipNode
  ValidateNode

Analysis operations
  FilterNode
  AggregateNode
  DeriveMetricNode
  TraverseNode
  CompareNode
  ScenarioNode

Projection operations
  GraphProjectionNode
  MatrixProjectionNode
  TimelineProjectionNode
  StoryProjectionNode
  SequenceProjectionNode

Product operations
  ThemeNode
  RenderNode
  ReportSectionNode
  ExportNode
```

These do not all need to be implemented immediately. The categories establish the intended separation of concerns.

### 9.2 Typed ports

The currently documented data-flow types are heavily graph-oriented. Introduce persisted port descriptors such as:

```text
Dataset<Record<Control>>
Dataset<Record<Application>>
Assertions<Ownership>
AnalysisFrame<ControlOperatingModel>
GraphView
MatrixView
StoryView
RenderedArtefact
```

Persisted example:

```json
{
  "kind": "dataset",
  "recordTypes": ["control"],
  "schemaVersion": "control@2"
}
```

Typed ports improve:

- validation;
- UI connection affordances;
- agent introspection;
- transform discoverability;
- error reporting;
- plan reuse;
- compatibility checking.

---

## 10. UI direction

A coherent UI should distinguish source data, semantic data, analysis definitions, and products.

Recommended top-level experiences:

### 10.1 Source and ingestion workspace

- source catalogue;
- revision history;
- parser and mapping configuration;
- preview and validation;
- import warnings;
- source lineage inspection.

### 10.2 Dataset and semantic workspace

- record tables;
- typed field editors;
- identity reconciliation queue;
- relationship editor;
- classification browser;
- evidence and provenance panel;
- version comparison.

### 10.3 Analysis plan workspace

- Plan DAG editor;
- typed ports;
- node schema inspection;
- execution state;
- input/output previews;
- validation and dry-run;
- materialisation history.

### 10.4 Product workspace

- graph editor/viewer;
- matrix editor/viewer;
- story and sequence editors;
- coordinated cross-highlighting;
- theme controls;
- product parameters;
- explain-element and trace-to-source.

### 10.5 Composite library

- reusable consulting templates;
- required input contracts;
- project-local parameterisation;
- version history;
- example outputs;
- compatibility checks.

The same canonical object should open from any product. Clicking a control in a graph, matrix, story, or report should show one semantic record, its relationships, provenance, findings, and evidence.

---

## 11. CLI and MCP direction

The CLI and MCP interfaces should be resource-oriented and semantic rather than exposing only graph-node CRUD.

Suggested CLI resources:

```text
layercake source ...
layercake dataset ...
layercake record ...
layercake assertion ...
layercake taxonomy ...
layercake view ...
layercake plan ...
layercake run ...
layercake product ...
layercake theme ...
```

Examples:

```bash
layercake record search \
  --project acme \
  --type control \
  --where 'lifecycle_stage = "build"'

layercake assertion create \
  --subject role:platform-security \
  --predicate accountable_for \
  --object control:secure-code-review

layercake view materialize \
  operating-model \
  --param business-unit=retail
```

### 11.1 Shared application service layer

GraphQL, CLI, MCP, and UI actions should call the same application commands and queries. Avoid parallel implementations with different validation rules.

Suggested shared operations:

```text
DescribeSchema
SearchRecords
GetRecord
CreateRecords
CreateAssertions
ValidateDataset
ReconcileEntities
MaterializeView
ExplainProductElement
CompareRevisions
TraceLineage
```

### 11.2 Agent-safe operations

Agents need more than CRUD. Provide:

- schema discovery;
- allowed-value discovery;
- relationship constraints;
- examples;
- dry-run;
- atomic batch operations;
- idempotency keys;
- optimistic concurrency;
- diffs;
- explain-query;
- explain-derivation;
- lineage inspection;
- proposal and review states;
- undo through supersession or version rollback.

MCP should expose coarse semantic tools by default:

```text
search_records
describe_schema
create_assertions
validate_dataset
materialize_view
explain_product_element
compare_dataset_revisions
```

Low-level graph mutation can remain available as an advanced capability, but it should not be the default agent interface for canonical data.

---

## 12. Versioning, mutability, and edits

Generated results and manually curated data need explicit lifecycle semantics.

### 12.1 Immutable revisions

Prefer immutable revisions for:

- source captures;
- dataset imports;
- plan definitions;
- view definitions;
- materialised products.

Mutable top-level resources can point to the current revision.

### 12.2 Manual edits to generated data

Manual edits to generated products should not silently mutate canonical generated output.

Represent edits as:

- semantic assertions;
- curated overrides;
- view-local layout or style overrides;
- edit operations replayed over a generated base;
- new revisions.

Every edit should be attributable and explainable.

### 12.3 Optimistic concurrency

All UI, CLI, and agent mutations should support revision or ETag-style concurrency checks. This prevents agents and users overwriting each other's work.

---

## 13. Storage technology recommendation

A native graph database is not required for this direction.

PostgreSQL remains suitable with:

- relational core tables;
- JSONB for validated extensible values;
- join tables for assertions and classifications;
- recursive CTEs for traversal;
- materialised views where useful;
- full-text and JSON indexes;
- object storage for large source artefacts.

Avoid prematurely adopting:

- a graph database as the system of record;
- an RDF triple store for all data;
- a fully generic entity-attribute-value model.

The proposed model remains relational and pragmatic while supporting graph projections.

### Candidate new tables

```text
sources
source_revisions
ingestion_definitions
ingestion_runs
datasets
dataset_revisions
record_types
schema_versions
records
canonical_entities
record_entity_links
relationship_types
assertions
provenance_links
dimensions
dimension_values
classifications
view_definitions
view_revisions
materializations
themes
theme_tokens
```

Existing tables retained during the transition include:

```text
projects
plans and Plan DAG tables
graph_data
graph_data_nodes
graph_data_edges
project_layers
layer_aliases
projections and artefact metadata
```

---

## 14. Architectural principles

Adopt the following as explicit design rules:

1. **Canonical data is not presentation data.**
2. **Graphs are views of records and assertions, not the only source format.**
3. **Every derived fact can explain its lineage.**
4. **Semantic type, classification, dimension, and visual layer are distinct.**
5. **Composite products reference foundational objects rather than copying them.**
6. **Resources are versioned or revision-aware.**
7. **Agent operations are discoverable, validated, atomic, and attributable.**
8. **Manual edits are assertions or overrides, not unexplained mutation of generated results.**
9. **Products can be regenerated deterministically from definitions and input revisions.**
10. **The same semantic identifiers survive across graph, matrix, story, sequence, and report views.**
11. **Source fidelity and canonical reconciliation are both preserved.**
12. **Project-local modelling is allowed without sacrificing platform-level consistency.**

---

## 15. Recommended roadmap

## Phase 0 — Complete and stabilise the current graph foundation

### Objectives

- finish the single-schema migration;
- remove or block legacy reads and writes;
- remove dual identifiers and offset-based assumptions where possible;
- consolidate graph and dataset mutations behind one service boundary;
- document current invariants;
- add database constraints and integrity checks;
- ensure GraphQL, CLI, MCP, and projections all use the unified path.

### Deliverables

- telemetry proving no legacy access;
- legacy table removal migration;
- canonical GraphData service;
- consistent resource identifiers;
- regression suite for import, execution, edit replay, projection, and export.

### Exit criteria

- one authoritative graph persistence path;
- no dual writes;
- no hidden legacy reads;
- all graph operations use the same service layer.

---

## Phase 1 — Semantic types and schema contracts

### Objectives

- add record types and relationship types;
- add schema versions;
- define canonical resource IDs and namespaces;
- validate JSON record values against schemas;
- make schemas introspectable through application services.

### Deliverables

- `record_types` and `schema_versions`;
- `relationship_types`;
- schema description API;
- schema-aware UI editors;
- CLI/MCP schema discovery;
- import mapping from source columns to typed fields.

### Exit criteria

- records and relationships have controlled meaning;
- agents can discover required fields and allowed relationships;
- plans can declare typed input and output contracts.

---

## Phase 2 — Versioned sources, datasets, and records

### Objectives

- separate source revisions from parsed datasets;
- introduce dataset revisions;
- store typed records independently of GraphData;
- support reproducible ingestion runs;
- preserve source locators.

### Deliverables

- source and ingestion tables;
- dataset and dataset revision tables;
- record persistence and query APIs;
- import history and validation UI;
- content hashes and reproducibility metadata.

### Exit criteria

- a source revision can be reprocessed deterministically;
- datasets can evolve without losing history;
- source content and parsed records have clear lifecycle boundaries.

---

## Phase 3 — Canonical identity and provenance

### Objectives

- support cross-dataset entity reconciliation;
- support multi-source provenance;
- track transform and agent runs;
- add confidence and review states.

### Deliverables

- canonical entities;
- record-to-entity links;
- reconciliation workflow;
- provenance links;
- source-locator UI;
- explain-lineage API;
- agent proposal and human review state.

### Exit criteria

- merged entities retain every contributing source;
- users can trace a product element to exact source records;
- agent inferences are distinguishable from observations and accepted facts.

---

## Phase 4 — Dimensions, classifications, and themes

### Objectives

- model multidimensional classification;
- separate semantic classification from presentation;
- evolve project layers into selectors and style roles;
- introduce semantic theme tokens.

### Deliverables

- dimensions and dimension values;
- classification persistence;
- taxonomy mappings and aliases;
- theme and token model;
- layer rules;
- renderer profiles;
- UI for mapping imported categories to canonical values.

### Exit criteria

- one record can participate in many dimensions;
- graph, matrix, story, and sequence products share a consistent theme;
- presentation can change without mutating canonical records.

---

## Phase 5 — View definitions and analysis frames

### Objectives

- introduce versioned view definitions;
- resolve data into reusable analysis frames;
- make graph and matrix projections peers;
- track materialisation inputs and hashes.

### Deliverables

- view definition schema;
- analysis-frame service;
- graph projector using records and assertions;
- matrix projector;
- materialisation history;
- stale-result detection;
- explain-product-element capability.

### Exit criteria

- one semantic selection can generate multiple coordinated products;
- materialisations are reproducible;
- product elements reference canonical records and assertions.

---

## Phase 6 — Story, sequence, timeline, and report products

### Objectives

- add additional product projections over AnalysisFrame;
- support temporal and scenario-aware views;
- support narrative generation with source references.

### Deliverables

- story model and projector;
- sequence model and projector;
- timeline model;
- report-section model;
- coordinated selection and cross-highlighting;
- export pipelines.

### Exit criteria

- stories and sequences are derived from the same semantic facts as graphs and matrices;
- narrative elements retain provenance;
- current-state and target-state scenarios can be compared.

---

## Phase 7 — Agentic parity and composite library

### Objectives

- expose semantic operations through CLI and MCP;
- support batch, dry-run, validation, diff, and explain;
- introduce reusable composite templates;
- add project-local parameterisation and overrides.

### Deliverables

- semantic CLI command groups;
- MCP tools over shared services;
- atomic batch API;
- idempotency and optimistic concurrency;
- composite definition schema;
- operating-model/control-ownership template;
- template versioning and compatibility checks.

### Exit criteria

- agents and humans manipulate the same model with equivalent safeguards;
- common consulting products can be instantiated from reusable templates;
- new product combinations do not require new foundational schemas.

---

## 16. Prioritised decisions

The following decisions should be made early and recorded as ADRs.

### Decision 1: canonical semantic store versus universal GraphData

**Recommendation:** adopt typed records and assertions as canonical; retain GraphData as graph materialisation and graph-native import format.

### Decision 2: schema technology

**Recommendation:** use versioned platform schemas with JSON Schema-compatible validation for record values, plus explicit platform metadata for identity and relationship constraints.

### Decision 3: identity scope

**Recommendation:** distinguish source record IDs, canonical entity IDs, and view element IDs.

### Decision 4: provenance granularity

**Recommendation:** support lineage at record and assertion level, including source locators and transform runs.

### Decision 5: layer semantics

**Recommendation:** treat layers as presentation rules over dimensions and types, not as the sole classification field.

### Decision 6: generated-data edits

**Recommendation:** represent semantic edits as new assertions or overrides and presentation edits as view-local changes; avoid unexplained mutation of generated canonical results.

### Decision 7: application-service boundary

**Recommendation:** all UI, GraphQL, CLI, and MCP operations call shared commands and queries with common validation.

---

## 17. Concrete first implementation slice

A useful initial slice should prove the architecture without attempting the entire roadmap.

### Use case

Generate a control-ownership matrix and corresponding graph across SDLC lifecycle stages from several source datasets.

### Foundational types

```text
Control
Role
Team
LifecycleStage
Risk
Finding
Evidence
```

### Relationship types

```text
responsible_for
accountable_for
applies_during
mitigates
concerns
supported_by
```

### Dimensions

```text
control_domain
lifecycle_stage
implementation_status
risk_severity
```

### Inputs

- control catalogue CSV;
- role/team CSV;
- lifecycle taxonomy;
- ownership mapping;
- risk register;
- assessment findings.

### Products

1. Matrix:
   - rows: controls;
   - columns: lifecycle stages;
   - cell: responsible and accountable roles;
   - colour: control domain;
   - status marker: implementation status.

2. Graph:
   - control → owner/team;
   - control → lifecycle stage;
   - control → risk;
   - finding → control.

3. Story:
   - design responsibility;
   - build implementation;
   - test verification;
   - deployment enforcement;
   - operational monitoring.

### What this slice validates

- typed records;
- controlled relationships;
- multidimensional classification;
- multi-source provenance;
- canonical identity;
- one AnalysisFrame;
- graph and matrix projections;
- shared theming;
- explain-to-source;
- UI and CLI parity.

This is preferable to implementing abstract infrastructure without a representative consulting product.

---

## 18. Risks and mitigations

### Risk: overengineering an ontology

**Mitigation:** use project-local, versioned schemas and mappings. Provide a small common vocabulary, not a mandatory universal ontology.

### Risk: duplicating GraphData and record stores indefinitely

**Mitigation:** define clear authority. Records and assertions are canonical for semantic data; GraphData is authoritative only for graph-native content or view-local graph edits.

### Risk: migration complexity

**Mitigation:** introduce semantic references incrementally. Existing GraphData remains valid. New pipelines can materialise GraphData from records while older plans continue to execute.

### Risk: JSONB becomes unqueryable

**Mitigation:** validate against schemas, index important fields, promote high-value cross-cutting properties, and provide query compilation through application services.

### Risk: agentic mutation creates low-quality data

**Mitigation:** require provenance, status, confidence, dry-run, validation, optimistic concurrency, and review workflows.

### Risk: layer and theme migration disrupts current visuals

**Mitigation:** treat current project layers as initial style roles and introduce selector-based rules behind compatibility adapters.

### Risk: too many product models

**Mitigation:** use AnalysisFrame and a common product envelope. Implement graph and matrix first, then add product types only when their semantics differ meaningfully.

---

## 19. Success measures

The direction is successful when Layercake can demonstrate the following:

### Data coherence

- the same entity has one canonical identity across datasets;
- relationships use controlled semantics;
- source and derivation lineage are inspectable;
- revisions can be compared and reproduced.

### Product coherence

- graph, matrix, story, and sequence products reference the same semantic objects;
- theme changes apply consistently across products;
- clicking an element explains its meaning and source;
- current and target scenarios can be compared.

### Developer coherence

- one application-service layer supports UI, GraphQL, CLI, and MCP;
- schemas and operations are introspectable;
- plans have typed contracts;
- new composite products require configuration and projectors, not new canonical tables.

### Agent coherence

- agents can discover schemas and allowed operations;
- batch changes can be validated before commit;
- agent-authored assertions are attributable and reviewable;
- agents can explain how a product was derived.

### Consulting value

- a consultant can import heterogeneous source material;
- reconcile it into a coherent project model;
- create multiple coordinated products;
- trace findings and conclusions to evidence;
- reuse a composite template on another engagement.

---

## 20. Final recommendation

Layercake should preserve its existing graph execution and rendering strengths while adding a semantic knowledge layer underneath them.

The target architecture is not "replace graphs with relational tables". It is:

```text
Typed records and assertions
        ↓
Reusable analysis definitions
        ↓
Graph, matrix, story, sequence, and other projections
        ↓
Shared semantic themes and renderers
```

The current `graph_data` model remains valuable as:

- a graph materialisation;
- a graph-native editing format;
- a projection cache;
- a renderer input;
- a graph analytics substrate.

It should not be required to carry every future domain, provenance, classification, identity, temporal, and evidence concern through increasingly overloaded node, edge, layer, and JSON fields.

The most important architectural move is therefore:

> Make versioned typed records, reified assertions, classifications, and provenance the canonical consulting knowledge layer; make GraphData and other product models reproducible views of that layer; and make project layers/theme tokens a consistent presentation system across all products.

This direction creates a durable foundation for:

- security assessment knowledge models;
- control-framework mappings;
- operating-model ownership matrices;
- SDLC lifecycle views;
- risk and evidence projections;
- current-state and target-state analysis;
- sequence stories;
- reusable consulting composites;
- safe human and agentic curation;
- consistent visuals across all outputs.

It also keeps the implementation evolutionary: stabilise the current graph system, add semantic capabilities in bounded phases, prove the direction through one representative composite product, and progressively move new analysis workloads onto the richer foundation.
