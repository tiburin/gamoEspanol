use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::apps::booktore;
const MATCH: &str = "";
const MATCHEND: bool = true;
const SORT_BY_POPULAR: bool = false;
const MUST_CONTAINS_WORDS: bool = true;
const USING_BOOKTORE: bool = false;

type Tipo = HashSet<String>;

mod rule {
    pub struct Word {}
    impl Word {
        pub fn min() -> usize {
            2
        }
        pub fn max() -> usize {
            25
        }
    }
    pub fn is_min(word: &str) -> bool {
        word.len() < Word::min()
    }
    pub fn is_max(word: &str) -> bool {
        word.len() > Word::max()
    }
}
struct English;
impl English {
    fn start() -> i32 {
        97
    }
    fn end() -> i32 {
        122
    }

    fn is_range(value: i32) -> bool {
        value >= English::start() && value <= English::end()
    }
}
struct Str;
impl Str {
    fn rm_start(input: &str) -> &str {
        let mut start = 0;
        let mut list = input.chars();
        let mut current = list.next();
        while let Some(letter) = current {
            if !English::is_range(letter as i32) {
                start += letter.len_utf8();
            } else {
                return &input[start..input.len()];
            }
            current = list.next();
        }
        &input[start..input.len()]
    }
    fn rm_end(input: &str) -> &str {
        let mut end = Str::utf_count(input);
        let mut list = input.chars();
        let mut current = list.next_back();
        while let Some(letter) = current {
            if !English::is_range(letter as i32) {
                end -= letter.len_utf8();
            } else {
                return &input[0..end];
            }
            current = list.next_back();
        }
        &input[0..end]
    }
    fn rm_start_end(input: &str) -> &str {
        Str::rm_end(Str::rm_start(input))
    }
    fn valid_english(input: &str) -> bool {
        for letter in input.chars() {
            if !English::is_range(letter as i32) {
                return false;
            }
        }
        input.len() > 0
    }
    pub fn is_ing(input: &str) -> bool {
        if input.len() <= 3 {
            return false;
        }
        let start = input.len() - 3;
        &input[start..input.len()] == "ing"
    }
    pub fn is_ed(input: &str) -> bool {
        if input.len() <= 2 {
            return false;
        }
        let start = input.len() - 2;
        &input[start..input.len()] == "ed"
    }
    pub fn is_plural(input: &str) -> bool {
        if input.len() <= 2 {
            return false;
        }
        let start = input.len() - 1;
        let letter = &input[start..input.len()];
        letter == "s" && &input[start - 1..start] != "s"
    }
    pub fn is_match(input: &str) -> bool {
        if input.len() < 4 || MATCH.trim() == "" {
            return false;
        }
        if MATCHEND {
            &input[input.len() - MATCH.len()..input.len()] == MATCH
        } else {
            &input[0..MATCH.len()] == MATCH
        }
    }
    pub fn get_word(word: &str) -> Option<&str> {
        if rule::is_min(word) || rule::is_max(word) {
            return None;
        }
        Some(word)
    }
    fn utf_count(input: &str) -> usize {
        input.chars().fold(0, |acc, letter| acc + letter.len_utf8())
    }
}

struct Parse;
impl Parse {
    fn sort_popular(list: Vec<&str>) -> Vec<&str> {
        if SORT_BY_POPULAR {
            let store = list.iter().fold(HashMap::new(), |mut store, word| {
                if let Some(word) = store.get_mut(word) {
                    *word += 1;
                } else {
                    store.insert(*word, 1);
                }
                store
            });

            let mut new_list: Vec<_> = store.into_iter().collect();
            new_list.sort_by(|(_, a), (_, b)| a.cmp(b));
            new_list.reverse();
            return new_list.iter().map(|(word, _)| *word).collect();
        }
        list
    }
    pub fn lines(input: &str) -> Vec<String> {
        let mut cache: HashSet<String> = HashSet::new();
        let mut list = Vec::new();
        for word in Parse::sort_popular(input.split_whitespace().collect()) {
            let word = word.trim().to_lowercase();
            let word = Str::rm_start_end(&word);
            if let Some(word) = Str::get_word(word) {
                let contain = cache.contains(word);
                if !contain && Str::valid_english(word) {
                    list.push(word.to_owned());
                    cache.insert(word.to_owned());
                }
            }
        }

        list
    }
}

#[derive(Debug)]
struct Writer {
    path: String,
    content: String,
}
impl Writer {
    fn new(path: String, content: String) -> Self {
        Self { path, content }
    }
}

