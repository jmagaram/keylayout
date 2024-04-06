use std::time::Instant;

use set32::Set32;

mod set32;
mod utility;

fn calc_subsets(print_each: bool, max_items: u32) {
    (1..=max_items).for_each(|item_count| {
        println!("");
        println!("== Items: {} ==", item_count);
        let set = Set32::fill(item_count);
        let start = Instant::now();
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
    (0..1).for_each(|n| {
        println!("{},{}", n, n & -n);
    });
}
