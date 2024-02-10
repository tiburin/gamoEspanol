use std::collections::HashMap;
use std::collections::HashSet;
use std::{
    env, fs,
    path::{Path, PathBuf},
    usize,
};
use uuid::Uuid;
fn copy_dirs(from: &PathBuf, to: &PathBuf) -> Result<u64, fs_extra::error::Error> {
    fs_extra::copy_items(&[from], to, &fs_extra::dir::CopyOptions::new())
}
#[derive(Debug, Clone)]
pub struct Change {
    pub from: String,
    pub to: String,
}

#[derive(Clone)]
pub struct Rename {
    name: String,
    file_name: String,
    pub file_path: PathBuf,
    pub temporary_file_path: PathBuf,
    pub comments: Vec<String>,
    pub changes: Vec<Change>,
}
impl Rename {
    fn new() -> Self {
        Self {
            name: String::new(),
            file_name: String::new(),
            comments: Vec::new(),
            changes: Vec::new(),
            file_path: PathBuf::new(),
            temporary_file_path: PathBuf::new(),
        }
    }
    fn setup(mut self, config: &Config, sphere: &Sphere) -> Self {
        self.name = "rename".to_string();
        self.file_name = format!("{}.on", self.name);
        self.file_path = config.dir.join(&self.file_name);
        let temp = format!("config/{}", self.file_name);
        self.temporary_file_path = sphere.temporary_dir.join(temp);
        ReadParseFile::new(&self.file_path).split_rename(self)
    }
}
impl Config {
    fn new() -> Self {
        Self {
            dir: PathBuf::new(),
            folders: Folders::new(),
            types: Types::new(),
            rename: Rename::new(),
        }
    }

    fn setup(mut self, sphere: &Sphere) -> Self {
        self.dir = sphere.current_dir.join("config");
        self.folders = self.folders.clone().setup(&self);
        self.types = self.types.clone().setup(&self);
        self.rename = self.rename.clone().setup(&self, sphere);
        self
    }
}
#[derive(Clone)]
pub struct Config {
    dir: PathBuf,
    pub folders: Folders,
    pub rename: Rename,
    pub types: Types,
}

#[derive(Clone)]
pub struct Folders {
    name: String,
    file_name: String,
    pub file_path: PathBuf,
    pub list: Vec<String>,
}
impl Folders {
    fn readparse(&self) -> Vec<String> {
        ReadParseFile::new(&self.file_path).split_whitespace()
    }
    fn new() -> Self {
        Self {
            name: String::new(),
            file_name: String::new(),
            file_path: PathBuf::new(),
            list: Vec::new(),
        }
    }
    fn setup(mut self, config: &Config) -> Self {
        self.name = "folders".to_string();
        self.file_name = format!("{}.on", self.name);
        self.file_path = config.dir.join(&self.file_name);
        self.list = self.readparse();
        self
    }
}
#[derive(Clone)]
pub struct Types {
    name: String,
    file_name: String,
    pub file_path: PathBuf,
    pub list: Vec<String>,
}
impl Types {
    fn new() -> Self {
        Self {
            name: String::new(),
            file_name: String::new(),
            file_path: PathBuf::new(),
            list: Vec::new(),
        }
    }
    fn setup(mut self, config: &Config) -> Self {
        self.name = "types".to_string();
        self.file_name = format!("{}.on", self.name);
        self.file_path = config.dir.join(&self.file_name);
        self.list = ReadParseFile::new(&self.file_path).split_whitespace();
        self
    }
}

struct ReadParseFile {
    content: String,
}

impl ReadParseFile {
    fn new<T: AsRef<Path>>(name: T) -> Self {
        let content = fs::read_to_string(name).unwrap();
        Self { content }
    }

    fn split_mas(self, mas: Mas) -> Vec<Mas> {
        self.content
            .split("\n")
            .enumerate()
            .map(|(index, word)| {
                let mut new_mas = mas.clone();
                new_mas.line = index + 1;
                new_mas.word = word.trim().to_owned();
                new_mas
            })
            .filter(|mas| mas.word.len() > 0)
            .collect()
    }

