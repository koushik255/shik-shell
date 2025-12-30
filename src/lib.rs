use rayon::prelude::*;

use std::fs::read_dir;

use std::{collections::HashMap, path::PathBuf};

pub fn dir_list_one(path: &str, extention: String, dir: bool) -> Vec<FilePlus> {
    let mut udo: Vec<PathBuf> = Vec::new();
    if let Some(entieti) = read_dir(path).ok() {
        udo = entieti
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        //udo.sort();
    }

    let this_exention = extention;
    let mut dirs = HashMap::new();
    let mut files = HashMap::new();
    let mut i = 0;
    let mut ifile = 0;

    udo.iter().for_each(|f| {
        if f.is_dir() {
            dirs.insert(f, i);

            i += 1;
        } else {
            files.insert(f, ifile);
            ifile += 1;
        }
    });

    let mut files_clone: Vec<PathBuf> = files
        .iter()
        .filter_map(|(f, _)| Some(f.to_owned().to_owned()))
        .by_ref()
        .collect();

    files_clone.sort();

    let unique: HashMap<String, i32> = files
        .into_keys()
        .filter_map(|p| {
            let ext = p.extension()?.to_string_lossy().into_owned();

            // why not just check in here?

            Some((ext, 0)) // key-value pair for collect()
        })
        .collect();

    let mut fp: Vec<FilePlus> = Vec::new();
    {
        fp = check_dupes_comp(&files_clone);
    }

    let mut all_this: Vec<FilePlus> = Vec::new();

    //// try par iter here
    fp.iter().for_each(|full| {
        //println!("{}", full.full_path.display());

        if full.extenstion.ends_with(&this_exention) {
            all_this.push(full.to_owned());
        }
        //println!("yep");
    });
    //
    // let gamba: Rc<Cell<Vec<FilePlus>>> = Rc::new(Cell::new(Vec::<FilePlus>::new()));

    if !dir {
        let more = walk_dir(dirs.clone(), this_exention.as_str());
        //println!("dirs {:?}", dirs.clone());
        let mut ddidy = more;
        ddidy.sort();
        ddidy.iter().for_each(|f| all_this.push(f.to_owned()));
    };

    all_this
}
pub fn check_dupes_comp<T: Eq + std::hash::Hash + Clone>(vec: &[T]) -> Vec<FilePlus>
where
    PathBuf: From<T>,
{
    let vec = vec;
    // let start = Instant::now();

    let mut fp_vec = Vec::new();
    for file in vec {
        let path = PathBuf::from(file.to_owned());
        //println!("{}", path.display());

        let extention = match path.extension() {
            Some(e) => e.to_string_lossy().into_owned(),
            None => "DONOT".to_string(),
        };

        let f = FilePlus {
            full_path: path,
            extenstion: extention,
        };

        fp_vec.push(f);
    }
    // this doesnt really check for dupes as much as it just set it to a vec of fileplus

    return fp_vec;
}

// i need to make this a crate somewhere else and just use it in my front end / gui
// i dont want to work on this anymore to be honest

pub fn walk_dir(dirs: HashMap<&PathBuf, i32>, ext: &str) -> Vec<FilePlus> {
    // what if i made it async parrallelism??
    //
    dirs.keys()
        .par_bridge()
        .flat_map(|dir| {
            dir.to_str()
                .map(|path_str| dir_list_one(path_str, ext.to_string(), false))
                .unwrap_or_default()
        })
        .collect()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePlus {
    pub full_path: PathBuf,
    pub extenstion: String,
}

// where from here, i think its fast enough to where idgaf right now i could probably make it O^N
// while creating the FilePLus but
// ok maybe i can implement my own ffempeg or gstremer widget now? ive always wanted to do it
