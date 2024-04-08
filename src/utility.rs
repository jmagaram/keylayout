use std::collections::HashMap;

trait Deconstructable<'a, T>
where
    Self: Sized,
{
    fn is_empty(&self) -> bool;
    fn items(&self) -> Vec<(&'a T, Self)>;
}

impl<'a> Deconstructable<'a, u32> for Vec<u32> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn items(&self) -> Vec<(&'a u32, Self)> {
        let mut result = vec![];
        for index_to_remove in 0..self.len() {
            let item = self.get(index_to_remove).unwrap();
            let starts_with = &self[0..index_to_remove];
            let ends_with = match index_to_remove == self.len() - 1 {
                true => &self[index_to_remove..],
                false => &[],
            };
            let rest = [starts_with, ends_with].concat();
            let part = (item, rest);
            result.push(part);
        }
        result
    }
}

impl<'a> Deconstructable<'a, String> for HashMap<&'a String, u32> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn items(&self) -> Vec<(&'a String, Self)> {
        let mut result = vec![];
        for i in self {
            let (k, count) = i;
            match *count {
                0 => (),
                1 => {
                    let mut copy = self.clone();
                    copy.remove(*k);
                    result.push((*k, copy))
                }
                count => {
                    let mut copy = self.clone();
                    copy.insert(*k, count - 1);
                    result.push((*k, copy))
                }
            }
        }
        result
    }
}

fn permutations<'a, T>(d: impl Deconstructable<'a, T>) -> Vec<Vec<&'a T>>
where
    T: Clone,
{
    match d.is_empty() {
        true => vec![vec![]],
        false => {
            let mut res = vec![];
            for i in d.items() {
                let (item, rest) = i;
                for j in permutations(rest) {
                    let mut x = j.to_vec();
                    x.push(item);
                    res.push(x);
                }
            }
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thisisit() {
        let mut map = HashMap::new();
        let a = String::from("a");
        let b = String::from("b");
        map.insert(&a, 2);
        map.insert(&b, 1);
        let results = permutations(map);
        results.iter().for_each(|v| {
            println!("{:?}", v);
        })
    }
}
