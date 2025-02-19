import csv
import random

# Define constants
NUM_TABLES_PER_DB = 40
NUM_LAMBDAS = 35
NUM_CONTAINERS = 10
NUM_STORED_PROCS = 15
NUM_API_GATEWAY_TARGETS = 12  # 2 containers + 10 lambdas
NUM_S3_BUCKETS = 15
DBS = ["mysql", "postgres"]

# Load resource labels from CSV
resource_labels = {
    "lambda_function": [],
    "ecs_container": [],
    "database_table": [],
    "s3_object_store": []
}

with open("resource_labels.csv", newline='') as csvfile:
    reader = csv.reader(csvfile)
    next(reader)  # Skip header
    for row in reader:
        resource_labels[row[0]].append(row[1])
print(resource_labels)

random.shuffle(resource_labels["lambda_function"])
random.shuffle(resource_labels["ecs_container"])
random.shuffle(resource_labels["database_table"])
random.shuffle(resource_labels["s3_object_store"])

# Layer colors
LAYER_COLORS = {
    "database": ("ffcccc", "990000", "660000"),
    "table": ("ffebcc", "996600", "663300"),
    "lambda": ("ccffcc", "009900", "006600"),
    "container": ("ccccff", "000099", "000066"),
    "stored_proc": ("ffccff", "990099", "660066"),
    "api_gateway": ("ffff99", "999900", "666600"),
    "s3": ("ccffff", "009999", "006666"),
    "root": ("ffffff", "000000", "000000")
}

layers = [(k, k.capitalize(), *v) for k, v in LAYER_COLORS.items()]

nodes = []

def add_node(node_id, label, layer, is_partition=False, belongs_to=None, comment=None):
    nodes.append([node_id, label, layer, str(is_partition).lower(), belongs_to or "", comment or ""])

# Root and partitions
add_node("project", "Project", "root", is_partition=True)
for part in ["database", "lambda", "container", "stored_proc", "api_gateway", "s3"]:
    add_node(part, part.capitalize(), part, is_partition=True, belongs_to="project")

# Databases and tables
for db in DBS:
    add_node(db, db.upper(), "database", is_partition=True, belongs_to="database")
    for i in range(NUM_TABLES_PER_DB):
        label = resource_labels["database_table"].pop()
        add_node(f"{db}_table_{i+1}", label, "table", belongs_to=db)

# Lambda functions
for i in range(NUM_LAMBDAS):
    label = resource_labels["lambda_function"].pop()
    add_node(f"lambda_{i+1}", label, "lambda", belongs_to="lambda")

# Containers
for i in range(NUM_CONTAINERS):
    label = resource_labels["ecs_container"].pop()
    add_node(f"container_{i+1}", label, "container", belongs_to="container")

# Stored procedures
for i in range(NUM_STORED_PROCS):
    db = random.choice(DBS)
    add_node(f"stored_proc_{i+1}", f"Stored Procedure {i+1}", "stored_proc", belongs_to=db)

# API Gateway
add_node("api_gateway_instance", "API Gateway", "api_gateway", belongs_to="api_gateway")

# S3 Buckets
for i in range(NUM_S3_BUCKETS):
    label = resource_labels["s3_object_store"].pop()
    add_node(f"s3_{i+1}", label, "s3", belongs_to="s3")

# Generate edges
edges = []
edge_counter = 1

def add_edge(source, target, label, layer, comment=None):
    global edge_counter
    edge_id = f"edge_{edge_counter}"
    edges.append([edge_id, source, target, label, layer, comment or ""])
    edge_counter += 1

# API Gateway routes
api_targets = random.sample([f"lambda_{i+1}" for i in range(NUM_LAMBDAS)] +
                            [f"container_{i+1}" for i in range(NUM_CONTAINERS)], NUM_API_GATEWAY_TARGETS)
for target in api_targets:
    add_edge("api_gateway_instance", target, "API Route", "api_gateway")

# Lambdas and Containers access tables
for func_id in [f"lambda_{i+1}" for i in range(NUM_LAMBDAS)] + [f"container_{i+1}" for i in range(NUM_CONTAINERS)]:
    for _ in range(random.randint(3, 7)):
        db = random.choice(DBS)
        table_id = f"{db}_table_{random.randint(1, NUM_TABLES_PER_DB)}"
        add_edge(func_id, table_id, "Reads/Writes", "table")

# Stored procedures interact with tables
for proc_id in [f"stored_proc_{i+1}" for i in range(NUM_STORED_PROCS)]:
    db = random.choice(DBS)
    for _ in range(random.randint(2, 5)):
        table_id = f"{db}_table_{random.randint(1, NUM_TABLES_PER_DB)}"
        add_edge(proc_id, table_id, "Executes", "stored_proc")

# API calls between containers and lambdas
for _ in range(random.randint(10, 20)):
    add_edge(random.choice([f"container_{i+1}" for i in range(NUM_CONTAINERS)]),
             random.choice([f"lambda_{i+1}" for i in range(NUM_LAMBDAS)]), "API Call", "container")

for _ in range(random.randint(10, 20)):
    add_edge(random.choice([f"lambda_{i+1}" for i in range(NUM_LAMBDAS)]),
             random.choice([f"container_{i+1}" for i in range(NUM_CONTAINERS)]), "API Call", "lambda")

# S3 Bucket Connections
s3_connections = random.sample([f"lambda_{i+1}" for i in range(NUM_LAMBDAS)] +
                               [f"container_{i+1}" for i in range(NUM_CONTAINERS)], NUM_S3_BUCKETS)
for s3, service in zip([f"s3_{i+1}" for i in range(NUM_S3_BUCKETS)], s3_connections):
    add_edge(service, s3, "Stores/Reads", "s3")

# Write CSV files
def write_csv(filename, header, rows):
    with open(filename, mode='w', newline='') as file:
        writer = csv.writer(file)
        writer.writerow(header)
        writer.writerows(rows)

write_csv("nodes.csv", ["id", "label", "layer", "is_partition", "belongs_to", "comment"], nodes)
write_csv("edges.csv", ["id", "source", "target", "label", "layer", "comment"], edges)
write_csv("layers.csv", ["id", "label", "background_color", "text_color", "border_color"], layers)

print("Graph simulation completed. CSV files generated.")
