pub trait Permutable<'a, T>
where
    Self: 'a + Sized,
    T: 'a + Clone,
{
    fn next(&self) -> Vec<(T, Self)>;

    fn permute(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        let next = self.next();
        if next.is_empty() {
            let once_empty = std::iter::once(vec![]);
            let once_empty_boxed: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(once_empty);
            once_empty_boxed
        } else {
            let result = self.next().into_iter().flat_map(|(item, rest)| {
                let children = rest.permute();
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

    impl<'a> Permutable<'a, Option<String>> for Combos<'a> {
        fn next(&self) -> Vec<(Option<String>, Self)> {
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

    #[test]
    fn display_empty() {
        let source = Combos {
            items: &vec![],
            index: 0,
        };
        println!("{}", source);
    }

    #[test]
    fn display_full() {
        let source = Combos {
            items: &vec!['a', 'b', 'c'],
            index: 0,
        };
        println!("{}", source);
    }

    fn normalize_result(c: Combos) -> Vec<String> {
        fn normalize_one_combo(c: Vec<Option<String>>) -> String {
            c.into_iter()
                .filter_map(|i| i)
                .collect::<Vec<String>>()
                .join("")
        }
        let results = c
            .permute()
            .into_iter()
            .map(|combo| normalize_one_combo(combo))
            .collect::<Vec<String>>();
        results
    }

    #[test]
    fn permute_with_items() {
        let source = Combos {
            items: &vec!['a', 'b', 'c'],
            index: 0,
        };
        let result = normalize_result(source);
        let expected = ["c,b,a", "b,a", "c,a", "a", "c,b", "b", "c"];
        for v in source.permute() {
            let result = v
                .into_iter()
                .filter_map(|r| r)
                .collect::<Vec<String>>()
                .join(",");
            println!("{0}", result)
        }
    }

    #[test]
    fn permute_with_one_item() {
        let source = Combos {
            items: &vec!['a'],
            index: 0,
        };
        for v in source.permute() {
            let result = v
                .into_iter()
                .filter_map(|r| r)
                .collect::<Vec<String>>()
                .join(",");
            println!("{0}", result)
        }
    }

    #[test]
    fn permute_with_zero_items() {
        let source = Combos {
            items: &vec![],
            index: 0,
        };
        for v in source.permute() {
            let result = v
                .into_iter()
                .filter_map(|r| r)
                .collect::<Vec<String>>()
                .join(",");
            println!("{0}", result)
        }
    }
}
