/// The Struct that implements this trait can be thought of as the "state" that
/// is used to lazily create a tree of items. The tree is created by recursively
/// calling the `children` function. The `children` function returns an Iterator
/// of pairs of `(value:N, state:Self)`. Every `value` along each path from root
/// to leaf is collected into in `T`, and all `T` values are returned.
///
/// N is the type of item calculated at each step in the recursive process
/// T is the type that collects each N into a result, such as a Vec  
pub trait Seed<'a, N, T>
where
    Self: 'a + Sized,
    N: 'a + Clone,
    T: 'a + Clone + Default + Eq,
{
    /// Used to collect each `N` into a `T`. The `T` value is initialized using
    /// the `Default` trait. For `i32`, for example, that would be zero. For a
    /// `Vec`, that would be an empty vector.
    fn add(result: T, item: N) -> T;

    /// Return true when at the leaf and no more children can be created.
    fn is_empty(&self) -> bool;

    /// This will never be called as long as is_empty is true.
    fn children(&self) -> impl Iterator<Item = (N, Self)> + 'a;

    /// Does a depth first traversal by recursively calling `children` until
    /// `is_empty`. Collects every item `N` along the each path from root to
    /// leaf into a T. Each `T` is initialized using the `Default` trait.
    ///
    /// **Note** If the tree is empty (no children are generated), this returns
    /// one item which is the default value of `T`.
    fn dfs_include_empty(&self) -> Box<dyn Iterator<Item = T> + 'a> {
        if self.is_empty() {
            let once_empty = std::iter::once(Default::default());
            let once_empty_boxed: Box<dyn Iterator<Item = T> + 'a> = Box::new(once_empty);
            once_empty_boxed
        } else {
            let result = self.children().flat_map(|(item, rest)| {
                let children = rest.dfs_include_empty();
                let with_item = children.map(move |child| Self::add(child, item.clone()));
                with_item
            });
            let result_boxed: Box<dyn Iterator<Item = T> + 'a> = Box::new(result);
            result_boxed
        }
    }

    /// Does a depth first traversal by recursively calling `children` until
    /// `is_empty`. Collects every item `N` along the each path from root to
    /// leaf into a T. Each `T` is initialized using the `Default` trait.
    ///
    /// **Note** If the tree is empty (no children are generated), this returns
    /// an empty Iterator.
    fn dfs(&self) -> Box<dyn Iterator<Item = T> + 'a> {
        Box::new(
            self.dfs_include_empty()
                .filter(|i| !i.eq(&Default::default())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The state used to generate all combinations of a vector of characters.
    // For example, if the items are 'a', 'b', 'c', and index starts at 0, this
    // is used to generate a, b, c, ab, ac, bc, abc.
    struct Combinations<'a> {
        items: &'a Vec<char>,
        index: usize,
    }

    impl<'a> Seed<'a, Option<char>, String> for Combinations<'a> {
        // We're done (at a leaf) when the index value is beyond the end of the
        // vector of characters.
        fn is_empty(&self) -> bool {
            self.index == self.items.len()
        }

        // Each state branches into two children, one where the current
        // character is included and one where that character is excluded.
        fn children(&self) -> impl Iterator<Item = (Option<char>, Self)> + 'a {
            if self.index == self.items.len() {
                let empty = std::iter::empty();
                Box::new(empty) as Box<dyn Iterator<Item = (Option<char>, Self)> + 'a>
            } else {
                let result = [
                    (
                        Some(self.items[self.index]),
                        Combinations {
                            index: self.index + 1,
                            ..*self
                        },
                    ),
                    (
                        None,
                        Combinations {
                            index: self.index + 1,
                            ..*self
                        },
                    ),
                ]
                .into_iter();
                Box::new(result) as Box<dyn Iterator<Item = (Option<char>, Self)> + 'a>
            }
        }

        // Adds an optional char to the result String.
        fn add(result: String, item: Option<char>) -> String {
            match item {
                None => result,
                Some(c) => {
                    let mut result = result;
                    result.push(c);
                    result
                }
            }
        }
    }

    #[test]
    fn combinations_with_many_items() {
        let source = Combinations {
            items: &vec!['a', 'b', 'c'],
            index: 0,
        };
        let result = source.dfs_include_empty().collect::<Vec<String>>();
        let expected = ["c", "b", "a", "cb", "ca", "ba", "cba", ""];
        assert_eq!(result.len(), expected.len());
        assert!(expected
            .into_iter()
            .all(|i| result.contains(&i.to_string())));
    }

    #[test]
    fn combinations_with_one_item() {
        let source = Combinations {
            items: &vec!['a'],
            index: 0,
        };
        let result = source.dfs_include_empty().collect::<Vec<String>>();
        let expected = ["a", ""];
        assert_eq!(result.len(), expected.len());
        assert!(expected
            .into_iter()
            .all(|i| result.contains(&i.to_string())));
    }

    #[test]
    fn combinations_with_empty() {
        let source = Combinations {
            items: &vec![],
            index: 0,
        };
        let result = source.dfs().collect::<Vec<String>>();
        assert!(result.is_empty());
    }
}
