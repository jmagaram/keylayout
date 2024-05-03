use std::{
    fmt::{self, Debug},
    ops::RangeInclusive,
};

use rand::Rng;

use crate::{lazy_tree::Seed, letter::Letter, prohibited::Prohibited, tally::Tally, util};

#[derive(PartialEq, PartialOrd, Eq, Debug, Clone, Copy, Default, Hash)]
pub struct Key(u32);

impl Key {
    pub const EMPTY: Key = Key(0);
    const MAX_SIZE: u8 = Letter::ALPHABET_SIZE as u8;

    pub fn new(letters: &str) -> Key {
        Key::try_from(letters).unwrap()
    }

    pub fn with_every_letter() -> Key {
        Key((1 << Letter::ALPHABET_SIZE) - 1)
    }

    pub fn with_first_n_letters(count: u8) -> Key {
        if (count as usize) > Letter::ALPHABET_SIZE {
            panic!(
                "A Key can have at most {} letters, but you asked for {}.",
                Letter::ALPHABET_SIZE,
                count,
            );
        }
        Key((1 << count) - 1)
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }

    pub fn with_one_letter(r: Letter) -> Key {
        Key(1 << r.to_u8_index())
    }

    pub fn split_by_letter(&self) -> impl Iterator<Item = Key> {
        self.letters().map(|letter| Key::with_one_letter(letter))
    }

    pub fn swap_random_letter(&self, other: &Key) -> Option<(Key, Key)> {
        let self_letter_to_remove = self.random_letter()?;
        let other_letter_to_remove = other.random_letter()?;
        let new_self = self
            .remove(self_letter_to_remove)
            .add(other_letter_to_remove);
        let new_other = other
            .remove(other_letter_to_remove)
            .add(self_letter_to_remove);
        Some((new_self, new_other))
    }

    pub fn add(&self, r: Letter) -> Key {
        Key(self.0 | 1 << r.to_u8_index())
    }

    pub fn remove(&self, r: Letter) -> Key {
        Key(self.0 & !(1 << r.to_u8_index()))
    }

    pub fn contains(&self, r: Letter) -> bool {
        self.0 & (1 << r.to_u8_index()) != 0
    }

    pub fn contains_all(&self, other: &Key) -> bool {
        self.intersect(*other) == *other
    }

    pub fn count_letters(&self) -> u8 {
        util::set_bits(self.0).count() as u8
    }

    pub fn len(&self) -> u8 {
        self.count_letters()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_prohibited(&self, prohibited: &Prohibited) -> bool {
        !prohibited.is_allowed(self.clone())
    }

    pub fn is_allowed(&self, prohibited: &Prohibited) -> bool {
        prohibited.is_allowed(self.clone())
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

    pub fn letters(&self) -> impl Iterator<Item = Letter> {
        util::set_bits(self.0).map(|bit| Letter::try_from(bit as u32).unwrap())
    }

    pub fn random_letter(&self) -> Option<Letter> {
        let total_letters = self.len();
        if total_letters == 0 {
            None
        } else {
            let index = rand::random::<usize>().rem_euclid(total_letters as usize);
            self.letters().nth(index)
        }
    }

    pub fn random_subset(&self, size: RangeInclusive<u8>) -> Key {
        debug_assert!(
            *size.start() <= self.len(),
            "Can not get a minimum of {} random letters from a Key with only {} letters in it.",
            size.start(),
            self.len()
        );
        debug_assert!(
            *size.end() <= self.len(),
            "Can not get a maximum of {} random letters from a Key with only {} letters in it.",
            size.end(),
            self.len()
        );
        debug_assert!(
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

    /// Splits a key into a random number of other keys according the specified
    /// key sizes, such that the letters on all those keys match the letters on
    /// the source key. For example, the key "abcdef" with group sizes [3,1,2]
    /// could get split into cab, d, and ef.
    pub fn random_subsets(&self, groupings: &Vec<u8>) -> impl Iterator<Item = Key> {
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
            remaining_letters: *self,
            group_index: 0,
        }
        .into_iter()
    }

    /// Generates all possible keys of size `size` that include the letters of
    /// the source key. For example,the key "abcd" with size 2 generates "ab",
    /// "ac", "ad", "bc", "bd", and "cd". This is essentially choosing unique
    /// combinations of letters.
    pub fn subsets_of_size(&self, size: u8) -> impl Iterator<Item = Key> {
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
            let letters = self.letters().collect::<Vec<Letter>>();
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
                    Key(i as u32).letters().fold(Key::EMPTY, |total, i| {
                        total.add(letters[i.to_usize_index()])
                    })
                });
            let result_boxed: Box<dyn Iterator<Item = Key>> = Box::new(result);
            result_boxed
        }
    }

