// #[derive(PartialOrd, Ord, PartialEq, Eq)]
// struct KeyLabel(i32);

// impl KeyLabel {
//     pub fn zero() -> KeyLabel {
//         KeyLabel(0)
//     }

//     pub fn increment(&self) -> KeyLabel {
//         KeyLabel(self.0 + 1)
//     }

//     pub fn to_string(&self) -> String {
//         self.0.to_string()
//     }
// }

// pub trait BitSet {
//     const EMPTY: Self;
//     fn isEmpty(&self) -> bool;
//     fn summarize(&self) -> String;
//     fn mutate(&self) -> Self;
// }

mod bits;
mod utility;

fn main() {
    // let z = Things::zero();
    // let _q = z.increment();
    // let _max: i64 = 160_787_493_266_400_000;
    // let loop_until: i64 = 1_000_000_000_000;
    // for i in 0i64..loop_until {
    //     let n: f32 = rand::random();
    //     if n < 0.00000001 {
    //         println!("Found at {}", i);
    //     }
    // }

    println!("Hello, world!");
}
