use std::{fmt, iter};

use rand::Rng;

use crate::{
    dictionary::Dictionary, key::Key, letter::Letter, partitions::Partitions, penalty::Penalty,
    solution::Solution, tally::Tally, word::Word,
};

// fix this!
#[derive(Clone)]
pub struct Keyboard {
    keys: Vec<Key>,
    letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE],
}

impl Keyboard {
    pub fn new_from_keys(keys: Vec<Key>) -> Keyboard {
        let mut letter_to_key_index: [Option<usize>; Letter::ALPHABET_SIZE] = Default::default();
        for (key_index, key) in keys.iter().enumerate() {
            for letter in *key {
                debug_assert!(
                    letter_to_key_index[letter.to_usize()].is_none(),
                    "Some keys on the keyboard have duplicate letters."
                );
                letter_to_key_index[letter.to_usize()] = Some(key_index);
            }
        }
        Keyboard {
            keys,
            letter_to_key_index,
        }
    }

    // abc,def,ghh
    pub fn new_from_layout(s: &str) -> Keyboard {
        let keys = s
            .split(",")
            .map(|letters| {
                Key::try_from(letters).expect("Expected each key to be separated by a comma.")
            })
            .collect::<Vec<Key>>();
        Keyboard::new_from_keys(keys)
    }

    pub fn with_penalty(self, penalty: Penalty) -> Solution {
        Solution::new(self, penalty, "".to_string())
    }

    pub fn with_penalty_and_notes(self, penalty: Penalty, notes: String) -> Solution {
        Solution::new(self, penalty, notes)
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub fn max_key_size(&self) -> Option<u32> {
        self.keys.iter().map(|k| k.count_letters()).max()
    }

    fn find_key_index_for_letter(&self, letter: Letter) -> Option<usize> {
        self.letter_to_key_index[letter.to_usize()]
    }

    fn find_key_for_letter(&self, letter: Letter) -> Option<Key> {
        let key_index = self.find_key_index_for_letter(letter)?;
        let key = self.keys.get(key_index)?;
        Some(*key)
    }

    pub fn spell_serialized(&self, word: &Word) -> String {
        let mut result = String::new();
        for letter in word.letters() {
            match self.find_key_index_for_letter(*letter) {
                None => panic!(
                    "Could not spell the word {} because the keyboard is missing the letter {}",
                    word, letter
                ),
                Some(index) => {
                    const BASE_CHAR: u32 = 'A' as u32;
                    let char = char::from_u32((index as u32 + BASE_CHAR) as u32).unwrap();
                    result.push(char);
                }
            }
        }
        result
    }

    pub fn spell(&self, word: &Word) -> String {
        let result = word
            .letters()
            .iter()
            .map(|letter| self.find_key_for_letter(*letter))
            .collect::<Option<Vec<Key>>>()
            .map(|keys| keys.iter().map(|k| k.to_string()).collect::<Vec<String>>())
            .map(|kk| kk.join(","));
        match result {
            None => panic!(
                "Could not spell the word {} because the keyboard is missing a necessary key.",
                word
            ),
            Some(spelling) => spelling,
        }
    }

    pub fn penalty_by_key_size(dictionary: &Dictionary, size: u32) -> Vec<(Key, Penalty)> {
        let alphabet = dictionary.alphabet();
        let keys_to_evaluate = alphabet.subsets_of_size(size);
        let mut result: Vec<(Key, Penalty)> = vec![];
        for evaluate in keys_to_evaluate {
            let rest = alphabet.except(evaluate);
            let mut keys = rest
                .into_iter()
                .map(Key::with_one_letter)
                .collect::<Vec<Key>>();
            keys.push(evaluate);
            let keyboard = Keyboard::new_from_keys(keys);
            let penalty = keyboard.penalty(&dictionary, Penalty::MAX);
            result.push((evaluate, penalty));
        }
        result
    }

    pub fn random(alphabet: Key, layout: &Partitions) -> impl Iterator<Item = Keyboard> {
        let mut rng = rand::thread_rng();
        let layout_options = layout.calculate();
        iter::repeat_with(move || {
            let layout_index = rng.gen_range(0..layout_options.len());
            let layout = layout_options.get(layout_index).unwrap();
            let keys = alphabet.random_subsets(layout).collect::<Vec<Key>>();
            let keyboard = Keyboard::new_from_keys(keys);
            keyboard
        })
    }

    pub fn swap_random_letters_n_times(k: Keyboard, count: u32) -> Result<Keyboard, &'static str> {
        if count == 0 {
            Ok(k)
        } else {
            let k = k.swap_random_letters()?;
            Keyboard::swap_random_letters_n_times(k, count - 1)
        }
    }

