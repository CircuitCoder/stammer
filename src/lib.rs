use failure::Error;
use hashbrown::hash_map::Entry;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::Path;

#[derive(Default)]
pub struct Store {
    two_gram: HashMap<char, (HashMap<char, u64>, u64)>,

    one_gram: HashMap<char, u64>,
    one_gram_total: u64,
}

impl Store {
    pub fn new() -> Store {
        Default::default()
    }

    pub fn put_single(&mut self, a: char) {
        match self.one_gram.entry(a) {
            Entry::Occupied(mut store) => *store.get_mut() += 1,
            Entry::Vacant(store) => {
                store.insert(1);
            }
        }

        self.one_gram_total += 1;
    }

    pub fn put_pair(&mut self, a: char, b: char) {
        match self.two_gram.entry(a) {
            Entry::Occupied(mut inner) => {
                let inner = inner.get_mut();

                match inner.0.entry(b) {
                    Entry::Occupied(mut store) => *store.get_mut() += 1,
                    Entry::Vacant(store) => {
                        store.insert(1);
                    }
                };

                inner.1 += 1;
            },
            Entry::Vacant(inner) => {
                let mut value = HashMap::new();
                value.insert(b, 1);
                inner.insert((value, 1));
            }
        }
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
}

const SINGLE_RATIO: f64 = 0.0;
const PAIR_RATIO: f64 = 1.0;

#[derive(Serialize, Deserialize)]
pub struct Engine {
    two_gram: HashMap<char, HashMap<char, f64>>,
    one_gram: HashMap<char, f64>,
}

impl From<Store> for Engine {
    fn from(store: Store) -> Engine {
        Engine {
            two_gram: HashMap::from_iter(store.two_gram.iter().map(|(k, (vh, vt))| (*k, HashMap::from_iter(vh.iter().map(|(ik, iv)| (*ik, *iv as f64 / *vt as f64)))))),
            one_gram: HashMap::from_iter(store.one_gram.iter().map(|(k, v)| (*k, *v as f64 / store.one_gram_total as f64))),
        }
    }
}

impl Engine {
    pub fn query<'a, I, S>(&self, segs: I, dict: &Dict) -> Vec<char>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str> + 'a,
    {
        // Viterbi
        let mut state: HashMap<char, f64> = self.one_gram.clone();
        let mut path: HashMap<char, Vec<char>> = HashMap::new();

        for s in segs.into_iter() {
            let mut new_state: HashMap<char, f64> = HashMap::new();
            let mut new_path: HashMap<char, Vec<char>> = HashMap::new();

            let choices = match dict.query(s) {
                Some(s) => s,
                None => return Vec::new(),
            };

            for c in choices.iter() {
                let (pair_max_k, pair_max_v) =
                    state.iter().fold((' ', -1.0), |(lk, lv), (pk, pv)| {
                        let pair_prob = self
                            .two_gram
                            .get(pk)
                            .and_then(|store| store.get(c))
                            .cloned()
                            .unwrap_or(0.0);
                        let cv = pv * pair_prob;

                        if cv < lv {
                            (lk, lv)
                        } else {
                            (*pk, cv)
                        }
                    });

                let single_v = self
                    .one_gram
                    .get(c)
                    .cloned()
                    .unwrap_or(0.0);
                let prob = single_v * SINGLE_RATIO + pair_max_v * PAIR_RATIO;

                new_state.insert(*c, prob);

                let mut cur_path = path.get(&pair_max_k).cloned().unwrap_or_else(Vec::new);
                cur_path.push(*c);
                new_path.insert(*c, cur_path);
            }

            state = new_state;
            path = new_path;
        }

        let (max_end, _) = state.iter().fold(
            (' ', -1.0),
            |l, (ck, cv)| if l.1 > *cv { l } else { (*ck, *cv) },
        );
        path.remove(&max_end).unwrap()
    }
}
