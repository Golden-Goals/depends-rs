mod dep_ref;
mod dep_state;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub use dep_ref::DepRef;
pub use dep_state::DependencyState;

use super::{HashValue, NodeHash, Resolve};
use crate::execution::{error::ResolveResult, NodeState, Visitor};

/// Short-hand for a reference to a single dependency.
pub type SingleRef<'a, T> = DepRef<'a, Ref<'a, NodeState<T>>>;
/// Short-hand for a single dependency type.
pub type SingleDep<T> = Dependency<Rc<T>>;

/// Wraps a dependency and tracks the hashed value each time it's resolved. This
/// allows the resolver to know if a dependency is 'dirty' from the perspective
/// of the Dependee.
#[derive(Debug)]
pub struct Dependency<T> {
    /// The state observed of the inner dependency when it was last resolved.
    last_state: RefCell<Option<NodeHash>>,
    /// The wrapped node.
    dependency: T,
}

impl<T> Dependency<T>
where
    T: Resolve,
{
    pub fn new(dependency: T) -> Self {
        Self {
            last_state: RefCell::new(None),
            dependency,
        }
    }
}

impl<T> Resolve for Dependency<T>
where
    T: Resolve,
    for<'a> <T as Resolve>::Output<'a>: HashValue,
{
    type Output<'a>
        = DepRef<'a, T::Output<'a>>
    where
        Self: 'a;

    fn resolve(&self, visitor: &mut impl Visitor) -> ResolveResult<Self::Output<'_>> {
        let mut last_state = self.last_state.try_borrow_mut()?;
        let data = self.dependency.resolve(visitor)?;
        let current_state = data.hash_value(&mut visitor.hasher());
        if last_state.map(|s| s == current_state).unwrap_or(false) {
            Ok(DepRef::new(DependencyState::Clean, data))
        } else {
            (*last_state) = Some(current_state);
            Ok(DepRef::new(DependencyState::Dirty, data))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;

    use serial_test::serial;

    use super::*;
    use crate::execution::{
        identifiable::reset_node_id, Clean, HashSetVisitor, InputNode, IsDirty, Named, UpdateInput,
    };

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct Foo(u8);

    impl Named for Foo {
        fn name() -> &'static str {
            "Foo"
        }
    }

    impl Clean for Foo {
        fn clean(&mut self) {}
    }

    impl HashValue for Foo {
        fn hash_value(&self, hasher: &mut impl Hasher) -> NodeHash {
            hasher.write_u8(self.0);
            NodeHash::Hashed(hasher.finish())
        }
    }

    impl UpdateInput for Foo {
        type Update = u8;

        fn update_mut(&mut self, update: Self::Update) {
            self.0 = update;
        }
    }

    #[test]
    #[serial]
    fn test_dependency() {
        reset_node_id();
        let node = InputNode::new(Foo(57));
        let dependency = Dependency::new(Rc::clone(&node));
        assert_eq!(
            "Dependency { last_state: RefCell { value: None }, dependency: InputNode { resolve_state: RefCell { value: Updating }, data: RefCell { value: NodeState { node_hash: NotHashed, data: Foo(57) } }, id: 0 } }",
            format!("{:?}", dependency)
        );
        let mut visitor = HashSetVisitor::new();
        {
            let output = dependency.resolve_root(&mut visitor).unwrap();
            assert_eq!(***output, Foo(57));
            assert!(output.is_dirty());
        }
        {
            let output = dependency.resolve_root(&mut visitor).unwrap();
            assert_eq!(***output, Foo(57));
            assert!(!output.is_dirty());
        }
        node.update(42).unwrap();
        {
            let output = dependency.resolve_root(&mut visitor).unwrap();
            assert_eq!(***output, Foo(42));
            assert!(output.is_dirty());
        }
    }
}
