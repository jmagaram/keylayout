// use std::{collections::HashSet, hash::Hash};

// trait Unfoldable<T> {
//     fn next(&self) -> Option<(T, Box<dyn Iterator<Item = Self>>)>;

//     fn go(&self) -> Box<dyn Iterator<Item = Vec<T>>> {
//         match self.next() {
//             None => {
//                 let empty: Box<dyn Iterator<Item = Vec<T>>> = Box::new(std::iter::once(vec![]));
//                 empty
//             }
//             Some((item, rest)) => {
//             }

//             // std::iter::once(vec![]),
//         }
//     }
// }

// // fn perms<T>(u: Unfoldable<T>) -> Box<dyn Iterator<Item = Vec<T>>> {
// //     0
// // }

// // f: fn(i32) -> i32
// // fn perms<T>(
// //     seed: T,
// //     f: fn(T) -> Option<(T, Box<dyn Iterator<Item = Vec<T>>>)>,
// // ) -> Box<dyn Iterator<Item = Vec<T>>> {
// //     match f(seed) {
// //         None => 3,
// //         Some(seed) => {
// //             let (item, rest) = seed;
// //             let children = rest.flat_map(|mut g| {
// //                 g.push(item);
// //                 g
// //             });
// //             children
// //         }
// //     }
// // }

// // struct Permutations {
// //     items: HashSet<i32>,
// // }

// // fn permutations<T: Clone>(set: &HashSet<T>) -> impl Iterator<Item = Vec<T>>
// // where
// //     T: Eq + PartialEq + Hash,
// // {
// //     fn generate_permutations<T: Clone>(
// //         remaining: &HashSet<T>,
// //         current: Vec<T>,
// //     ) -> Box<dyn Iterator<Item = Vec<T>>>
// //     where
// //         T: Eq + PartialEq + Hash + Clone,
// //     {
// //         if remaining.is_empty() {
// //             Box::new(std::iter::once(current))
// //         } else {
// //             Box::new(remaining.iter().cloned().flat_map(move |item| {
// //                 let mut next_current = current.clone();
// //                 next_current.push(item.clone());

// //                 let mut next_remaining = remaining.clone();
// //                 next_remaining.remove(&item);

// //                 generate_permutations(&next_remaining, next_current)
// //             }))
// //         }
// //     }

// //     generate_permutations(set, Vec::new())
// // }

// // fn permutations<'a>(set: &'a HashSet<i32>) -> Box<dyn Iterator<Item = Vec<i32>> + 'a> {
// //     match set.is_empty() {
// //         true => {
// //             let result = vec![];
// //             let once = std::iter::once(result);
// //             Box::new(once) as Box<dyn Iterator<Item = Vec<i32>> + 'a>
// //         }
// //         false => {
// //             let results = set.iter().map(move |i| {
// //                 let mut copy = set.clone();
// //                 copy.remove(i);
// //                 let children = permutations(&copy);
// //                 let with_item = children.map(|mut v| {
// //                     v.push(*i);
// //                     v
// //                 });
// //                 with_item
// //                 // let m: Box<dyn Iterator<Item = Vec<i32>>> = Box::new(with_item);
// //                 // m
// //                 // let v: Vec<Vec<i32>> = adjusted.collect();
// //                 // if v.len() > 200 {
// //                 //     println!("Size: {:?}", v.len());
// //                 // }
// //                 // v
// //             });
// //             let y = results.flatten();
// //             Box::new(y) as Box<dyn Iterator<Item = Vec<i32>>>
// //         }
// //     }
// // }

// // #[cfg(test)]
// // mod tests {

// //     use super::*;

// //     #[test]
// //     fn perms_of_nums() {
// //         let mut set = HashSet::new();
// //         (1..=9).for_each(|i| {
// //             set.insert(i);
// //         });
// //         // permutations(&set)
// //         //     .take(15)
// //         //     .for_each(|i| println!("{:?}", i));
// //         let count = permutations(&set).count();
// //         println!("Count {}", count)
// //     }
// // }

// // fn permutations<'a>(set: &'a HashSet<i32>) -> Box<dyn Iterator<Item = Vec<i32>> + 'a>
// // fn permutations<'a>(set: &'a HashSet<i32>) -> Box<dyn Iterator<Item = Vec<i32>> + 'a> {
// //     match set.is_empty() {
// //         true => {
// //             let result = vec![];
// //             let once = std::iter::once(result);
// //             let boxed: Box<dyn Iterator<Item = Vec<i32>> + 'a> = Box::new(once);
// //             boxed
// //         }
// //         false => {
// //             let results = set.iter().flat_map(|i| {
// //                 let mut copy = set.clone();
// //                 copy.remove(i);
// //                 let children = permutations(&copy);
// //                 let adjusted = children.map(|mut v| {
// //                     v.push(*i);
// //                     v
// //                 });
// //                 adjusted
// //             });
// //             let boxed_results: Box<dyn Iterator<Item = Vec<i32>> + 'a> = Box::new(results);
// //             boxed_results
// //         }
// //     }
// // }

// // impl Iterator for Permutations {
// //     type Item = Vec<i32>;

// //     fn next(&mut self) -> Option<Self::Item> {
// //         match self.items.is_empty() {
// //             true => None,
// //             false => {
// //                 let mut results = vec![];
// //                 for i in self.items.iter() {
// //                     let set = self.items.clone();
// //                     set.remove(i);
// //                     let pair = ()
// //                 }
// //             }
// //         }
// //     }
// // }

// // trait UnfoldMany<T> {}

// // use std::collections::HashSet;

// // trait UnfoldMany<T>
// // where
// //     Self: Sized,
// // {
// //     fn next(&self) -> Vec<(T, Self)>;
// //     fn iter(&self) -> Iterator<Item = Vector<T>> {}

// // }

// // struct Permutations {
// //     items: HashSet<i32>,
// // }

// // impl UnfoldMany<i32> for Permutations {
// //     fn next(&self) -> Vec<(i32, Self)> {
// //         let mut results = Vec::new();
// //         for i in self.items.iter() {
// //             let mut cloned = self.items.clone();
// //             cloned.remove(&i);
// //             let state = Permutations { items: cloned };
// //             let result = (*i, state);
// //             results.push(result);
// //         }
// //         results
// //     }
// // }
