use std::collections::HashMap;

trait Deconstructable<'a, T>
where
    Self: Sized,
{
    fn is_empty(&self) -> bool;
    fn items(&self) -> Vec<(&'a T, Self)>;
}

impl<'a> Deconstructable<'a, u32> for Vec<&'a u32> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn items(&self) -> Vec<(&'a u32, Self)> {
        let mut result = vec![];
        for current_index in 0..self.len() {
            let item = self.get(current_index).unwrap();
            let before_item = match current_index {
                0 => &[],
                _ => &self[0..current_index],
            };
            let after_item = match current_index == self.len() - 1 {
                true => &[],
                false => &self[current_index + 1..],
            };
            let rest = [before_item, after_item].concat();
            let part = (*item, rest);
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
    fn perms_of_nums() {
        let one = 1;
        let two = 2;
        let three = 3;
        let nums = [&one, &two, &three].to_vec();
        let results = permutations(nums);
        println!("=== Permuations of integers in a vector ===");
        results.iter().for_each(|v| {
            println!("{:?}", v);
        })
    }

    #[test]
    fn perms_of_freq_count() {
        let mut map = HashMap::new();
        let a = String::from("a");
        let b = String::from("b");
        let c = String::from("c");
        map.insert(&a, 2);
        map.insert(&b, 1);
        map.insert(&c, 2);
        let results = permutations(map);
        println!("=== Permuations by frequency count ===");
        results.iter().for_each(|v| {
            println!("{:?}", v);
        })
    }
}
