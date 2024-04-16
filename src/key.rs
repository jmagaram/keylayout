use std::{
    fmt::{self, Debug},
    ops::RangeInclusive,
};

use rand::Rng;

use crate::{item_count::ItemCount, lazy_tree::Seed, letter::Letter, permutable::Permutable, util};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Key(u32);

impl Key {
    pub const EMPTY: Key = Key(0);
    const MAX_SIZE: u32 = Letter::ALPHABET_SIZE as u32;

    pub fn with_every_letter() -> Key {
        Key((1 << Letter::ALPHABET_SIZE) - 1)
    }

    pub fn with_first_n_letters(count: u32) -> Key {
        if (count as usize) > Letter::ALPHABET_SIZE {
            panic!(
                "A Key can have at most {} letters, but you asked for {}.",
                Letter::ALPHABET_SIZE,
                count,
            );
        }
        Key((1 << count) - 1)
    }

    pub fn with_one_letter(r: Letter) -> Key {
        Key(1 << r.to_u8())
    }

    pub fn add(&self, r: Letter) -> Key {
        Key(self.0 | 1 << r.to_u8())
    }

    pub fn remove(&self, r: Letter) -> Key {
        Key(self.0 & !(1 << r.to_u8()))
    }

    pub fn contains(&self, r: Letter) -> bool {
        self.0 & (1 << r.to_u8()) != 0
    }

    pub fn count_letters(&self) -> u32 {
        self.into_iter().count() as u32 // fix!
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn except(&self, other: Key) -> Key {
        Key(self.0 & !other.0)
    }

    pub fn union(&self, other: Key) -> Key {
        Key(self.0 | other.0)
    }

    pub fn intersect(&self, other: Key) -> Key {
        Key(self.0 & other.0)
    }

    pub fn max_letter(&self) -> Option<Letter> {
        match self.0.leading_zeros() {
            u32::BITS => None,
            n => Letter::try_from(u32::BITS - n - 1).ok(),
        }
    }

    pub fn min_letter(&self) -> Option<Letter> {
        match self.0.trailing_zeros() {
            32 => None,
            n => Letter::try_from(n).ok(),
        }
    }

    pub fn random_letter(&self) -> Option<Letter> {
        let letters = self.collect::<Vec<Letter>>();
        match letters.len() {
            0 => None,
            count => {
                let index = rand::random::<usize>().rem_euclid(count);
                Some(letters[index])
            }
        }
    }

    pub fn random_subset(&self, size: RangeInclusive<u32>) -> Key {
        assert!(
            *size.start() <= self.count_letters(),
            "Can not get a minimum of {} random letters from a Key with only {} letters in it.",
            size.start(),
            self.count_letters()
        );
        assert!(
            *size.end() <= self.count_letters(),
            "Can not get a maximum of {} random letters from a Key with only {} letters in it.",
            size.end(),
            self.count_letters()
        );
        assert!(
            *size.start() <= *size.end(),
            "The range {:?} is not valid. Expected start <= end.",
            size
        );
        let mut rng = rand::thread_rng();
        let subset_size = rng.gen_range(size);
        let mut result_key = Key::EMPTY;
        let mut remain = self.clone();
        for _i in 1..=subset_size {
            let r = remain.random_letter().unwrap();
            remain = remain.remove(r);
            result_key = result_key.add(r);
        }
        result_key
    }

    pub fn random_subsets(&self, groupings: &Vec<u32>) -> impl Iterator<Item = Key> {
        debug_assert!(
            !groupings.contains(&0),
            "Every subset size must be 1 or more."
        );
        debug_assert!(
            groupings.iter().fold(0, |total, i| total + i) <= self.count_letters(),
            "The total size of all the groups exceeds the number of letters in the Key."
        );
        RandomSubsets {
            groups: groupings.to_vec(),
            remaining_letters: self.clone(), // needed?
        }
        .into_iter()
    }

    pub fn subsets_of_size(&self, size: u32) -> impl Iterator<Item = Key> {
        assert!(
            size <= Key::MAX_SIZE,
            "Expected the subset size to be 0..={} (the maximum letters in the alphabet.",
            Key::MAX_SIZE
        );
        if size == 0 {
            let result = std::iter::once(Key::EMPTY);
            let result_boxed: Box<dyn Iterator<Item = Key>> = Box::new(result);
            result_boxed
        } else {
            let letters = self.into_iter().collect::<Vec<Letter>>();
            let letters_count = letters.len();
            let max_exclusive = 1 << letters_count;
            assert!(
                letters_count >= size.try_into().unwrap(),
                "Expected the subset size ({}) to be <= the number of letters on the key ({}).",
                size,
                letters_count
            );
            let result = util::same_set_bits(size)
                .take_while(move |i| *i < max_exclusive)
                .map(move |i| {
                    Key(i as u32)
                        .into_iter()
                        .fold(Key::EMPTY, |total, i| total.add(letters[i.to_usize()]))
                });
            let result_boxed: Box<dyn Iterator<Item = Key>> = Box::new(result);
            result_boxed
        }
    }

    pub fn distribute(&self, key_sizes: ItemCount<u32>) -> impl Iterator<Item = Vec<Key>> + '_ {
        let results = key_sizes
            .permute()
            .into_iter()
            .map(|arrangement| {
                let distributor = DistributeLetters {
                    letters: *self,
                    key_sizes: arrangement,
                }
                .dfs();
                distributor
            })
            .flatten();
        results
    }
}

