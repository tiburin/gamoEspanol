pub mod apps;
use apps::booktore;
use apps::Sphere;
use std::{fs, path::Path};

fn get_folder_path(folder_name: &str) -> &Path {
    if std::path::Path::new(folder_name).is_dir() {
        fs::remove_dir_all(folder_name).unwrap();
    }

    fs::create_dir_all(folder_name).unwrap();
    Path::new(folder_name)
}


pub fn write_booktore(sphere: &Sphere) {
    println!("\nBOOKTORE Running...");
    let store = booktore::init_get_system();
    let root = get_folder_path("booktore");
    for (tipo, data) in &sphere.vocabulary.data {
        let mut list: Vec<_> = data.iter().map(|n| n.word.to_owned()).collect();
        list.sort_by(|a, b| {
            let value_a = store.get(a).map_or(0, |x| x.0);
            let value_b = store.get(b).map_or(0, |x| x.0);
            value_b.cmp(&value_a) // Reversed order
        });

        let path = root.join(format!("{}.off", tipo));
        booktore::write_to_file_system(&path, &list, &store);
    }
}
pub fn write_build(sphere: &Sphere, keys: bool) {
    println!("\nBUILD Running...");
    let root = get_folder_path("build");

    for (tipo, list) in &sphere.vocabulary.data {
        let contents = list
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (rank, mas)| {
                let content = match keys {
                    false => format!("{}\n", mas.word),
                    true => format!("{},{},s\n", rank + 1, mas.word),
                };

                acc.push_str(&content);
                acc
            });

        let path = root.join(format!("{}.on", tipo));

        if contents.len() > 0 {
            fs::write(path, contents).unwrap();
        }
    }
}

pub fn start(name: &str) -> Sphere {
    let name = name.trim();
    Sphere::new().setup(name)
}
