pub trait Permutable<'a, T>
where
    Self: 'a + Sized,
    T: 'a + Clone,
{
    fn next(&self) -> Vec<(T, Self)>;

    fn permute(&self) -> Box<dyn Iterator<Item = Vec<T>> + 'a> {
        // let once_empty = std::iter::once(vec![]);
        // let once_empty_boxed: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(once_empty);
        // once_empty_boxed
        let result = self.next().into_iter().flat_map(|(item, rest)| {
            let children = rest.permute();
            let with_item = children.map(move |mut child| {
                child.push(item.clone());
                child
            });
            with_item
        });
        let z: Box<dyn Iterator<Item = Vec<T>> + 'a> = Box::new(result);
        z
    }
}

struct EveryLetter<'a> {
    items: &'a Vec<char>,
    index: usize,
}

impl<'a> Permutable<'a, Option<String>> for EveryLetter<'a> {
    fn next(&self) -> Vec<(Option<String>, Self)> {
        if self.index == self.items.len() {
            vec![]
        } else {
            vec![
                (
                    Some(self.items[self.index].to_string()),
                    EveryLetter {
                        items: &self.items,
                        index: self.index + 1,
                    },
                ),
                (
                    None,
                    EveryLetter {
                        items: &self.items,
                        index: self.index + 1,
                    },
                ),
            ]
        }
    }
}
