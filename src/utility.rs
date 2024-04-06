// use std::iter::once;

// struct Subsets<T> {
//     items: Vec<T>,
//     mask: usize,
// }

// let rec subsets found set =
//      match set with
//      | [] -> found
//      | head::tail ->
//         let with_x = found |> Seq.map(fun f -> head::f)
//         let found = found |> Seq.append (with_x)
//         subsets found tail

// type RESULT = Box<dyn Iterator<Item = Vec<i32>>>;

// fn combos(n: i32) -> RESULT {
//     fn from_vector(v: Vec<i32>) -> RESULT {
//         Box::new(once(v))
//     }
//     match n {
//         0 => from_vector(vec![]),
//         n => {
//             let rest = combos(n - 1);
//             let answer: RESULT = Box::new(rest.flat_map(move |without_n| {
//                 let mut with_n = without_n.clone();
//                 with_n.push(n);
//                 let with_n: RESULT = from_vector(with_n);
//                 let without_n: RESULT = from_vector(without_n);
//                 let result: RESULT = Box::new(with_n.chain(without_n));
//                 result
//             }));
//             answer
//         }
//     }
// }

// pub struct Partitions {
//     sum: i32,
//     parts: i32,
//     min: i32,
//     max: i32,
// }

// enum Partition {
//     Result(Vec<i32>),
//     Work {
//         current: Vec<i32>,
//         sum: i32,
//         parts: i32,
//         min: i32,
//         max: i32,
//     },
// }

// impl Partition {
//     fn next(&self) -> Partition {
//         match self {
//             Partition::Result(r) => r,
//             Partition::Work(s) => 0,
//         }
//     }
// }

// impl Partitions {
//     fn next(&self, result: Vec<i32>) -> Option<Vec<i32>> {
//         match self.sum {
//             0 => Some(result),
//             _ => None,
//         }
//     }
// }

// impl Iterator for Partitions {
//     type Item = Vec<i32>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let x = true;
//         match x {
//             true => Some(Vec::new()),
//             false => Some(Vec::new()),
//         }
//     }
// }

// impl Iterator for Partitions {
//     type Item = Box<dyn Iterator<Item = i32>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let x = true;
//         match x {
//             true => Some(Box::new(empty()).filter(|_| true)),
//             false => Some(Box::new(empty())),
//         }
//     }
// }

// type rec t<'a> = Done('a) | Work(unit => t<'a>)
// let work: (unit => t<'a>) => t<'a>
// let resolve: 'a => t<'a>
// let solve: t<'a> => 'a

// enum Trampoline<'a, T> {
//     Done(&'a T),
//     Work(Box<fn() -> Trampoline<'a, T>>),
// }

// fn factorial(n: i32) -> Trampoline<i32> {
//     Trampoline::Done(&n)
// }

// impl<T> Trampoline<'_, T> {
//     fn solve(&self) -> T {
//         let mut state = self;
//         // let mut solution = None;
//         loop {
//             match state {
//                 Trampoline::Done(s) => break s,
//                 Trampoline::Work(f) => state = &f(),
//             }
//         }
//         // let r = solution.unwrap();
//         // *r
//     }
// }

// let factorial = n => {
//     let rec go = (total, n) =>
//       switch n {
//       | 1 => TR.resolve(total)
//       | n => TR.work(() => go(total * n, n - 1))
//       }
//     let initialState = TR.work(() => go(1, n))
//     TR.solve(initialState)
//   }
// pub struct LinkedList<T> {
//     pub val: Option<T>,
//     pub next: Option<Box<LinkedList<T>>>,
// }

// pub fn combos(args: Partitions) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = i32>>>> {
//     match args.sum == 0 {
//         true => {
//             let q = empty();
//             Box::new(empty())
//         }
//         false => {
//             let result = Range {
//                 start: args.min,
//                 end: args.max,
//             }
//             .filter_map(move |n| {
//                 let sum = args.sum - n;
//                 let parts = args.parts - 1;
//                 match args.min * parts <= sum && n * parts > args.sum {
//                     true => Some((
//                         n,
//                         Partitions {
//                             sum: sum,
//                             parts: parts,
//                             min: args.min,
//                             max: args.max,
//                         },
//                     )),
//                     false => None,
//                 }
//             })
//             .flat_map(|(n, state)| {
//                 let remain = combos(state);
//                 let once = Box::new(once(n));
//                 let result = remain.map(move |r| r.chain(once));
//                 result
//                 // let first = once(n);
//                 // let result = remain.map(move |r| r.chain(first));
//                 // result
//             });
//             Box::new(result)
//         }
//     }
// }

// impl Iterator for Fibonacci {
//     // We can refer to this type using Self::Item
//     type Item = u32;

//     // Here, we define the sequence using `.curr` and `.next`.
//     // The return type is `Option<T>`:
//     //     * When the `Iterator` is finished, `None` is returned.
//     //     * Otherwise, the next value is wrapped in `Some` and returned.
//     // We use Self::Item in the return type, so we can change
//     // the type without having to update the function signatures.
//     fn next(&mut self) -> Option<Self::Item> {
//         let current = self.curr;

//         self.curr = self.next;
//         self.next = current + self.next;

//         // Since there's no endpoint to a Fibonacci sequence, the `Iterator`
//         // will never return `None`, and `Some` is always returned.
//         Some(current)
//     }
// }

// let partitions = (~sum, ~parts, ~min, ~max) => {
//   if sum <= 0 || parts <= 0 || min > max || max <= 0 || min <= 0 {
//     Exn.raiseError(`Invalid arguments`)
//   }
//   let expand = ((sum, parts, max)) =>
//     switch sum == 0 {
//     | true => Seq.empty
//     | false =>
//       Seq.range(max, min)->Seq.filterMap(n => {
//         let sum = sum - n
//         let parts = parts - 1
//         switch min * parts <= sum && n * parts >= sum {
//         | true => Some(n, (sum, parts, n))
//         | false => None
//         }
//       })
//     }
//   Seq.unfoldMany((sum, parts, sum), expand)
// }

// #[cfg(test)]
// mod tests {

//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;

//     // #[test]
//     // fn perf() {
//     //     let count = 25;
//     //     println!("total of {} is {}", count, combos(count).count());
//     // }

//     #[test]
//     fn perf_im() {
//         let count = 25;
//         println!("total of {} is {}", count, combos(count).count());
//     }

//     #[test]
//     fn dumb() {
//         assert_eq!(combos(6).filter(|i| i.len() > 0).count(), 63);
//     }
// }
