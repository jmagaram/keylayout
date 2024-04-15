pub trait Seed<'a, T>
where
    Self: 'a + Sized,
    T: 'a + Clone,
{
    fn children(&self) -> Vec<(T, Self)>;

    fn dfs_include_empty(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        let next = self.children();
        if next.is_empty() {
            let once_empty = std::iter::once(vec![]);
            let once_empty_boxed: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(once_empty);
            once_empty_boxed
        } else {
            let result = self.children().into_iter().flat_map(|(item, rest)| {
                let children = rest.dfs_include_empty();
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

    fn dfs(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        Box::new(self.dfs_include_empty().filter(|i| i.len() > 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    struct Combos<'a> {
        items: &'a Vec<char>,
        index: usize,
    }

    impl Display for Combos<'_> {
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

    impl<'a> Seed<'a, Option<String>> for Combos<'a> {
        fn children(&self) -> Vec<(Option<String>, Self)> {
            if self.index == self.items.len() {
                vec![]
            } else {
                vec![
                    (
                        Some(self.items[self.index].to_string()),
                        Combos {
                            index: self.index + 1,
                            ..*self
                        },
                    ),
                    (
                        None,
                        Combos {
                            index: self.index + 1,
                            ..*self
                        },
                    ),
                ]
            }
        }
    }

    impl Combos<'_> {
        fn format_combo(c: Vec<Option<String>>) -> String {
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

        pub fn permute_to_vec_string(&self) -> Vec<String> {
            let results = self
                .dfs()
                .into_iter()
                .map(|combo| Combos::format_combo(combo))
                .collect::<Vec<String>>();
            results
        }
    }

    #[test]
    fn permute_many_items() {
        let source = Combos {
            items: &vec!['a', 'b', 'c'],
            index: 0,
        };
        let result = source.permute_to_vec_string();
        let expected = ["c", "b", "a", "c,b", "c,a", "b,a", "c,b,a", ""];
        assert_eq!(result.len(), expected.len());
        assert!(expected
            .into_iter()
            .all(|i| result.contains(&i.to_string())));
    }

    #[test]
    fn permute_one_item() {
        let source = Combos {
            items: &vec!['a'],
            index: 0,
        };
        let result = source.permute_to_vec_string();
        let expected = ["a", ""];
        assert_eq!(result.len(), expected.len());
        assert!(expected
            .into_iter()
            .all(|i| result.contains(&i.to_string())));
    }

    #[test]
    fn permute_empty() {
        let source = Combos {
            items: &vec![],
            index: 0,
        };
        let result = source.permute_to_vec_string();
        let expected: Vec<String> = vec![];
        assert_eq!(result.len(), expected.len());
        assert!(expected
            .into_iter()
            .all(|i| result.contains(&i.to_string())));
    }
}
