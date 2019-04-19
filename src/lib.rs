use failure::Error;
use hashbrown::hash_map::Entry;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::Path;
use std::collections::BinaryHeap;
use itertools::Itertools;

const MAX_WORD_LEN: usize = 4;

#[derive(Default)]
pub struct Trie(HashMap<char, Box<Trie>>);

impl Trie {
    fn put(&mut self, c: char) -> &mut Trie {
        self.0.entry(c).or_insert_with(Default::default)
    }

    fn get(&self, c: char) -> Option<&Trie> {
        self.0.get(&c).map(|b| &**b)
    }

    fn put_all<I: IntoIterator<Item = char>>(&mut self, c: I) -> &mut Trie {
        let mut iter = c.into_iter();
        match iter.next() {
            None => self,
            Some(nc) => self.put(nc).put_all(iter),
        }
    }
}

#[derive(Default)]
pub struct TrainingStore {
    all_pairs: HashMap<(String, String), u64>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Engine {
    two_gram: HashMap<String, HashMap<String, u64>>,
    counter: HashMap<String, u64>,
    total: u64,

    #[serde(skip)]
    trie: Trie,
}

// Create a min heap
#[derive(Eq, PartialEq)]
struct HeapInd((String, String), u64);

impl Ord for HeapInd {
    fn cmp(&self, other: &HeapInd) -> std::cmp::Ordering {
        self.1.cmp(&other.1).reverse()
    }
}

impl PartialOrd for HeapInd {
    fn partial_cmp(&self, other: &HeapInd) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TrainingStore {
    pub fn new() -> TrainingStore {
        Default::default()
    }

    pub fn add_pair(&mut self, a: String, b: String) {
        *self.all_pairs.entry((a, b)).or_insert(0) += 1;
    }

    pub fn extract(self, two_gram_size: usize) -> Engine {
        println!("Sieving...");
        let mut heap = BinaryHeap::new();
        for (p, v) in self.all_pairs.into_iter() {
            heap.push(HeapInd(p, v));

            if heap.len() > two_gram_size {
                heap.pop();
            }
        }

        println!("Sieving completed. Total count: {}", heap.len());
        println!("Inserting into engines");

        let mut eng: Engine = Default::default();

        while let Some(HeapInd((k1, k2), v)) = heap.pop() {
            eng.two_gram.entry(k2.clone()).or_insert_with(HashMap::new).insert(k1.clone(), v);
            *eng.counter.entry(k1).or_insert(0) += v;
            eng.total += v;
        }

        println!("Engine created.");

        eng
    }
}

pub struct Dict(HashMap<String, HashSet<char>>);

impl Dict {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Dict, Error> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        let mut dict = Dict(HashMap::new());

        for line in reader.lines() {
            let line = line?;
            let mut entry = None;
            for seg in line.split(' ') {
                match entry {
                    None => entry = Some(dict.0.entry(seg.to_owned()).or_insert_with(HashSet::new)),
                    Some(ref mut mapper) => {
                        mapper.insert(seg.chars().next().unwrap());
                    }
                }
            }
        }

        Ok(dict)
    }

    fn query<S: AsRef<str>>(&self, seg: S) -> Option<&HashSet<char>> {
        self.0.get(seg.as_ref())
    }

    fn build_words<S: AsRef<str>>(&self, pinyins: &[S], rev: &Trie) -> Vec<String> {
        if pinyins.len() == 1 {
            // Return all possible words
            return self.query(&pinyins[0]).map(|bucket| bucket.iter().map(|c| format!("{}", c)).collect()).unwrap_or_else(Vec::new);
        }

        pinyins.iter().rev().map(|pinyin| self.query(pinyin)).fold(None, |acc, curchars| -> Option<Box<dyn Iterator<Item = (String, &Trie)>>> {
            let curchars = match curchars {
                Some(c) => c,
                None => return None,
            };

            let inner = match acc {
                None => return Some(Box::new(curchars.iter().cloned().filter_map(|c| rev.get(c).map(|inner| (format!("{}", c), inner))))),
                Some(inner) => inner,
            };

            Some(Box::new(inner.cartesian_product(curchars.iter().cloned()).filter_map(|((s, t), c)| t.get(c).map(|inner| (format!("{}{}", c, s), inner)))))
        }).unwrap().map(|(s, _)| s).collect()
    }
}

impl Engine {
    fn get_transfer_count(&self, from: &str, to: &str) -> u64 {
        if from == "" {
            self.counter.get(to).cloned().unwrap_or(1)
        } else {
            self.two_gram.get(from).and_then(|bucket| bucket.get(to)).cloned().unwrap_or(1)
        }
    }

    pub fn init_trie(&mut self) {
        println!("Init trie...");
        let mut inserted = HashSet::new();

        // Traverse all words
        for (fw, bs) in self.two_gram.iter() {
            if !inserted.contains(fw) {
                inserted.insert(fw);
                self.trie.put_all(fw.chars().rev());
            }

            for (bw, _) in bs.iter() {
                if !inserted.contains(bw) {
                    inserted.insert(bw);
                    self.trie.put_all(bw.chars().rev());
                }
            }
        }
        println!("Trie initialized");
    }

    pub fn query<'a, S>(&self, segs: &[S], dict: &Dict) -> String
    where
        S: AsRef<str> + 'a,
    {
        // Viterbi
        let mut states: Vec<HashMap<String, f64>> = Vec::new();
        let mut paths: Vec<HashMap<String, String>> = Vec::new();
        
        // Initial state
        let mut initial_state = HashMap::new();
        initial_state.insert(String::new(), 1.0);
        states.push(initial_state);

        let mut initial_path = HashMap::new();
        initial_path.insert("".to_owned(), "".to_owned());
        paths.push(initial_path);

        for index in 0..segs.len() {
            let mut new_state: HashMap<String, f64> = HashMap::new();
            let mut new_path: HashMap<String, String> = HashMap::new();

            // Build word
            for wordlen in 0..MAX_WORD_LEN {
                if wordlen > index { break; }

                let state = &states[index-wordlen];
                let path = &paths[index-wordlen];

                for s in dict.build_words(&segs[(index-wordlen)..(index+1)], &self.trie) {
                    let (max_k, weighted_count, _total) =
                        state.iter().fold((String::new(), 0.0, 0), |(lk, lv, tot), (pk, pv)| {
                            let pair_count = self.get_transfer_count(pk, &s);
                            let cv = pv * pair_count as f64;

                            if cv < lv {
                                (lk, lv, tot + pair_count)
                            } else {
                                (pk.clone(), cv, tot + pair_count)
                            }
                        });

                    let cur_path = path.get(&max_k).unwrap();
                    new_path.insert(s.clone(), format!("{}{}", cur_path, s));

                    new_state.insert(s, weighted_count);
                }
            }

            // Normalize weighted counts
            let total_weight = new_state.iter().fold(0.0, |acc, (_, w)| acc + w);
            for (_, v) in new_state.iter_mut() {
                *v /= total_weight;
            }

            states.push(new_state);
            paths.push(new_path);
        }

        let (max_end, _) = states.pop().unwrap().into_iter().fold(
            (String::new(), 0.0),
            |l, (ck, cv)| if l.1 > cv { l } else { (ck, cv) },
        );

        paths.pop().unwrap().remove(&max_end).unwrap()
    }
}