#[derive(Debug)]
struct Voc {
    list: Vec<String>,
    ing: Vec<String>,
    ed: Vec<String>,
    plural: Vec<String>,
    simple: Vec<String>,
    matching: Vec<String>,
    writer: Vec<Option<Writer>>,
}
impl Voc {
    fn new(list: Vec<String>) -> Self {
        Self {
            list,
            ing: vec![],
            ed: vec![],
            plural: vec![],
            simple: vec![],
            matching: vec![],
            writer: Vec::new(),
        }
    }

    fn write(name: &str, list: &Vec<String>, mas: &Mas) -> Option<Writer> {
        let path = if name != "word.on" {
            mas.path_parts.join(name)
        } else {
            mas.root.join(name)
        };
        let path = path.into_os_string().into_string().unwrap();
        if list.len() > 0 || name == "word.on" {
            let content = format!("{}\n", list.join("\n"));
            let mi_writter = Writer::new(path, content);
            return Some(mi_writter);
        }

        None
    }

    fn compose(&mut self, book_data: &HashMap<String, String>, mas: &Mas) -> &mut Self {
        self.writer.push(Voc::write("word.on", &self.list, mas));
        self.writer
            .push(Voc::write("match.on", &self.matching, mas));

        self.writer
            .append(&mut Voc::insert(&self.ed, "N", book_data, mas));
        self.writer
            .append(&mut Voc::insert(&self.ing, "O", book_data, mas));
        self.writer
            .append(&mut Voc::insert(&self.plural, "P", book_data, mas));
        self.writer
            .append(&mut Voc::insert(&self.simple, "F", book_data, mas));

        self
    }
    fn direct_data(&mut self) -> &mut Self {
        for word in self.list.clone() {
            if Str::is_match(&word) {
                self.matching.push(word)
            } else {
                self.simple.push(word)
            }
        }

        self
    }
    fn collect(
        store: &mut HashMap<usize, Vec<String>>,
        letter: &str,
        book_data: &HashMap<String, String>,
        mas: &Mas,
    ) -> Vec<Option<Writer>> {
        let mut inner = vec![];
        for rank in store.keys() {
            let list = store.get(rank).unwrap();
            let name = format!("{}-{}.off", letter, rank);
            let name = mas
                .path_parts
                .join(name)
                .into_os_string()
                .into_string()
                .unwrap();
            booktore::write_to_file(Path::new(&name), list, book_data);
            inner.push(Voc::write(&format!("{}-{}.on", letter, rank), list, mas));
        }
        inner
    }

    fn insert(
        list: &Vec<String>,
        letter: &str,
        book_data: &HashMap<String, String>,
        mas: &Mas,
    ) -> Vec<Option<Writer>> {
        let mut store = Voc::store();

        for word in list {
            if let Some(node) = store.get_mut(&word.len()) {
                node.push(word.to_owned());
            } else {
                panic!("wrong length invalid data should't be at this point")
            }
        }
        Voc::collect(&mut store, letter, book_data, mas)
    }
    fn store() -> HashMap<usize, Vec<String>> {
        let mut store = HashMap::new();
        let mut start = rule::Word::min();
        while start <= rule::Word::max() {
            store.insert(start, vec![]);
            start += 1;
        }
        store
    }
    fn write_to_files(&mut self) {
        for writer in &self.writer {
            if let Some(write) = writer {
                fs::write(&write.path, &write.content).unwrap();
            }
        }
        eprintln!("Mas: {}", self.list.len());
    }
}

struct App {
    off_content: String,
    on_content: String,
    store: HashSet<String>,
}
fn update_off_file(list: Vec<String>, mas: &Mas) {
    let mut acc = vec![];
    let mut store = HashSet::new();
    for word in list {
        store.insert(word);
    }

    for word in Parse::lines(&fs::read_to_string(&mas.path_off).unwrap()) {
        if !store.contains(&word) {
            acc.push(word);
        }
    }
    acc.sort_by(|a, b| a.len().cmp(&b.len()));
    fs::write(&mas.path_off, acc.join("\n")).unwrap();
}

struct Forbid;
impl Forbid {
    fn start(mut store: Tipo, listas: Vec<&Vec<String>>) -> Tipo {
        for lista in listas {
            for line in lista {
                store.insert(line.to_owned());
            }
        }
        store
    }
}
impl App {
    fn new(on_content: String, off_content: String) -> Self {
        Self {
            on_content,
            off_content,
            store: HashSet::new(),
        }
    }
    fn forbid(&mut self, vocabulary_list: &Vec<String>, off_list: &Vec<String>) -> &mut Self {
        let store: Tipo = HashSet::new();
        self.store = Forbid::start(store, vec![vocabulary_list, off_list]);
        self
    }
    fn start(&mut self, vocabulary_list: &Vec<String>) -> Vec<String> {
        let off_list = Parse::lines(&self.off_content);
        self.forbid(vocabulary_list, &off_list);

        let data = Parse::lines(&self.on_content).into_iter();
        if MUST_CONTAINS_WORDS {
            let mut errors = vec![];
            let on_file_data = data.clone().fold(HashSet::new(), |mut acc, word| {
                acc.insert(word);
                acc
            });

            for word in off_list {
                if !on_file_data.contains(&word) {
                    errors.push(word);
                }
            }

            for word in &errors {
                println!("({}) does not exist! in word.on", &word);
            }
            if errors.len() > 0 {
                panic!("contains duplicate words in total: {}", errors.len());
            }
        }

        data.filter(|n| !self.store.contains(n)).collect()
    }
}

