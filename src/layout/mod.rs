mod actions;
mod core;
pub mod commands;

#[cfg(test)]
mod unit_tests;

pub use self::actions::movement::MovementError;
pub use self::actions::focus::FocusError;
pub use self::actions::resize::ResizeErr;
pub use self::core::GraphError;

pub use self::core::background::{Background, IncompleteBackground,
                                 MaybeBackground};
pub use self::core::action::{Action, ActionErr};
pub use self::core::container::{Container, ContainerType, Handle, Layout};
pub use self::core::tree::{Direction, TreeError};
pub use self::core::bar::Bar;
use self::core::InnerTree;
pub use self::core::MIN_SIZE;

use petgraph::graph::NodeIndex;
use rustc_serialize::json::{Json, ToJson};

use std::sync::{Mutex, MutexGuard, TryLockError, PoisonError};

/// A wrapper around tree, to hide its methods
#[derive(Debug)]
pub struct Tree(LayoutTree);
/// Mutex guard around the tree
pub type TreeGuard = MutexGuard<'static, Tree>;
/// Error for trying to lock the tree
pub type TreeErr = TryLockError<TreeGuard>;

impl Tree {
    /// Constructs a new tree.
    pub fn new() -> Self {
        Tree(LayoutTree {
            tree: InnerTree::new(),
            active_container: None
        })
    }
}


#[derive(Debug)]
pub struct LayoutTree {
    tree: InnerTree,
    active_container: Option<NodeIndex>
}

lazy_static! {
    static ref TREE: Mutex<Tree> = {
        Mutex::new(Tree::new())
    };
    static ref PREV_ACTION: Mutex<Option<Action>> = Mutex::new(None);
}

impl ToJson for LayoutTree {
    fn to_json(&self) -> Json {
        use std::collections::BTreeMap;
        fn node_to_json(node_ix: NodeIndex, tree: &LayoutTree) -> Json {
            match &tree.tree[node_ix] {
                &Container::Workspace { ref name, .. } => {
                    let mut inner_map = BTreeMap::new();
                    let children = tree.tree.children_of(node_ix).iter()
                        .map(|node| node_to_json(*node, tree)).collect();
                    inner_map.insert(format!("Workspace {}", name), Json::Array(children));
                    return Json::Object(inner_map);
                }
                &Container::Container { ref layout, id, .. } => {
                    let mut inner_map = BTreeMap::new();
                    let children = tree.tree.children_of(node_ix).iter()
                        .map(|node| node_to_json(*node, tree)).collect();
                    inner_map.insert(format!("Container w/ layout {:?} and id {:?}", layout, id),
                                     Json::Array(children));
                    return Json::Object(inner_map);
                }
                &Container::View { ref handle, id, .. } => {
                    return Json::String(format!("View: title: \"{:?}\", class: \"{:?}\", id: {}",
                                                handle.get_title(), handle.get_class(), id));
                },
                ref container => {
                    let mut inner_map = BTreeMap::new();
                    let children = tree.tree.children_of(node_ix).iter()
                        .map(|node| node_to_json(*node, tree)).collect();
                    inner_map.insert(format!("{:?}", container.get_type()),
                                     Json::Array(children));
                    return Json::Object(inner_map)
                }
            }
        }
        return node_to_json(self.tree.root_ix(), self);
    }
}

/// Attempts to lock the tree. If the Result is Err, then the lock could
/// not be returned at this time, already locked.
pub fn try_lock_tree() -> Result<TreeGuard, TreeErr> {
    let tree = try!(TREE.try_lock());
    Ok(tree)
}

pub fn lock_tree() -> Result<TreeGuard, PoisonError<TreeGuard>> {
    let tree = try!(TREE.lock());
    Ok(tree)
}

/// Attempts to lock the action mutex. If the Result is Err, then the lock could
/// not be returned at this time, already locked.
pub fn try_lock_action() -> Result<MutexGuard<'static, Option<Action>>,
                                 TryLockError<MutexGuard<'static,
                                                         Option<Action>>>> {
    PREV_ACTION.try_lock()
}
