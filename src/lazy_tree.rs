pub trait Seed<'a, T>
where
    Self: 'a + Sized,
    T: 'a + Clone,
{
    /// Return true when no more children can be created.
    fn is_empty(&self) -> bool;

    /// This will never be called as long as is_empty is true.
    fn children(&self) -> impl Iterator<Item = (T, Self)> + 'a;

    /// Do not use. For internal use only.
    ///
    /// This returns a vec![[]] if called with a seed that generates no
    /// children. Ideally this would not be needed as part of the trait
    /// implementation, but removing it is difficult because private trait
    /// members are not supported. The core dfs functionality could be moved to
    /// an external function.
    fn dfs_internal(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        if self.is_empty() {
            let once_empty = std::iter::once(vec![]);
            let once_empty_boxed: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(once_empty);
            once_empty_boxed
        } else {
            let result = self.children().flat_map(|(item, rest)| {
                let children = rest.dfs_internal();
                let with_item = children.map(move |mut child| {
                    child.push(item.clone());
                    child
                });
                with_item
            });
            let result_boxed: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(result);
            result_boxed
        }
    }

    /// Does a depth first traversal by recursively calling `children` until
    /// `is_empty`. Returns every path from leaf to root, in that order.
    fn dfs(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        Box::new(self.dfs_internal().filter(|i| i.len() > 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    struct Combinations<'a> {
        items: &'a Vec<char>,
        index: usize,
    }

    #[derive(Clone)]
    struct Exponential {
        value: u32,
        current_depth_index: u32,
        max_depth_index: u32,
        child_count: usize,
    }

    impl<'a> Seed<'a, u32> for Exponential {
        fn is_empty(&self) -> bool {
            self.current_depth_index > self.max_depth_index
        }

        fn children(&self) -> impl Iterator<Item = (u32, Self)> + 'a {
            let child = (
                self.value,
                Exponential {
                    current_depth_index: self.current_depth_index + 1,
                    ..*self
                },
            );
            let result = match self.is_empty() {
                true => panic!(
                    "Attempting to generate children for an empty seed. This should never happen."
                ),
                false => std::iter::repeat(child).take(self.child_count),
            };
            let boxed_result: Box<dyn Iterator<Item = (u32, Self)> + 'a> = Box::new(result);
            boxed_result
        }
    }

    impl Display for Combinations<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let chars_as_string = self
                .items
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("");
            write!(f, "{} '{}'", self.index, chars_as_string)
        }
    }

    impl<'a> Seed<'a, Option<String>> for Combinations<'a> {
        fn is_empty(&self) -> bool {
            self.index == self.items.len()
        }

        fn children(&self) -> impl Iterator<Item = (Option<String>, Self)> + 'a {
            if self.index == self.items.len() {
                let empty = std::iter::empty();
                Box::new(empty) as Box<dyn Iterator<Item = (Option<String>, Self)> + 'a>
            } else {
                let result = vec![
                    (
                        Some(self.items[self.index].to_string()),
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
                Box::new(result) as Box<dyn Iterator<Item = (Option<String>, Self)> + 'a>
            }
        }
    }

    impl Combinations<'_> {
        fn combination_as_string(c: Vec<Option<String>>) -> String {
            if c.len() == 0 {
                "(empty)".to_string()
            } else {
                let result = c
                    .into_iter()
                    .filter_map(|i| i)
                    .collect::<Vec<String>>()
                    .join(",");
                result
            }
        }

        pub fn combinations(&self) -> Vec<String> {
            let results = self
                .dfs()
                .into_iter()
                .map(Combinations::combination_as_string)
                .collect::<Vec<String>>();
            results
        }
    }

    #[test]
    fn exponential_dfs_is_completely_lazy() {
        let source = Exponential {
            child_count: 1000,
            current_depth_index: 0,
            max_depth_index: 99,
            value: 7,
        };
        let result: Vec<Vec<u32>> = source.dfs().take(1).collect();
        assert!(result[0].iter().all(|n| *n == 7));
        assert_eq!(100, result[0].len());
    }

    #[test]
    fn combinations_with_many_items() {
        let source = Combinations {
            items: &vec!['a', 'b', 'c'],
            index: 0,
        };
        let result = source.combinations();
        let expected = ["c", "b", "a", "c,b", "c,a", "b,a", "c,b,a", ""];
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
        let result = source.combinations();
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
        let result = source.combinations();
        assert!(result.is_empty());
    }
}
