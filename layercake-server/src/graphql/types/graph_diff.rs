use async_graphql::SimpleObject;

/// Structural diff between two graphs (datasets or computed graphs).
#[derive(SimpleObject)]
pub struct GraphDiff {
    pub nodes: ItemDiff,
    pub edges: ItemDiff,
}

/// Added/removed/changed ids for one item kind (nodes or edges).
#[derive(SimpleObject)]
pub struct ItemDiff {
    /// Ids present in `to` but not `from`.
    pub added: Vec<String>,
    /// Ids present in `from` but not `to`.
    pub removed: Vec<String>,
    /// Ids present in both but with differing content.
    pub changed: Vec<String>,
    /// Count present in both and identical.
    pub unchanged: i32,
}

impl From<layercake_core::graph_diff::ItemDiff> for ItemDiff {
    fn from(d: layercake_core::graph_diff::ItemDiff) -> Self {
        Self {
            added: d.added,
            removed: d.removed,
            changed: d.changed,
            unchanged: d.unchanged as i32,
        }
    }
}

impl From<layercake_core::graph_diff::GraphDiff> for GraphDiff {
    fn from(d: layercake_core::graph_diff::GraphDiff) -> Self {
        Self {
            nodes: d.nodes.into(),
            edges: d.edges.into(),
        }
    }
}
