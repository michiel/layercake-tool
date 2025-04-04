# Description of the project

## Graph Structure
- The graph represents a **distributed inventory management application**.
- **Nodes** include:
  - **Databases** (MySQL and PostgreSQL) with 40 tables each.
  - **Lambda functions** (35).
  - **ECS containers** (10).
  - **Stored procedures** (15).
  - **API Gateway** (routing to 2 containers and 10 Lambdas).
  - **S3 object stores** (15).
- **Partitions** organize nodes into categories (database, lambda, container, stored procedure, API Gateway, S3).
- A **root node ("Project")** links all partition nodes.

## Resource Naming
- Uses a **CSV list** of meaningful names for Lambdas, containers, tables, and S3 buckets.
- Tables are assigned names relevant to inventory management (e.g., `orders_table`, `inventory_table`).

## Connection Rules
- **Lambdas and containers** connect **exclusively** to either MySQL **or** PostgreSQL, not both.
- **Stored procedures** connect **only** to tables in a single database type.
- Every table has **at least one** connection.
- Some **"exclusive clusters"** of functions access only one or two tables and interact externally via API calls.
- API calls exist between **containers and Lambdas** (both directions).
- **S3 buckets** are connected to **either** a Lambda or container (one connection per bucket).

## Graph Output
- Generates CSV files for **nodes, edges, and layers**.
