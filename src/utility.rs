use std::collections::HashMap;

trait Deconstructable<'a, T>
where
    Self: Sized,
{
    fn is_empty(&self) -> bool;
    fn items(&self) -> Vec<(&'a T, Self)>;
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