    /// Generates all unique ways the letters on the source key can be
    /// distributed across other keys. For example, if the source key contains
    /// a-z, and the `key_sizes` indicates 8 keys of size 3, and 3 keys of size
    /// 2, every way of distributing those letters is generated.
    pub fn distribute(&self, key_sizes: Tally<u8>) -> impl Iterator<Item = Vec<Key>> + '_ {
        let results = key_sizes
            .combinations()
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

mod subset_implementation {

    use crate::{lazy_tree::Seed, letter::Letter};

    use super::Key;

    pub struct SubsetSeed {
        pub available: Key,
        pub needed: usize,
        pub include_empty_set: bool, // i think this is actually ONLY empty set returned!
    }

    impl<'a> Seed<'a, Option<Letter>, Key> for SubsetSeed {
        fn is_empty(&self) -> bool {
            self.needed == 0 && !self.include_empty_set
        }

        fn children(&self) -> impl Iterator<Item = (Option<Letter>, Self)> + 'a {
            let max_letter = self.available.max_letter();
            match (max_letter, self.include_empty_set) {
                (None, false) => {
                    panic!("Could not get a letter from the Key but was expecting one.")
                }
                (_, true) => {
                    let result = (
                        None,
                        SubsetSeed {
                            available: Key::EMPTY,
                            needed: 0,
                            include_empty_set: false,
                        },
                    );
                    vec![result].into_iter()
                }
                (Some(max_letter), _) => {
                    let available = self.available.remove(max_letter);
                    let with_max_letter = (
                        Some(max_letter),
                        SubsetSeed {
                            available,
                            needed: self.needed - 1,
                            include_empty_set: self.include_empty_set,
                        },
                    );
                    let result = match self.needed == self.available.count_letters() as usize {
                        true => vec![with_max_letter].into_iter(),
                        false => vec![
                            with_max_letter,
                            (
                                None,
                                SubsetSeed {
                                    available,
                                    needed: self.needed,
                                    include_empty_set: self.include_empty_set,
                                },
                            ),
                        ]
                        .into_iter(),
                    };
                    result
                }
            }
        }

        fn add(result: Key, item: Option<Letter>) -> Key {
            match item {
                Some(letter) => result.add(letter),
                None => result,
            }
        }
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
        for c in self.letters() {
            result.push(c.to_char());
        }
        write!(f, "{}", result)
    }
}

struct RandomSubsets {
    groups: Vec<u8>,
    group_index: usize,
    remaining_letters: Key,
}

impl Iterator for RandomSubsets {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        if self.group_index >= self.groups.len() {
            None
        } else {
            let group_size = self.groups[self.group_index];
            let key = self
                .remaining_letters
                .random_subset(group_size..=group_size);
            let remaining_letters = self.remaining_letters.except(key);
            self.remaining_letters = remaining_letters;
            self.group_index = self.group_index + 1;
            Some(key)
        }
    }
}

struct DistributeLetters {
    key_sizes: Vec<u8>,
    letters: Key,
}

impl<'a> Seed<'a, Key, Vec<Key>> for DistributeLetters {
    fn is_empty(&self) -> bool {
        self.key_sizes.is_empty()
    }