    fn split_whitespace(&self) -> Vec<String> {
        self.content
            .split_whitespace()
            .map(|line| line.trim())
            .filter(|line| line.len() > 0)
            .map(|line| line.to_owned())
            .collect()
    }

    fn split_rename(self, mut rename: Rename) -> Rename {
        let list: Vec<_> = self
            .content
            .split("\n")
            .map(|line| line.trim())
            .filter(|line| line.len() > 0)
            .collect();
        let comments: Vec<String> = list
            .iter()
            .take_while(|line| &line[0..2] == "//")
            .map(|line| line.to_string())
            .collect();
        let rest: Vec<_> = list
            .iter()
            .skip(comments.len())
            .map(|line| line.to_owned())
            .collect();

        let changes: Vec<_> = rest
            .iter()
            .map(|line| {
                let inner: Vec<_> = line.split("to:").collect();
                Change {
                    from: inner[0][5..].trim().to_owned(),
                    to: inner[1].trim().to_owned(),
                }
            })
            .collect();

        rename.comments = comments;
        rename.changes = changes;
        rename
    }
}

#[derive(Clone)]
pub struct Mas {
    pub line: usize,
    pub word: String,
    file_path: String,
    pub oldfile_path: String,
    pub folder_name: String,
    type_name: String,
}

impl Mas {
    fn new() -> Self {
        Self {
            line: 0,
            word: String::new(),
            file_path: String::new(),
            oldfile_path: String::new(),
            folder_name: String::new(),
            type_name: String::new(),
        }
    }

    fn build(mut self, sphere: &Sphere, folder_name: &String, type_name: &String) -> Self {
        let file_name = format!("{}/{}.on", folder_name, type_name);
        let dir = sphere.vocabulary.dir.to_str().unwrap();
        let old_dir = sphere.current_dir.to_str().unwrap();

        self.folder_name = folder_name.to_owned();
        self.type_name = type_name.to_owned();
        self.file_path = format!("{}/{}", &dir, file_name);
        self.oldfile_path = format!("{}/vocabulary/{}", old_dir, file_name);
        self
    }
}

#[derive(Clone)]
pub struct Vocabulary {
    pub name: String,
    pub dir: PathBuf,
    pub data: Vec<Vec<Mas>>,
    pub all_data: Vec<Mas>,
    pub release: HashMap<String, Vec<Mas>>,
}
impl Vocabulary {
    fn new() -> Self {
        Self {
            name: String::new(),
            data: Vec::new(),
            dir: PathBuf::new(),
            all_data: Vec::new(),
            release: HashMap::new(),
        }
    }
    fn compose_folders(&self, sphere: &Sphere) -> Vec<Vec<Mas>> {
        let mut acc = vec![];
        for folder_name in &sphere.config.folders.list {
            for type_name in &sphere.config.types.list {
                let path = self.dir.join(format!("{}/{}.on", folder_name, type_name));
                let mas = Mas::new().build(sphere, &folder_name, &type_name);
                acc.push(ReadParseFile::new(path).split_mas(mas));
            }
        }
        acc
    }
    fn put_all_data(&self) -> Vec<Mas> {
        let mut acc = vec![];

        for data in &self.data {
            for mas in data {
                acc.push(mas.clone());
            }
        }
        acc
    }
    fn insert_release(&self, sphere: &Sphere) -> HashMap<String, Vec<Mas>> {
        let mut store = HashMap::new();

        for tipo in &sphere.config.types.list {
            store.insert(tipo.clone(), Vec::new());
        }
        for mas in &self.all_data {
            let vec = store.get_mut(&mas.type_name).unwrap();
            vec.push(mas.clone());
        }
        store
    }