    pub fn swap_random_letters(&self) -> Result<Keyboard, &'static str> {
        let total_keys = self.keys.len();
        if total_keys == 1 {
            Err("It is not possible to swap letters on a keyboard with only 1 key.")
        } else if total_keys == 0 {
            Err("It is not possible to swap letters on a keyboard with 0 keys.")
        } else {
            let mut rng = rand::thread_rng();
            let from_index = rng.gen_range(0..total_keys);
            let to_index = iter::repeat_with(move || rng.gen_range(0..total_keys))
                .find(|n| *n != from_index)
                .unwrap();
            let a_key = self.keys[from_index];
            let b_key = self.keys[to_index];
            let a_letter_to_swap = a_key.random_letter().unwrap();
            let b_letter_to_swap = b_key.random_letter().unwrap();
            let new_a_key = a_key.remove(a_letter_to_swap).add(b_letter_to_swap);
            let new_b_key = b_key.remove(b_letter_to_swap).add(a_letter_to_swap);
            let new_keys = self
                .keys
                .iter()
                .map(|k| {
                    if *k == a_key {
                        new_a_key
                    } else if *k == b_key {
                        new_b_key
                    } else {
                        *k
                    }
                })
                .collect();
            Ok(Keyboard::new_from_keys(new_keys))
        }
    }

    pub fn every_swap(&self) -> Vec<Keyboard> {
        if self.keys.len() < 2 {
            panic!("Can not swap keys on a keyboard with less than 2 keys on it.")
        }
        let mut result = vec![];
        for a_key_index in 0..=self.keys.len() - 2 {
            for b_key_index in a_key_index + 1..=(self.keys.len() - 1) {
                let a_key = self.keys[a_key_index];
                let b_key = self.keys[b_key_index];
                for a_letter in a_key {
                    for b_letter in b_key {
                        if a_letter < b_letter {
                            let a_key_after = a_key.remove(a_letter).add(b_letter);
                            let b_key_after = b_key.remove(b_letter).add(a_letter);
                            let letters = self
                                .keys
                                .iter()
                                .map(|k| {
                                    if *k == a_key {
                                        a_key_after
                                    } else if *k == b_key {
                                        b_key_after
                                    } else {
                                        *k
                                    }
                                })
                                .collect();
                            let keyboard = Keyboard::new_from_keys(letters);
                            result.push(keyboard);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn every_combine_two_keys_filter(&self, prohibited_pairs: &Vec<Key>) -> Vec<Keyboard> {
        if self.keys.len() <= 1 {
            panic!("It is not possible to combine keys on the keyboard since it only has {} keys right now.", self.keys.len());
        }
        let mut results = vec![];
        for a_index in 0..=self.keys.len() - 2 {
            for b_index in a_index + 1..=self.keys.len() - 1 {
                let combined_key = self.keys[a_index].union(self.keys[b_index]);
                if prohibited_pairs
                    .iter()
                    .all(move |k| k.intersect(combined_key).count_letters() <= 1)
                {
                    let new_keys: Vec<Key> = self
                        .keys
                        .iter()
                        .enumerate()
                        .filter_map(|(index, k)| {
                            if index == a_index {
                                Some(combined_key)
                            } else if index == b_index {
                                None
                            } else {
                                Some(*k)
                            }
                        })
                        .collect();
                    let new_keyboard = Keyboard::new_from_keys(new_keys);
                    results.push(new_keyboard);
                }
            }
        }
        results
    }

    pub fn every_combine_two_keys(&self) -> Vec<Keyboard> {
        self.every_combine_two_keys_filter(&vec![])
    }

    pub fn penalty_by_word<'a>(
        &'a self,
        dictionary: &'a Dictionary,
    ) -> impl Iterator<Item = (&Word, Penalty)> {
        let mut found = Tally::new();
        dictionary.words().iter().map(move |word| {
            let how_to_spell = self.spell_serialized(word);
            let word_penalty = match found.count(&how_to_spell) {
                0 => {
                    found.increment(how_to_spell);
                    Penalty::ZERO
                }
                seen => {
                    found.increment(how_to_spell);
                    Penalty::new(word.frequency().to_f32() * seen.min(4) as f32)
                }
            };
            (word, word_penalty)
        })
    }

    pub fn penalty(&self, dictionary: &Dictionary, to_beat: Penalty) -> Penalty {
        let mut penalty = Penalty::ZERO;
        for (_, word_penalty) in self.penalty_by_word(dictionary) {
            penalty = penalty + word_penalty;
            if penalty >= to_beat {
                break;
            }
        }
        penalty
    }

    pub const PAIR_PENALTIES: [(&'static str, f64); 351] = [
        ("ai", 0.034938633),
        ("st", 0.024594279),
        ("ns", 0.02124434),
        ("nt", 0.019316979),
        ("io", 0.01580517),
        ("fn", 0.013309657),
        ("ds", 0.012791129),
        ("ao", 0.010599401),
        ("ey", 0.009333591),
        ("eo", 0.009239843),
        ("tw", 0.009207384),
        ("bm", 0.009041642),
        ("hw", 0.00853797),
        ("nr", 0.0084806355),
        ("ae", 0.008404736),
        ("bh", 0.0071061947),
        ("dt", 0.0067602484),
        ("ms", 0.0063932003),
        ("dr", 0.0058965),
        ("rs", 0.0058608945),
        ("mw", 0.0055901012),
        ("fr", 0.005558195),
        ("bw", 0.0053312825),
        ("dn", 0.0052265828),
        ("fs", 0.0051839105),
        ("mt", 0.005010666),
        ("ei", 0.004851741),
        ("hm", 0.004703275),
        ("ft", 0.0046909666),
        ("rt", 0.0046867616),
        ("mn", 0.0043854862),
        ("lr", 0.0043663485),
        ("au", 0.0042790277),
        ("dm", 0.0041806),
        ("ps", 0.004098979),
        ("cs", 0.004091303),
        ("ln", 0.0039783414),
        ("ls", 0.0039698603),
        ("lm", 0.0038753543),
        ("dl", 0.00383779),
        ("gs", 0.0038198945),
        ("gn", 0.003743759),
        ("lt", 0.0037366103),
        ("dg", 0.0037240214),
        ("iu", 0.0036709243),
        ("mp", 0.0036509389),
        ("ny", 0.0035114663),
        ("sy", 0.0034593104),
        ("gt", 0.00343734),
        ("mr", 0.0032789693),
        ("et", 0.0032714608),
        ("ct", 0.003270481),
        ("cw", 0.0031359922),
        ("bo", 0.003079699),
        ("hn", 0.0030791138),
        ("er", 0.0030649663),
        ("sw", 0.0030611695),
        ("no", 0.0030562456),
        ("cm", 0.002966851),
        ("ou", 0.002939674),
        ("lp", 0.0028898208),
        ("as", 0.0028364013),
        ("hs", 0.0028225405),
        ("pt", 0.0027923826),
        ("hr", 0.0027912583),
        ("bt", 0.002694945),
        ("dp", 0.0026912433),
        ("dk", 0.0026174048),
        ("bs", 0.0026076843),
        ("gl", 0.002557508),
        ("my", 0.0025448825),
        ("cp", 0.0025337401),
        ("dw", 0.0025312102),
        ("bl", 0.0025163915),
        ("hl", 0.0024908527),
        ("fl", 0.0024903028),
        ("es", 0.0024451704),
        ("ch", 0.0024315133),
        ("cr", 0.0024222417),
        ("bp", 0.002419809),
        ("fm", 0.0023908734),
        ("br", 0.0023595325),
        ("dy", 0.0023465664),
        ("cn", 0.0023222894),
        ("eh", 0.0022879122),
        ("de", 0.0022748448),
        ("cg", 0.0022622326),
        ("bc", 0.0022495878),
        ("ht", 0.0022463475),
        ("pr", 0.002168716),
        ("is", 0.00210572),
        ("ar", 0.0020618192),
        ("lw", 0.0020535958),
        ("el", 0.0020454777),
        ("hp", 0.0020341265),
        ("nw", 0.002032511),
        ("eu", 0.001984791),
        ("np", 0.0019788411),
        ("cd", 0.001978185),
        ("bf", 0.0019675822),
        ("fw", 0.001941773),
        ("bd", 0.0019111093),
        ("cf", 0.0018890548),
        ("fk", 0.0018768562),
        ("fh", 0.00186572),
        ("ry", 0.0018381415),
        ("cl", 0.0018211696),
        ("ah", 0.0018034346),
        ("dh", 0.0017902877),
        ("op", 0.001767973),
        ("or", 0.0016779626),
        ("bg", 0.001669267),
        ("rv", 0.0016669049),
        ("pw", 0.001661955),
        ("at", 0.001654305),
        ("nv", 0.0016516458),
        ("rw", 0.0016149661),
        ("ac", 0.0016147534),
        ("kn", 0.0016087964),
        ("em", 0.0015865165),
        ("fp", 0.0015806851),
        ("ru", 0.0015626126),
        ("gr", 0.0015531301),
        ("ow", 0.0015489389),
        ("al", 0.0015466985),
        ("kt", 0.0015398094),
        ("km", 0.001536243),
        ("oy", 0.0015360372),
        ("gm", 0.0015058222),
        ("ks", 0.00147461),
        ("fg", 0.0014668794),
        ("ty", 0.0014622778),
        ("am", 0.0014597507),
        ("ad", 0.001449734),
        ("ap", 0.0014423573),
        ("ep", 0.0014405127),
        ("bn", 0.0014038488),
        ("mo", 0.0013921777),
        ("gk", 0.0013875354),
        ("gh", 0.0013801802),
        ("ir", 0.0013684466),
        ("in", 0.0013683765),
        ("en", 0.0013652439),
        ("it", 0.0013621608),
        ("ce", 0.0013553995),
        ("kl", 0.0013312096),
        ("df", 0.0013265227),
        ("an", 0.0013032985),
        ("sv", 0.0012885676),
        ("mv", 0.0012550894),
        ("gp", 0.0012540071),
        ("hk", 0.0012280903),
        ("kr", 0.0012278788),
        ("kp", 0.0012208547),
        ("jm", 0.0012124081),
        ("di", 0.0012109122),
        ("wy", 0.0012075548),
        ("il", 0.001199773),
        ("im", 0.0011846136),
        ("os", 0.0011485728),
        ("co", 0.001140208),
        ("gw", 0.0011373984),
        ("ek", 0.0011253913),
        ("ot", 0.0011223631),
        ("eg", 0.0010751832),
        ("ci", 0.0010679739),
        ("fv", 0.0010587791),
        ("be", 0.0010500258),
        ("lo", 0.0010263727),
        ("lv", 0.0010219335),
        ("hy", 0.0010186664),
        ("ab", 0.0010124278),
        ("kv", 0.0010107702),
        ("do", 0.001003653),
        ("tv", 0.0010024523),
        ("gy", 0.0009977762),
        ("dv", 0.0009857182),
        ("ef", 0.0009571722),
        ("ly", 0.0009538479),
        ("ip", 0.0009487942),
        ("ag", 0.00088893145),
        ("fy", 0.00088823383),
        ("bk", 0.0008568096),
        ("hi", 0.0008392771),
        ("ck", 0.0008363756),
        ("cv", 0.0008221284),
        ("bj", 0.0008125552),
        ("pv", 0.00079067156),
        ("ew", 0.00078971486),
        ("by", 0.0007868809),
        ("ay", 0.0007818777),
        ("kw", 0.000779123),
        ("tx", 0.0007752647),
        ("bi", 0.00077389216),
        ("ho", 0.00077185605),
        ("cy", 0.00075785944),
        ("py", 0.00073933613),
        ("jp", 0.0007386043),
        ("su", 0.000737399),
        ("af", 0.00073524466),
        ("iy", 0.00073465344),
        ("jt", 0.00072211435),
        ("fi", 0.0007192856),
        ("ak", 0.00069200963),
        ("dj", 0.0006890764),
        ("bv", 0.00067223655),
        ("js", 0.00066936243),
        ("jl", 0.00066728925),
        ("mu", 0.0006654469),
        ("rx", 0.0006604183),
        ("tu", 0.0006578958),
        ("nu", 0.0006490484),
        ("jr", 0.0006472417),
        ("nx", 0.0006421336),
        ("pu", 0.0006247606),
        ("sx", 0.0006164256),
        ("gi", 0.000614233),
        ("go", 0.00061118725),
        ("cu", 0.0006060776),
        ("jn", 0.0005924803),
        ("vw", 0.0005898182),
        ("gj", 0.0005885452),
        ("gv", 0.0005879404),
        ("iv", 0.00058629864),
        ("aw", 0.0005813429),
        ("du", 0.00057927653),
        ("sz", 0.00057238917),
        ("dx", 0.00057096424),
        ("iw", 0.000569633),
        ("lu", 0.00056877837),
        ("fo", 0.00054675917),
        ("av", 0.0005369823),
        ("dz", 0.00053662685),
        ("cj", 0.00053373893),
        ("fj", 0.0005301501),
        ("ex", 0.00052501087),
        ("hv", 0.0005185061),
        ("hu", 0.00051793165),
        ("hj", 0.0005165349),
        ("tz", 0.0005107184),
        ("ov", 0.00050912297),
        ("ax", 0.00049861876),
        ("lx", 0.00049096416),
        ("bu", 0.00047160278),
        ("ev", 0.0004590373),
        ("jw", 0.00044341251),
        ("px", 0.00044165403),
        ("ik", 0.00043755237),
        ("ky", 0.00043260667),
        ("gu", 0.00043256144),
        ("mx", 0.0004315104),
        ("cx", 0.0004196525),
        ("xy", 0.00041042146),
        ("ko", 0.00040843044),
        ("vy", 0.00039995284),
        ("uv", 0.00039819756),
        ("jk", 0.00039805568),
        ("fu", 0.0003967361),
        ("aj", 0.00038005403),
        ("bx", 0.00037362927),
        ("uw", 0.00037350258),
        ("ix", 0.0003691435),
        ("uy", 0.0003616645),
        ("jy", 0.0003569446),
        ("wx", 0.0003438482),
        ("ij", 0.0003339339),
        ("jv", 0.0003285603),
        ("gx", 0.0003222686),
        ("ox", 0.0003183555),
        ("jo", 0.0003182494),
        ("ej", 0.00031449166),
        ("ku", 0.00030996834),
        ("ux", 0.0002860748),
        ("vx", 0.00028163896),
        ("nz", 0.00028118535),
        ("mz", 0.00027888294),
        ("lz", 0.00027064144),
        ("hx", 0.0002640678),
        ("fx", 0.00025855863),
        ("kx", 0.00025651685),
        ("rz", 0.00025096312),
        ("bz", 0.0002481085),
        ("cz", 0.00024738142),
        ("ju", 0.00024329402),
        ("gz", 0.00024239234),
        ("hz", 0.00022124845),
        ("pz", 0.00021785818),
        ("jx", 0.00018967818),
        ("kz", 0.00018301516),
        ("qs", 0.00018219529),
        ("ez", 0.00015883187),
        ("fz", 0.00015179566),
        ("az", 0.00015173986),
        ("vz", 0.00014783313),
        ("yz", 0.00014133686),
        ("wz", 0.00014108724),
        ("gq", 0.00014098953),
        ("jz", 0.00013722252),
        ("dq", 0.00013670708),
        ("iz", 0.00013358802),
        ("nq", 0.00013269007),
        ("oz", 0.00012744244),
        ("e'", 0.00012733998),
        ("cq", 0.0001239321),
        ("qr", 0.00012080652),
        ("aq", 0.00011997989),
        ("qt", 0.00011971474),
        ("mq", 0.00011824584),
        ("lq", 0.00011288348),
        ("pq", 0.00011277724),
        ("bq", 0.00011188132),
        ("iq", 0.000108433844),
        ("eq", 0.000108083186),
        ("uz", 0.000105574625),
        ("oq", 0.00010541758),
        ("a'", 0.00010528333),
        ("fq", 0.00010353169),
        ("hq", 0.00010157328),
        ("kq", 0.00009930828),
        ("n'", 0.00009882715),
        ("qw", 0.000092947834),
        ("qu", 0.00009094803),
        ("qv", 0.00009072169),
        ("xz", 0.00008921111),
        ("qy", 0.00008165882),
        ("jq", 0.00008104033),
        ("qx", 0.00007750158),
        ("r'", 0.000075293145),
        ("l'", 0.00006573848),
        ("qz", 0.000057669356),
        ("i'", 0.000057658817),
        ("t'", 0.000049946455),
        ("u'", 0.000049142574),
        ("o'", 0.00004662499),
        ("d'", 0.00004626442),
        ("b'", 0.000038726226),
        ("p'", 0.000035401517),
        ("s'", 0.000033268272),
        ("m'", 0.000031117906),
        ("g'", 0.000028738921),
        ("c'", 0.000026300975),
        ("k'", 0.000015372958),
        ("y'", 0.000014575174),
        ("h'", 0.000013167547),
        ("f'", 0.000012042015),
        ("w'", 0.0000118407115),
        ("v'", 0.00000986977),
        ("x'", 0.00000333694),
        ("j'", 0.0000031387651),
        ("q'", 0.0000019751137),
        ("z'", 0.0000006280785),
    ];
}

impl fmt::Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = self
            .keys
            .iter()
            .map(|k| Key::to_string(k))
            .collect::<Vec<String>>();
        result.sort();
        let result = result.join(" ");
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {

    use crate::util;

    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn new_panic_if_keys_with_duplicate_letters() {
        Keyboard::new_from_layout("abc,def,ghi,axy");
    }

    #[test]
    fn spell_test() {
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,stu,vwx,yz'");
        let w = Word::try_from("word").unwrap();
        let actual = k.spell(&w);
        assert_eq!(actual, "vwx,mno,pqr,def");
    }

    #[test]
    #[should_panic]
    fn spell_panic_if_required_letter_not_on_keyboard() {
        let k = Keyboard::new_from_layout("abc,def,ghi");
        let w = Word::try_from("abcx").unwrap();
        k.spell(&w);
    }

    #[test]
    fn find_key_index_for_letter() {
        let data = [
            ("abc", 'a', Some(0)),
            ("abc", 'b', Some(0)),
            ("abc", 'c', Some(0)),
            ("abc", 'x', None),
            ("abc", 'b', Some(0)),
            ("abc,def", 'd', Some(1)),
            ("abc,def", 'e', Some(1)),
            ("abc,def", 'f', Some(1)),
            ("abc,def", 'x', None),
        ];
        for (layout, letter, expected_key_index) in data {
            let keyboard = Keyboard::new_from_layout(layout);
            let letter_to_find = Letter::try_from(letter).unwrap();
            let actual = keyboard.find_key_index_for_letter(letter_to_find);
            assert_eq!(actual, expected_key_index);
        }
    }

    #[test]
    #[ignore]
    fn spell_print_each_dictionary_word_out() {
        let d = Dictionary::load();
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mnop,qrs,tuv,wxyz'");
        d.words().iter().take(20).for_each(|w| {
            let spelling = k.spell(&w);
            println!("{} : {}", w, spelling);
        })
    }

    #[test]
    fn penalty_score_is_correct() {
        let d = Dictionary::load();
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,st,uv,wx,yz'");
        let actual: f32 = k.penalty(&d, Penalty::MAX).to_f32(); // why into does not work
        assert!(actual >= 0.0802 && actual <= 0.0804); // 0.0803
    }

    #[test]
    #[ignore]
    fn swap_random_letters() {
        let mut k = Keyboard::new_from_layout("abc,def,ghi");
        for _i in 1..10 {
            k = k.swap_random_letters().unwrap();
            println!("{}", k)
        }
    }

    #[test]
    #[ignore]
    fn every_swap() {
        let k = Keyboard::new_from_layout("abc,def,ghi,jkl,mno,pqr,stu,vw,xy,z'");
        k.every_swap().iter().for_each(|k| println!("{}", k));
        println!("Total swaps: {}", k.every_swap().iter().count());
    }

    #[test]
    #[ignore]
    fn every_combine_two_keys() {
        let k = Keyboard::new_from_layout("a,b,c,d,efg,hi");
        k.every_combine_two_keys()
            .iter()
            .for_each(|k| println!("{}", k));
    }

    #[test]
    fn every_combine_two_keys_generates_correct_number_of_answers() {
        let data = [
            "a,b",
            "a,b,c,d",
            "a,b,c,d,e,f,g,h,i,j,k,l,m",
            "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,'",
        ];
        for d in data {
            let k = Keyboard::new_from_layout(d);
            let actual_count = k.every_combine_two_keys().len();
            let expected = util::choose(k.keys.len() as u32, 2);
            assert_eq!(actual_count, expected as usize);
        }
    }

    #[test]
    #[ignore]
    fn create_file_of_penalty_per_word() {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("output.txt").unwrap();
        writeln!(file, "index, word, penalty").unwrap();
        let d = Dictionary::load();
        let keyboard = Keyboard::new_from_layout("ot,gr,dh,su,im,bn,awz,cky',fjlx,epqv");
        for (word_index, (word, penalty)) in keyboard.penalty_by_word(&d).enumerate() {
            writeln!(file, "{},{},{}", word_index + 1, word, penalty.to_f32()).unwrap();
        }
    }

    #[test]
    #[ignore]
    fn random_keyboard_print_out() {
        let partition = Partitions {
            sum: 27,
            parts: 10,
            min: 2,
            max: 5,
        };
        let dict = Dictionary::load();
        let keyboards = Keyboard::random(dict.alphabet(), &partition);
        for k in keyboards.take(50) {
            println!("{}", k)
        }
    }

    #[test]
    #[ignore]
    fn output_penalty_by_key_size() {
        use std::fs::File;
        use std::io::prelude::*;
        let key_size = 2;
        let mut file = File::create("output.txt").unwrap();
        let dict = Dictionary::load();
        for (key, penalty) in Keyboard::penalty_by_key_size(&dict, key_size) {
            writeln!(file, "{},{}", key, penalty.to_f32()).unwrap();
        }
    }
}