impl FromIterator<Letter> for Key {
    fn from_iter<I: IntoIterator<Item = Letter>>(iter: I) -> Self {
        let mut c = Key::EMPTY;
        for i in iter {
            c = c.add(i);
        }
        c
    }
}

impl TryFrom<&str> for Key {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(Letter::try_from)
            .try_fold(Key::EMPTY, |total, r| r.map(|letter| total.add(letter)))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        for c in self.into_iter() {
            result.push(c.to_char());
        }
        write!(f, "{}", result)
    }
}

// iter not into_iter?
impl Iterator for Key {
    type Item = Letter;

    fn next(&mut self) -> Option<Letter> {
        match self.0 {
            0 => None,
            _ => {
                let trailing_zeros = self.0.trailing_zeros();
                self.0 = self.0 ^ (1 << (trailing_zeros));
                Letter::try_from(trailing_zeros).ok()
            }
        }
    }
}

struct RandomSubsets {
    groups: Vec<u32>,
    remaining_letters: Key,
}

impl Iterator for RandomSubsets {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        let (group_size, remaining_groups) = self.groups.split_first()?;
        let key = self
            .remaining_letters
            .random_subset(*group_size..=*group_size);
        let remaining_letters = self.remaining_letters.except(key);
        self.groups = remaining_groups.to_vec();
        self.remaining_letters = remaining_letters;
        Some(key)
    }
}

struct DistributeLetters {
    key_sizes: Vec<u32>,
    letters: Key,
}

// can have more letters than groups
impl<'a> Seed<'a, Key> for DistributeLetters {
    fn is_empty(&self) -> bool {
        self.key_sizes.is_empty()
    }

