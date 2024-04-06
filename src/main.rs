use std::time::Instant;

use set32::Set32;

mod partitions;
mod set32;
mod utility;

fn calc_subsets(print_each: bool, max_items: u32) {
    let start = Instant::now();
    (1..=max_items).for_each(|item_count| {
        println!("");
        println!("== Items: {} ==", item_count);
        let set = Set32::fill(item_count);
        let mut subsets_found = 0;
        (1..=item_count).for_each(|subset_size| {
            println!("  items:{} choose:{}", item_count, subset_size,);
            set.subsets_of_size(subset_size).for_each(|subset| {
                subsets_found += 1;
                match print_each {
                    true => println!("     {}", subset.to_string()),
                    false => (),
                }
            });
        });
        let duration = start.elapsed();
        println!("     subsets:{} duration: {:?}", subsets_found, duration);
    });
}

fn main() {
    calc_subsets(false, 28);
    // fn print_one(n: i32) {
    //     println!("{:08b}", n);
    //     println!("{:08b}", n & -n);
    //     println!("")
    // }
    // (0..100).for_each(|n| print_one(n));
    // print_one(-1);
}