#[derive(Clone)]
pub struct Mas {
    root: PathBuf,
    path_on: PathBuf,
    path_off: PathBuf,
    path_parts: PathBuf,
}

impl Mas {
    pub fn new() -> Self {
        Self {
            root: PathBuf::new(),
            path_on: PathBuf::new(),
            path_off: PathBuf::new(),
            path_parts: PathBuf::new(),
        }
    }

    fn get_path(self, name: &str) -> PathBuf {
        if name == "aparter" {
            PathBuf::from(name)
        } else {
            self.root
        }
    }
    fn get_path_on(self) -> PathBuf {
        self.root.join("word.on")
    }
    fn get_path_off(self) -> PathBuf {
        self.root.join("word.off")
    }
    fn get_path_parts(self) -> PathBuf {
        self.root.join("parts")
    }

    pub fn setup(mut self, name: &str) -> Self {
        if name == "aparter" {
            println!("\nAPARTER Running...");
        } else {
            println!("\nNORMAL Running...");
        }
        self.root = self.clone().get_path(name);
        self.path_on = self.clone().get_path_on();
        self.path_off = self.clone().get_path_off();
        self.path_parts = self.clone().get_path_parts();
        self
    }
    pub fn start(self, not_allow: Vec<String>) -> Vec<String> {
        let book_data = if USING_BOOKTORE {
            booktore::init()
        } else {
            HashMap::new()
        };

        for inner_path in vec![&self.path_on, &self.path_off] {
            if !inner_path.exists() {
                fs::File::create(inner_path).unwrap();
            }
        }
        if self.path_parts.is_dir() {
            fs::remove_dir_all(&self.path_parts).unwrap();
        }
        let on_content = fs::read_to_string(&self.path_on).unwrap();
        let off_content = fs::read_to_string(&self.path_off).unwrap();

        fs::create_dir(&self.path_parts).unwrap();
        let list = App::new(on_content, off_content).start(&not_allow);
        Voc::new(list.clone())
            .direct_data()
            .compose(&book_data, &self)
            .write_to_files();
        update_off_file(not_allow, &self);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rule::Word;
    #[test]
    fn store_test() {
        let store = Voc::store();
        for rank in Word::min()..Word::max() {
            assert_eq!(store.get(&rank), Some(&vec![]));
        }
    }
    #[test]
    fn get_word_test() {
        let max_letter = "a".repeat(Word::max() + 1);
        let min_letter = "a".repeat(Word::min() - 1);
        assert!(Str::get_word(&max_letter).is_none());
        assert!(Str::get_word(&min_letter).is_none());
        assert_eq!(Str::get_word("hello"), Some("hello"));
    }
    #[test]
    fn is_plural_test() {
        assert!(!Str::is_plural("es"));
        assert!(!Str::is_plural("discuss"));
        assert!(Str::is_plural("houses"));
    }
    #[test]
    fn is_ed_test() {
        assert!(Str::is_ed("worked"));
        assert!(!Str::is_ed("ed"));
    }
    #[test]
    fn is_ing_test() {
        assert!(Str::is_ing("working"));
        assert!(!Str::is_ing("worknng"));
        assert!(!Str::is_ing(""));
    }
    #[test]
    fn valid_english_test() {
        assert!(Str::valid_english("z"));
        assert!(Str::valid_english("ab"));
        assert!(!Str::valid_english(""));
    }
    #[test]
    fn rm_start_end_test() {
        assert_eq!(Str::rm_start_end("  "), "");
        assert_eq!(Str::rm_start_end(" hello "), "hello");
        assert_eq!(Str::rm_start_end("1/#*hello1/#*"), "hello");
    }
    #[test]
    fn rm_start_test() {
        assert_eq!(Str::rm_start("  "), "");
        assert_eq!(Str::rm_start(" hello"), "hello");
        assert_eq!(Str::rm_start("1/#*hello"), "hello");
    }
    #[test]
    fn rm_end_test() {
        assert_eq!(Str::rm_end("  "), "");
        assert_eq!(Str::rm_end("hello "), "hello");
        assert_eq!(Str::rm_end("hello1/#*"), "hello");
    }
}
