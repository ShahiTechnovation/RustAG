//! Branching-stagenet lineage: the ancestry tree of forks-of-forks.
//!
//! Every branch records its parent and the slot it diverged at, so "what staged
//! state produced this bug?" is always answerable by walking back to the root.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rustag_core::Stagenet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ReplayError, Result};

/// A single node in the lineage tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineageNode {
    /// Branch id.
    pub id: Uuid,
    /// Parent branch id (`None` for the root).
    pub parent: Option<Uuid>,
    /// Human-friendly label.
    pub label: String,
    /// The slot this branch diverged from its parent at.
    pub branch_slot: u64,
    /// When this branch was created.
    pub created_at: DateTime<Utc>,
}

/// The ancestry tree of a family of branching stagenets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lineage {
    nodes: HashMap<Uuid, LineageNode>,
    root: Option<Uuid>,
}

impl Lineage {
    /// Start a lineage rooted at a base stagenet, returning its branch id.
    pub fn new_root(label: impl Into<String>, slot: u64) -> (Self, Uuid) {
        let id = Uuid::new_v4();
        let mut nodes = HashMap::new();
        nodes.insert(
            id,
            LineageNode {
                id,
                parent: None,
                label: label.into(),
                branch_slot: slot,
                created_at: Utc::now(),
            },
        );
        (
            Self {
                nodes,
                root: Some(id),
            },
            id,
        )
    }

    /// Record a new branch off `parent`. Errors if `parent` is unknown.
    pub fn branch(&mut self, parent: Uuid, label: impl Into<String>, slot: u64) -> Result<Uuid> {
        if !self.nodes.contains_key(&parent) {
            return Err(ReplayError::UnknownBranch(parent));
        }
        let id = Uuid::new_v4();
        self.nodes.insert(
            id,
            LineageNode {
                id,
                parent: Some(parent),
                label: label.into(),
                branch_slot: slot,
                created_at: Utc::now(),
            },
        );
        Ok(id)
    }

    /// The root branch id, if the lineage has been initialized.
    pub fn root(&self) -> Option<Uuid> {
        self.root
    }

    /// Look up a node.
    pub fn get(&self, id: Uuid) -> Option<&LineageNode> {
        self.nodes.get(&id)
    }

    /// Total number of branches.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the lineage has no branches.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Ancestors of `id`, nearest first, ending at the root (excludes `id`).
    pub fn ancestors(&self, id: Uuid) -> Vec<Uuid> {
        let mut out = Vec::new();
        let mut cur = self.nodes.get(&id).and_then(|n| n.parent);
        while let Some(p) = cur {
            out.push(p);
            cur = self.nodes.get(&p).and_then(|n| n.parent);
        }
        out
    }

    /// Direct children of `id`.
    pub fn children(&self, id: Uuid) -> Vec<Uuid> {
        let mut kids: Vec<Uuid> = self
            .nodes
            .values()
            .filter(|n| n.parent == Some(id))
            .map(|n| n.id)
            .collect();
        kids.sort_by_key(|k| self.nodes[k].created_at);
        kids
    }

    /// Depth of `id` from the root (root has depth 0).
    pub fn depth(&self, id: Uuid) -> usize {
        self.ancestors(id).len()
    }
}

/// Fork `parent` into a new offline branch and record it in `lineage` under
/// `parent_id`. This is fork-of-fork made first-class: the returned stagenet is
/// isolated, and the lineage tree captures exactly where it diverged.
pub async fn branch_stagenet(
    parent: &Stagenet,
    lineage: &mut Lineage,
    parent_id: Uuid,
    label: impl Into<String> + Clone,
) -> Result<(Stagenet, Uuid)> {
    let branch = parent.fork(&label.clone().into()).await?;
    let id = lineage.branch(parent_id, label, parent.current_slot())?;
    Ok((branch, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lineage_tracks_ancestry_and_children() {
        let (mut lin, root) = Lineage::new_root("base", 0);
        let a = lin.branch(root, "feature-a", 10).unwrap();
        let b = lin.branch(root, "feature-b", 10).unwrap();
        let a1 = lin.branch(a, "feature-a-fix", 25).unwrap();

        assert_eq!(lin.depth(root), 0);
        assert_eq!(lin.depth(a), 1);
        assert_eq!(lin.depth(a1), 2);
        assert_eq!(lin.ancestors(a1), vec![a, root]);
        assert_eq!(lin.children(root), vec![a, b]);
        assert_eq!(lin.children(a), vec![a1]);
        assert_eq!(lin.len(), 4);
    }

    #[test]
    fn branching_unknown_parent_errors() {
        let (mut lin, _root) = Lineage::new_root("base", 0);
        let ghost = Uuid::new_v4();
        assert!(matches!(
            lin.branch(ghost, "x", 1),
            Err(ReplayError::UnknownBranch(_))
        ));
    }

    #[test]
    fn lineage_json_roundtrip() {
        let (mut lin, root) = Lineage::new_root("base", 0);
        lin.branch(root, "a", 5).unwrap();
        let json = serde_json::to_string(&lin).unwrap();
        let back: Lineage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back.root(), Some(root));
    }
}