    fn children(&self) -> impl Iterator<Item = (Key, Self)> + 'a {
        assert!(
            self.key_sizes.len() > 0,
            "Expected the list of key sizes to not be empty."
        );
        assert!(
            self.letters.count_letters() > 0,
            "Expected the number of remaining letters to be 1 or more."
        );
        debug_assert!(
            self.key_sizes.iter().all(|i| *i > 0),
            "Every key size should be 1 or more."
        );
        debug_assert!(
            self.key_sizes.iter().fold(0, |total, i| total + i) <= self.letters.count_letters(),
            "Expected the sum of all the key sizes <= the number of remaining letters."
        );
        let (current_key_size, remaining_key_sizes) = self
            .key_sizes
            .split_first()
            .expect("Could not extract the first and remaining key sizes.");
        let remaining_key_sizes = remaining_key_sizes.to_vec();
        let max_letter = self.letters.max_letter().unwrap();
        let remaining_letters = self.letters.remove(max_letter);
        let letters_to_distribute = self.letters.clone();
        let results =
            remaining_letters
                .subsets_of_size(current_key_size - 1)
                .map(move |other_letters| {
                    let new_key = other_letters.add(max_letter);
                    let remaining_letters = letters_to_distribute.except(new_key);
                    let seed = DistributeLetters {
                        key_sizes: remaining_key_sizes.clone(),
                        letters: remaining_letters,
                    };
                    (new_key, seed)
                });
        let boxed_results: Box<dyn Iterator<Item = (Key, Self)> + 'a> = Box::new(results);
        boxed_results
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        item_count::{self, ItemCount},
        keyboard::Keyboard,
        lazy_tree::Seed,
        letter::Letter,
        permutable::Permutable,
        util,
    };

    use super::*;

    fn aa() -> Letter {
        Letter::try_from('a').unwrap()
    }
    fn bb() -> Letter {
        Letter::try_from('b').unwrap()
    }
    fn cc() -> Letter {
        Letter::try_from('c').unwrap()
    }
    fn dd() -> Letter {
        Letter::try_from('d').unwrap()
    }

    #[test]
    fn try_from_str_when_valid_test() {
        for s in ["abc", "a", "abcdez\'", "xyz", ""] {
            let key: Key = s.try_into().unwrap();
            assert_eq!(key.to_string(), s.to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid_test() {
        for s in ["098", "a#4", "   "] {
            let key: Result<Key, _> = s.try_into();
            assert!(key.is_err());
        }
    }

    #[test]
    fn to_string_test() {
        let data = ["abc", "", "mnop'"];
        for s in data {
            let actual = Key::try_from(s).unwrap().to_string();
            assert_eq!(s.to_string(), actual);
        }
    }

    #[test]
    fn with_every_letter_test() {
        let target = Key::with_every_letter();
        assert_eq!(
            target.to_string(),
            "abcdefghijklmnopqrstuvwxyz'".to_string()
        );
    }

    #[test]
    fn with_first_n_letters_test() {
        let data = [(0, ""), (1, "a"), (27, "abcdefghijklmnopqrstuvwxyz'")];
        for (count, expected) in data {
            let target = Key::with_first_n_letters(count);
            assert_eq!(expected, target.to_string())
        }
    }

    #[test]
    #[should_panic]
    fn with_first_n_letters_panic_if_more_than_alphabet() {
        Key::with_first_n_letters(28);
    }

    #[test]
    fn max_letters_is_alphabet_size() {
        assert_eq!(Letter::ALPHABET_SIZE, Key::MAX_SIZE as usize);
    }

    #[test]
    fn into_iterator_test() {
        let data = ["", "abc", "a", "xyz", "az'"];
        for d in data {
            let k: Vec<String> = Key::try_from(d)
                .unwrap()
                .into_iter()
                .map(|r| r.to_string())
                .collect();
            let result = k.join("");
            assert_eq!(d, result);
        }
    }

    #[test]
    fn add_test() {
        assert_eq!(
            Key::EMPTY
                .add(aa())
                .add(bb())
                .add(cc())
                .add(dd())
                .to_string(),
            "abcd"
        );
    }

    #[test]
    fn except_test() {
        let data = [
            ("", "", ""),
            ("a", "a", ""),
            ("abc", "bc", "a"),
            ("abc", "x", "abc"),
            ("abcdefg", "afg", "bcde"),
            ("", "abc", ""),
            ("a", "abc", ""),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let except = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.except(except), expected);
        }
    }

    #[test]
    fn remove_test() {
        let data = [
            ("", 'a', ""),
            ("a", 'a', ""),
            ("abc", 'b', "ac"),
            ("abc", 'x', "abc"),
            ("abcdefg", 'e', "abcdfg"),
            ("", 'a', ""),
        ];
        for (start, to_remove, expected) in data {
            let start = Key::try_from(start).unwrap();
            let except = Letter::try_from(to_remove).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.remove(except), expected);
        }
    }

    #[test]
    fn contains_test() {
        let data = [
            ("", 'a', false),
            ("a", 'a', true),
            ("a", 'b', false),
            ("abcd", 'b', true),
            ("abcd", 'x', false),
        ];
        for (start, find, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Letter::try_from(find).unwrap();
            assert_eq!(start.contains(other), expected);
        }
    }

    #[test]
    fn count_letters_test() {
        let data = [(""), ("a"), ("abcde"), ("abcdefghijklmnopqrstuvwxyz'")];
        for start in data {
            let start = Key::try_from(start).unwrap();
            assert_eq!(start.count_letters() as usize, start.to_string().len());
        }
    }

    #[test]
    fn with_one_letter_test() {
        assert_eq!("a", Key::with_one_letter(aa()).to_string());
        assert_eq!("b", Key::with_one_letter(bb()).to_string());
        assert_eq!("c", Key::with_one_letter(cc()).to_string());
    }

    #[test]
    fn union_test() {
        let data = [
            ("", "", ""),
            ("a", "a", "a"),
            ("a", "", "a"),
            ("abc", "x", "abcx"),
            ("abcdefg", "afg", "abcdefg"),
            ("abc", "xyz", "abcxyz"),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.union(other), expected);
        }
    }

    #[test]
    fn intersect_test() {
        let data = [
            ("", "", ""),
            ("a", "a", "a"),
            ("a", "", ""),
            ("abc", "x", ""),
            ("abcdefg", "afg", "afg"),
            ("abc", "xyz", ""),
            ("abcd", "cdef", "cd"),
        ];
        for (start, other, expected) in data {
            let start = Key::try_from(start).unwrap();
            let other = Key::try_from(other).unwrap();
            let expected = Key::try_from(expected).unwrap();
            assert_eq!(start.intersect(other), expected);
        }
    }

    #[test]
    fn max_letter_test() {
        let data = [("a", 'a'), ("abc", 'c'), ("cba", 'c')];
        for (start, expected) in data {
            let start = Key::try_from(start).unwrap();
            let expected = Letter::try_from(expected).unwrap();
            assert_eq!(start.max_letter(), Some(expected));
        }
        assert!(Key::EMPTY.max_letter().is_none());
    }

    #[test]
    fn min_letter_test() {
        let data = [("a", 'a'), ("abc", 'a'), ("cba", 'a'), ("xyfwfg", 'f')];
        for (start, expected) in data {
            let start = Key::try_from(start).unwrap();
            let expected = Letter::try_from(expected).unwrap();
            assert_eq!(start.min_letter(), Some(expected));
        }
        assert!(Key::EMPTY.min_letter().is_none());
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_panic_if_bigger_than_alphabet_size() {
        let key = Key::with_every_letter();
        key.subsets_of_size(Key::MAX_SIZE + 1).take(1).count();
    }

    #[test]
    fn subsets_of_size_return_one_empty_set() {
        let key = Key::with_every_letter();
        let result = key.subsets_of_size(0).collect::<Vec<Key>>();
        assert_eq!(1, result.len());
        assert_eq!(Key::EMPTY, result[0]);
    }

    #[test]
    fn subsets_of_size_test() {
        fn test(items: &str) {
            let key = Key::try_from(items).unwrap();
            let ones_count = key.into_iter().count() as u32; // fix

            // subsets have correct number of items (no duplicates)
            (1..ones_count).for_each(|subset_size| {
                let actual_size = key.subsets_of_size(subset_size).count();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(actual_size, expected_count as usize);
            });

            // subsets items are unique
            (1..ones_count).for_each(|subset_size| {
                let set = key
                    .subsets_of_size(subset_size.into())
                    .map(|b| b.to_string())
                    .collect::<std::collections::HashSet<String>>();
                let expected_count = util::choose(ones_count as u32, subset_size as u32);
                assert_eq!(set.len(), expected_count as usize);
            });

            // subsets items are all in the source bits
            (1..ones_count).for_each(|subset_size| {
                let all_valid_items = key.subsets_of_size(subset_size.into()).all(move |subset| {
                    let m = subset.except(key) == Key::EMPTY;
                    m
                });
                assert!(all_valid_items)
            });
        }

        let data = ["abdfj", "abcdefghmpq", "apqrx", "cdexyz", "", "f"];
        for s in data {
            test(&s);
        }
    }

    #[test]
    fn from_iterator_of_letter() {
        let result = Key::from_iter(
            ['a', 'b', 'c', 'd', 'e', 'f']
                .iter()
                .map(|c| Letter::try_from(*c).unwrap()),
        );
        assert_eq!("abcdef", result.to_string());
    }

    #[test]
    fn random_letter_when_empty_return_none() {
        for _i in 1..=1000 {
            assert_eq!(None, Key::EMPTY.random_letter());
        }
    }

    #[test]
    fn random_letter_gets_every_letter_eventually() {
        let data = ["a", "abc", "abcdefghijklmnopqrtsuvwxyz'"];
        for d in data {
            let target = Key::try_from(d).unwrap();
            let found = Key::from_iter((1..=1000).map(move |_| target.random_letter().unwrap()));
            assert_eq!(target, found)
        }
    }

    #[test]
    fn random_subset_gets_every_letter_eventually() {
        let data = ["", "a", "abc", "abcdefghijklmnopqrtsuvwxyz'"];
        for key in data {
            let source = Key::try_from(key).unwrap();
            let letter_count: u32 = key.len().try_into().unwrap();
            for min_size in 0..=letter_count {
                for max_size in min_size..=letter_count {
                    let mut result = Key::EMPTY;
                    for i in 1..1000 {
                        let subset = source.random_subset(min_size..=max_size);
                        result = result.union(subset);
                        if i > 100 && result == source {
                            break;
                        }
                    }
                    if max_size == 0 {
                        assert_eq!(result, Key::EMPTY)
                    } else {
                        assert_eq!(result, source);
                    }
                }
            }
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn random_subsets_panic_if_more_groups_than_letters() {
        #[allow(unused_must_use)]
        Key::with_first_n_letters(4).random_subsets(&vec![2, 2, 1]);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn random_subsets_panic_if_any_group_has_size_zero() {
        #[allow(unused_must_use)]
        Key::with_first_n_letters(4).random_subsets(&vec![1, 1, 0, 2]);
    }

    #[test]
    #[ignore]
    fn random_subsets_print_out() {
        let key = Key::with_every_letter();
        for n in key.random_subsets(&vec![3, 3, 5, 4]) {
            println!("{}", n)
        }
    }

    // fn format_distribute_results()
    #[test]
    #[ignore]
    fn distribute_key_and_print() {
        let key = Key::with_first_n_letters(6);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![2, 3, 1]); // ugly syntax
        let results = key.distribute(key_sizes);
        for r in results {
            let k = Keyboard::new_from_keys(r);
            println!("{}", k)
        }
    }

    fn format_keys(keys: &Vec<Vec<Key>>) -> String {
        let mut formatted = keys
            .iter()
            .map(|k| Keyboard::new_from_keys(k.clone()).to_string())
            .collect::<Vec<String>>();
        formatted.sort();
        formatted.join(" : ")
    }

    #[test]
    fn distribute_letters_with_equal_key_sizes() {
        let key = Key::with_first_n_letters(4);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![2, 2]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("ab cd : ac bd : bc ad", results_as_text);
    }

    #[test]
    fn distribute_letters_with_unequal_key_sizes() {
        let key = Key::with_first_n_letters(3);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![1, 2]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("a bc : ab c : b ac", results_as_text);
    }

    #[test]
    fn distribute_many_letters_with_one_key() {
        let key = Key::with_first_n_letters(3);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![3]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("abc", results_as_text);
    }

    #[test]
    fn distribute_one_letter_on_one_key() {
        let key = Key::with_first_n_letters(1);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![1]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("a", results_as_text);
    }

    #[test]
    fn distribute_letters_calculates_correct_number_of_results() {
        let key = Key::with_first_n_letters(14);
        let key_sizes = ItemCount::<u32>::with_groups(&vec![2, 2, 3, 3, 4]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        assert_eq!(6306300, results.iter().count());
    }
}