    fn add(result: Vec<Key>, item: Key) -> Vec<Key> {
        let mut result = result.clone();
        result.push(item);
        result
    }

    fn children(&self) -> impl Iterator<Item = (Key, Self)> + 'a {
        assert!(
            self.key_sizes.len() > 0,
            "Expected the list of key sizes to not be empty."
        );
        assert!(
            self.letters.len() > 0,
            "Expected the number of remaining letters to be 1 or more."
        );
        debug_assert!(
            self.key_sizes.iter().all(|i| *i > 0),
            "Every key size should be 1 or more."
        );
        debug_assert!(
            self.key_sizes.iter().fold(0, |total, i| total + i) <= self.letters.len(),
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
        results // was boxed before
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::{keyboard::Keyboard, letter::Letter, util};

    use super::*;

    #[test]
    fn try_from_str_when_valid() {
        for s in ["abc", "a", "abcdez\'", "xyz", ""] {
            let key: Key = s.try_into().unwrap();
            assert_eq!(key.to_string(), s.to_string());
        }
    }

    #[test]
    fn try_from_str_when_invalid() {
        for s in ["098", "a#4", "   "] {
            let key: Result<Key, _> = s.try_into();
            assert!(key.is_err());
        }
    }

    #[test]
    fn to_string_concatenates_letters() {
        let data = ["abc", "", "mnop'"];
        for s in data {
            let actual = Key::new(s).to_string();
            assert_eq!(s.to_string(), actual);
        }
    }

    #[test]
    fn with_every_letter_includes_every_letter() {
        let target = Key::with_every_letter();
        assert_eq!(
            target.to_string(),
            "abcdefghijklmnopqrstuvwxyz'".to_string()
        );
    }

    #[test]
    fn with_first_n_letters_includes_first_n() {
        let data = [
            (0, ""),
            (1, "a"),
            (3, "abc"),
            (27, "abcdefghijklmnopqrstuvwxyz'"),
        ];
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
    fn letters_test() {
        let data = ["", "abc", "a", "xyz", "az'"];
        for d in data {
            let k: Vec<String> = Key::new(d).letters().map(|r| r.to_string()).collect();
            let result = k.join("");
            assert_eq!(d, result);
        }
    }

    #[test]
    fn add_test() {
        assert_eq!(
            Key::EMPTY
                .add(Letter::new('a'))
                .add(Letter::new('b'))
                .add(Letter::new('c'))
                .add(Letter::new('d'))
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
            let start = Key::new(start);
            let except = Key::new(other);
            let expected = Key::new(expected);
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
            let start = Key::new(start);
            let except = Letter::new(to_remove);
            let expected = Key::new(expected);
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
            let start = Key::new(start);
            let other = Letter::new(find);
            assert_eq!(start.contains(other), expected);
        }
    }

    #[test]
    fn contains_all_test() {
        let data = [
            ("", "", true),
            ("a", "a", true),
            ("a", "ab", false),
            ("a", "x", false),
            ("abc", "a", true),
            ("abc", "ab", true),
            ("abc", "ac", true),
            ("abc", "abc", true),
            ("abc", "abcx", false),
            ("abc", "ax", false),
            ("abc", "x", false),
            ("abc", "", true),
        ];
        for (start, other, expected) in data {
            let start = Key::new(start);
            let other = Key::new(other);
            assert_eq!(start.contains_all(&other), expected);
        }
    }

    #[test]
    fn count_letters_test() {
        let data = [(""), ("a"), ("abcde"), ("abcdefghijklmnopqrstuvwxyz'")];
        for start in data {
            let start = Key::new(start);
            assert_eq!(start.count_letters() as usize, start.to_string().len());
        }
    }

    #[test]
    fn with_one_letter_test() {
        assert_eq!("a", Key::with_one_letter(Letter::new('a')).to_string());
        assert_eq!("b", Key::with_one_letter(Letter::new('b')).to_string());
        assert_eq!("c", Key::with_one_letter(Letter::new('c')).to_string());
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
            let start = Key::new(start);
            let other = Key::new(other);
            let expected = Key::new(expected);
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
            let start = Key::new(start);
            let other = Key::new(other);
            let expected = Key::new(expected);
            assert_eq!(start.intersect(other), expected);
        }
    }

    #[test]
    fn max_letter_test() {
        let data = [("a", 'a'), ("abc", 'c'), ("cba", 'c')];
        for (start, expected) in data {
            let start = Key::new(start);
            let expected = Letter::new(expected);
            assert_eq!(start.max_letter(), Some(expected));
        }
        assert!(Key::EMPTY.max_letter().is_none());
    }

    #[test]
    fn min_letter_test() {
        let data = [("a", 'a'), ("abc", 'a'), ("cba", 'a'), ("xyfwfg", 'f')];
        for (start, expected) in data {
            let start = Key::new(start);
            let expected = Letter::new(expected);
            assert_eq!(start.min_letter(), Some(expected));
        }
        assert!(Key::EMPTY.min_letter().is_none());
    }

    #[test]
    fn is_allowed() {
        let mut p = Prohibited::new();
        p.add(Key::new("ae"));
        p.add(Key::new("st"));

        assert!(Key::new("aem").is_prohibited(&p));
        assert!(false == Key::new("aem").is_allowed(&p));

        assert!(false == Key::new("a").is_prohibited(&p));
        assert!(Key::new("a").is_allowed(&p));
    }

    #[test]
    #[should_panic]
    fn subsets_of_size_panic_if_bigger_than_alphabet_size() {
        let key = Key::with_every_letter();
        key.subsets_of_size(Key::MAX_SIZE + 1).take(1).count();
    }

    #[test]
    fn subsets_of_size_zero_return_one_empty_set() {
        let key = Key::with_every_letter();
        let result = key.subsets_of_size(0).collect::<Vec<Key>>();
        assert_eq!(1, result.len());
        assert_eq!(Key::EMPTY, result[0]);
    }

    #[test]
    fn subsets_of_size_test() {
        fn test(items: &str) {
            let key = Key::new(items);
            let ones_count = key.letters().count() as u8;

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
                .map(|c| Letter::new(*c)),
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
            let target = Key::new(d);
            let found = Key::from_iter((1..=1000).map(move |_| target.random_letter().unwrap()));
            assert_eq!(target, found)
        }
    }

    #[test]
    fn random_subset_gets_every_letter_eventually() {
        let data = ["", "a", "abc", "abcdefghijklmnopqrtsuvwxyz'"];
        for key in data {
            let source = Key::new(key);
            let letter_count = key.len().try_into().unwrap();
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
        Key::with_first_n_letters(4)
            .random_subsets(&vec![2, 2, 1])
            .count();
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn random_subsets_panic_if_any_group_has_size_zero() {
        #[allow(unused_must_use)]
        Key::with_first_n_letters(4)
            .random_subsets(&vec![1, 1, 0, 2])
            .count();
    }

    #[test]
    fn random_subsets_returns_keys_with_all_the_letters_of_source_key() {
        let key = Key::with_every_letter();
        let data = [
            vec![3, 3, 5, 4],
            vec![1, 1, 1, 1],
            vec![5, 5, 2, 1],
            vec![1],
        ];
        for key_sizes in data {
            let result = key.random_subsets(&key_sizes).collect::<Vec<Key>>();
            let total_letters = key_sizes.iter().fold(0, |total, i| total + i);
            let every_letter = result.iter().fold(Key::EMPTY, |total, i| total.union(*i));
            assert_eq!(every_letter.count_letters(), total_letters);
            let unique_letters = result
                .into_iter()
                .flat_map(|k| k.letters().map(|r| r))
                .collect::<HashSet<Letter>>();
            assert_eq!(unique_letters.len(), total_letters as usize);
        }
    }

    #[test]
    #[ignore]
    fn random_subsets_print_out() {
        let key = Key::with_every_letter();
        for n in key.random_subsets(&vec![3, 3, 5, 4]) {
            println!("{}", n)
        }
    }

    #[test]
    #[ignore]
    fn distribute_key_and_print() {
        let key = Key::with_first_n_letters(5);
        let key_sizes = Tally::from([2, 3]);
        let results = key.distribute(key_sizes);
        for r in results {
            let k = Keyboard::with_keys(r);
            println!("{}", k)
        }
    }

    fn format_keys(keys: &Vec<Vec<Key>>) -> String {
        let mut formatted = keys
            .iter()
            .map(|k| Keyboard::with_keys(k.clone()).to_string())
            .collect::<Vec<String>>();
        formatted.sort();
        formatted.join(" : ")
    }

    #[test]
    fn distribute_letters_with_equal_key_sizes() {
        let key = Key::with_first_n_letters(4);
        let key_sizes = Tally::from([2, 2]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("ab cd : ac bd : ad bc", results_as_text);
    }

    #[test]
    fn distribute_letters_with_unequal_key_sizes() {
        let key = Key::with_first_n_letters(3);
        let key_sizes = Tally::from([1, 2]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("a bc : ab c : ac b", results_as_text);
    }

    #[test]
    fn distribute_many_letters_with_one_key() {
        let key = Key::with_first_n_letters(3);
        let key_sizes = Tally::from([3]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("abc", results_as_text);
    }

    #[test]
    fn distribute_one_letter_on_one_key() {
        let key = Key::with_first_n_letters(1);
        let key_sizes = Tally::from([1]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        let results_as_text = format_keys(&results);
        assert_eq!("a", results_as_text);
    }

    #[test]
    fn distribute_letters_calculates_correct_number_of_results() {
        let key = Key::with_first_n_letters(11);
        let key_sizes = Tally::from([2, 2, 3, 4]);
        let results = key.distribute(key_sizes).collect::<Vec<Vec<Key>>>();
        assert_eq!(34650, results.iter().count());
    }

    #[test]
    fn distribute_letters_calculates_unique_results() {
        let data = [
            vec![1u8],
            vec![1, 2],
            vec![2, 2, 3, 3],
            vec![1, 2, 3],
            vec![4, 5],
            vec![5, 5],
        ];
        for d in data {
            let letter_count = d.iter().fold(0, |total, i| total + i);
            let key = Key::with_first_n_letters(letter_count);
            let key_sizes = Tally::from_iter(d);
            key.distribute(key_sizes)
                .map(|ks| Keyboard::with_keys(ks).to_string())
                .fold(HashSet::new(), |mut total, i| {
                    if total.contains(&i) {
                        panic!("There are duplicate results.");
                    }
                    total.insert(i);
                    total
                });
        }
    }

    #[test]
    fn distribute_letters_calculates_results_with_only_correct_letters() {
        let data = [
            vec![1],
            vec![1, 2],
            vec![2, 2, 3, 3],
            vec![1, 2, 3],
            vec![4, 5],
            vec![5, 5],
        ];
        for d in data {
            let letter_count = d.iter().fold(0, |total, i| total + i);
            let key = Key::with_first_n_letters(letter_count);
            let key_sizes = Tally::from_iter(d);
            let all_letters = key
                .distribute(key_sizes)
                .flat_map(|ks| ks)
                .fold(Key::EMPTY, |total, i| total.union(i));
            assert_eq!(all_letters.letters().count(), letter_count as usize);
        }
    }

    #[test]
    fn subsets() {
        let total_letters = 7;
        let choose = 3;
        let k = Key::with_first_n_letters(total_letters);
        let m = subset_implementation::SubsetSeed {
            needed: choose,
            available: k,
            include_empty_set: false,
        };
        for i in m.dfs() {
            println!("{}", i);
        }
    }
}
