mod support;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use std::{collections::HashMap, usize};
use support::{Mas, Sphere};
#[derive(Clone)]
struct State {
    sphere: Sphere,
}

impl State {
    fn new() -> State {
        let sphere = Sphere::new().setup();
        Self { sphere }
    }
    fn rename() -> State {
        let sphere = support::new_rename_fake_changes(State::new().sphere);
        Self { sphere }
    }
}
#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref GLOBAL: RwLock<HashMap<usize, State>> = {
        let id_one = 1;
        let id_two = 2;
        let mut store = HashMap::new();
        store.insert(id_one, State::new());
        store.insert(id_two, State::rename());
        RwLock::new(store)
    };
}

fn error_duplicate(old_mas: &Option<&Mas>, mas: &Mas) -> String {
    let old_mas = old_mas.unwrap();
    format!(
        " 
        This ({}) is duplicate in file {} line {}
        Word already inserted  in file {} in line {}
        ",
        mas.word, mas.oldfile_path, mas.line, old_mas.oldfile_path, old_mas.line
    )
}

fn error_invalid(mas: &Mas) -> String {
    format!(
        "
       This word ({}) have {} characters therefore is invalid in file {} line number {}
       Please make sure all words in {} only contains {} characters
        ",
        mas.word,
        mas.word.len(),
        mas.oldfile_path,
        mas.line,
        mas.oldfile_path,
        mas.folder_name
    )
}

fn error_comment() -> String {
    "Must have at least one comment to explain the process of renaming files".to_owned()
}

fn error_invalid_folder(folder: &str, folder_path: &PathBuf) -> String {
    format!(
        "This folder ({}) is invalid must be specified inside  {:?} remove it in /vocabulary
    ",
        folder, folder_path
    )
}

fn error_invalid_files(path: &PathBuf, type_path: &PathBuf, folder: &str) -> String {
    format!(
        " 
      {:?} is Invalid Must be specified inside {:?} remove it in folder ({})
    ",
        path, type_path, folder
    )
}

fn buildup<T: FnOnce(Sphere)>(id: usize, fnn: T) {
    let rw_lock = GLOBAL.read().unwrap();
    let state = rw_lock.get(&id).unwrap();
    fnn(state.sphere.clone());
}

fn function_invalid_files(sphere: &Sphere, path: &str, folder: &str, name: &str) -> String {
    let original_path = sphere.current_dir.join(format!(
        "vocabulary/{}/{}.{}",
        folder,
        name,
        Path::new(&path).extension().unwrap().to_str().unwrap()
    ));

    error_invalid_files(&original_path, &sphere.config.types.file_path, folder)
}

#[test]
fn rename_files() {
    buildup(2, |sphere| {
        let paths = sphere.rename_paths();
        assert_eq!(
            paths.len(),
            sphere.config.folders.list.len() * sphere.config.types.list.len()
        );
        for (old, new) in sphere.rename_files() {
            assert!(!old.exists());
            assert!(new.exists());
        }
        let old_comment = sphere.config.rename.comments.join("\n");
        let new_comments = fs::read_to_string(&sphere.config.rename.temporary_file_path).unwrap();
        assert_eq!(old_comment, new_comments);
        sphere.terminate();
    })
}

#[test]
fn rename() {
    buildup(2, |sphere| {
        assert!(sphere.config.rename.changes.len() == sphere.config.types.list.len());
        let mut start = 1;
        for change in sphere.config.rename.changes {
            let from = format!("{}_{}", change.from, start);
            assert_eq!(from, change.to);
            start += 1;
        }
    })
}

#[test]
fn rename_isset() {
    buildup(1, |sphere| {
        let rename = sphere.config.rename;
        assert!(rename.comments.len() >= 1, "{}", error_comment());
        assert!(rename.changes.len() == 0);
    })
}

#[test]
fn vocabulary_duplicate_files() {
    buildup(1, |sphere| {
        let store = support::get_keys_into_hashmap(&sphere.config.types.list);

        for folder in &sphere.config.folders.list {
            let path = &sphere.vocabulary.dir.join(&folder);
            if let Ok(dir_content) = fs_extra::dir::get_dir_content(path) {
                for file_path in dir_content.files {
                    let name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();
                    assert!(
                        store.contains(name),
                        "{}",
                        function_invalid_files(&sphere, &file_path, &folder, name)
                    );
                }
            }
        }
    })
}
#[test]
fn vocabulary_duplicate_directories() {
    buildup(1, |sphere| {
        let store = support::get_keys_into_hashmap(&sphere.config.folders.list);

        for folder in support::read_vocabulary_folders(&sphere.vocabulary.name) {
            let start_size = sphere.vocabulary.name.len() + "/".len();
            let valid_folder = store.contains(&folder[start_size..]);
            let file_path = &sphere.config.folders.file_path;

            assert!(valid_folder, "{}", error_invalid_folder(&folder, file_path));
        }
    })
}

#[test]
fn vocabulary() {
    let mut store = std::collections::HashMap::new();
    buildup(1, |sphere| {
        for mas in sphere.vocabulary.all_data {
            let is_same = mas.word.len().to_string() == mas.folder_name;
            let contains = store.get(&mas.word);

            assert!(is_same, "{}", error_invalid(&mas));
            assert!(contains.is_none(), "{}", error_duplicate(&contains, &mas));
            store.insert(mas.word.to_owned(), mas.clone());
            store.insert(mas.word.clone(), mas);
        }
    })
}

#[test]
fn system() {
    buildup(1, |sphere| {
        let folders_len = sphere.config.folders.list.len();
        let types_len = sphere.config.types.list.len();
        let vocabulary_len = sphere.vocabulary.data.len();
        assert_eq!(folders_len * types_len, vocabulary_len);
    });
}
