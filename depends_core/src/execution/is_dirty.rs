/// For any dependee's dependencies (or single dependency), this is used to
/// check whether previously observed values have changed, indicating its stored
/// value needs to be recomputed.
pub trait IsDirty {
    fn is_dirty(&self) -> bool;
}
