use crate::IntoView;

use super::{ForEach, ForIndex, Rcx};

/// A namespace that contains constructor functions for various kinds of for-loops:
/// * `For::each()`
/// * `For::keyed()`
/// * `For::index()`
pub struct For;

impl For {
    /// Construct an index for loop for an array of items. The callback is called once for each
    /// array element; its arguments are the item and the array index, and its result is a View.
    /// During rebuild, the elements are overwritten based on their current array index, so the
    /// order of child views never changes.
    pub fn index<
        Item: Send + Sync + Clone + PartialEq + 'static,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        V: 'static + IntoView,
        F: Send + Sync + 'static + Fn(&Item, usize) -> V,
    >(
        item_fn: ItemFn,
        each_fn: F,
    ) -> ForIndex<Item, ItemIter, ItemFn, V, F> {
        ForIndex::new(item_fn, each_fn)
    }

    /// Construct an keyed for loop for an array of items. There are two callbacks, one which
    /// produces a unique key for each array item, and one which produces a child view for each
    /// array item. During rebuilds, the list of child views may be re-ordered based on a
    /// comparison of the generated keys.
    pub fn each_cmp<
        Item: Clone,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Fn(&Rcx) -> ItemIter,
        Cmp: Fn(&Item, &Item) -> bool,
        V: IntoView,
        F: Fn(&Item) -> V + Send,
    >(
        item_fn: ItemFn,
        cmp: Cmp,
        each: F,
    ) -> ForEach<Item, ItemIter, ItemFn, Cmp, V, F> {
        ForEach::new(item_fn, cmp, each)
    }

    /// Construct an unkeyed for loop for an array of items. The callback is called once for each
    /// array element; its argument is the item, which must be equals-comparable, and it's result
    /// is a View. During rebuild, the list of child views may be re-ordered based on a comparison
    /// of the items from the previous build.
    pub fn each<
        Item: Clone + PartialEq,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Fn(&Rcx) -> ItemIter,
        V: IntoView,
        F: Fn(&Item) -> V + Send,
    >(
        item_fn: ItemFn,
        each: F,
    ) -> ForEach<Item, ItemIter, ItemFn, impl Fn(&Item, &Item) -> bool, V, F> {
        ForEach::new(item_fn, |a, b| a == b, each)
    }
}