    fn setup(mut self, sphere: &Sphere) -> Self {
        self.name = "vocabulary".to_string();
        self.dir = sphere.temporary_dir.join(&self.name);
        self.data = self.compose_folders(sphere);
        self.all_data = self.put_all_data();
        self.release = self.insert_release(sphere);
        self
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub current_dir: PathBuf,
    temporary_dir: PathBuf,
    pub config: Config,
    pub vocabulary: Vocabulary,
}

impl Sphere {
    pub fn new() -> Self {
        Self {
            current_dir: PathBuf::new(),
            temporary_dir: PathBuf::new(),
            config: Config::new(),
            vocabulary: Vocabulary::new(),
        }
    }
    pub fn rename_paths(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut acc = Vec::new();
        for folder in &self.config.folders.list {
            let folder_dir = self.vocabulary.dir.join(folder);
            for change in &self.config.rename.changes {
                let old_path = folder_dir.join(format!("{}.on", change.from));
                let new_path = folder_dir.join(format!("{}.on", change.to));
                acc.push((old_path, new_path));
            }
        }
        acc
    }

    pub fn rename_files(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut acc = Vec::new();
        for folder in &self.config.folders.list {
            let folder_dir = self.vocabulary.dir.join(folder);
            for change in &self.config.rename.changes {
                let old_path = folder_dir.join(format!("{}.on", change.from));
                let new_path = folder_dir.join(format!("{}.on", change.to));
                fs::rename(&old_path, &new_path).unwrap();
                acc.push((old_path, new_path));
            }
        }
        let comments = self.config.rename.comments.join("\n");
        fs::write(&self.config.rename.temporary_file_path, comments).unwrap();
        acc
    }

    pub fn setup(mut self) -> Self {
        let uuid_id = format!("{}", Uuid::new_v4());
        let temporary_dir = env::temp_dir().join(&uuid_id);
        let current_dir = env::current_dir().unwrap();

        if temporary_dir.is_dir() {
            fs::remove_dir_all(&temporary_dir).unwrap()
        }
        fs::create_dir(&temporary_dir).unwrap();

        for name in vec!["config", "vocabulary"] {
            copy_dirs(&current_dir.join(name), &temporary_dir).unwrap();
        }

        for name in vec!["word.on", "word.off"] {
            fs::copy(&current_dir.join(name), &temporary_dir.join(name)).unwrap();
        }
        self.temporary_dir = temporary_dir;
        self.current_dir = current_dir;

        self.config = self.config.clone().setup(&self);
        self.vocabulary = self.vocabulary.clone().setup(&self);
        env::set_current_dir(&self.temporary_dir).unwrap();
        self
    }

    pub fn terminate(self) {
        fs::remove_dir_all(self.temporary_dir).unwrap();
    }
}

pub fn get_keys_into_hashmap(list: &Vec<String>) -> HashSet<String> {
    let mut store = HashSet::new();
    for folder in list {
        store.insert(folder.to_owned());
    }
    store
}

fn write_rename_content(config: &Config) {
    let content =
        config
            .types
            .list
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (index, tipo)| {
                let new_type = format!("{}_{}", tipo, &index + 1);
                let line = format!("from: {} to: {} \n", tipo, new_type);
                acc.push_str(&line);
                acc
            });

    let comments = config.rename.comments.join("\n");
    let content = format!("{}\n {}", comments, content);
    fs::write(&config.rename.temporary_file_path, content).unwrap();
}

pub fn read_vocabulary_folders(name: &str) -> Vec<String> {
    let mut acc = Vec::new();
    if let Ok(dir) = fs_extra::dir::get_dir_content(name) {
        for folder in dir.directories {
            if folder != name {
                acc.push(folder);
            }
        }
    }
    acc
}
pub fn new_rename_fake_changes(mut sphere: Sphere) -> Sphere {
    write_rename_content(&sphere.config);
    let content = fs::read_to_string(&sphere.config.rename.temporary_file_path).unwrap();
    sphere.config.rename = ReadParseFile { content }.split_rename(sphere.config.rename);
    sphere
}
