use std::{collections::HashMap, fs::File, io::BufReader};

struct Dictionary {
    frequencies: HashMap<String, f32>,
}
// type dictionary = {
//   dict: Map.t<word, frequency>,
//   words: array<(word, frequency)>,
//   count: int,
//   frequencyTotal: frequency,
// }

impl Dictionary {
    pub fn load_json() -> HashMap<String, f32> {
        let file_name = "./words.json";
        let file = File::open(file_name).expect("file not found");
        let reader = BufReader::new(file);
        let word_frequencies: HashMap<String, f32> =
            serde_json::from_reader(reader).expect("read json properly");
        word_frequencies
    }
}

// let map: HashMap<String, String> = serde_json::from_str(data)?;
// let formatted_layout: Layout = format_layout(&layout);
// let mut current_config: Config = Vec::new();
// let mut freq_list: FreqList = word_frequencies
//     .iter() // or `.into_iter()` to consume the hashmap
//     .map(|(key, &value)| (key.clone(), value)) // Clone the key if necessary, value is copied
//     .collect();
// freq_list.sort_by(|a, b| {
//     b.1.partial_cmp(&a.1) // First, try to compare the values
//         .unwrap_or(std::cmp::Ordering::Equal) // In case of NaNs or partial comparison failure
//         .then_with(|| a.0.cmp(&b.0)) // If values are equal, sort by keys in ascending order
// });
// // Ignore the formatted_layout and current_configgc
