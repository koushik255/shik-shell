use itertools::Itertools;
use rayon::prelude::*;
use std::fs::read_dir;

use std::time::Instant;
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

    //let files_clone = files.i.map(|(e, _)| e.to_owned().to_owned());
    // YEAH THIS BLOCK UNDER ME IS TRASH
    //
    //
    // geting the files into owned
    let mut files_clone: Vec<PathBuf> = files
        .iter()
        .filter_map(|(f, _)| Some(f.to_owned().to_owned()))
        .by_ref()
        .collect();
    // files_clone = Vec<PathBuf>

    //
    //let mut files_clone22: Vec<PathBuf> = files.to_owned().to_owned().keys();
    //
    files_clone.sort();

    // let mut files_extenstion_unique: HashMap<PathBuf, i32> = HashMap::new();

    // let mut fuckyou: Vec<_> = files
    //     .iter()
    //     .filter_map(|(f, _)| {
    //         // getting the extension
    //         let didwa = f
    //             .extension()
    //             .map(|e| e.to_os_string().into_string().unwrap());
    //
    //         //need this to insert
    //         let wtbr = match didwa.clone() {
    //             Some(t) => t,
    //             None => "nada".to_string(),
    //         };
    //         // this whole blok is just putting the extentions in a hashmap to check if their unique
    //         files_extenstion_unique.insert(wtbr.clone(), 0);
    //         didwa
    //     })
    //     .collect();
    //

    //let mut value = files_extenstion_unique.clone();

    let unique: HashMap<String, i32> = files
        .into_keys()
        .filter_map(|p| {
            let ext = p.extension()?.to_string_lossy().into_owned();
            Some((ext, 0)) // key-value pair for collect()
        })
        .collect();
    // i can put checkdupes/vreating fps into here because im already itering over it
    //
    // ORDER OF ITER
    // so itering to collect key values
    // then im itering again to create a fileplus vec
    // then im iterting over the fileplus vec
    //
    // what if instead if just iterd over the extenstion and just checked if the extenstionending
    // with &this_exention and if it is then add that to vec plus

    let mut fp: Vec<FilePlus> = Vec::new();
    {
        fp = check_dupes_comp(&files_clone);
    }

    let mut all_this: Vec<FilePlus> = Vec::new();
    //
    // for full in fp {
    //     for s in unique.keys() {
    //         if *s == full.extenstion {
    //             if full.extenstion.ends_with(&this_exention) {
    //                 all_this.push(full.clone());
    //                 // so this does work
    //             }
    //         }
    //     }
    // }
    //
    fp.iter().for_each(|full| {
        if full.extenstion.ends_with(&this_exention) {
            all_this.push(full.to_owned());
        }
    });

    // if dir == false {
    // for this in all_this.clone() {
    //     println!("o file {}", this.full_path.display(),);
    // }
    // } else {
    //     for this in all_this.clone() {
    //         println!("m file {}", this.full_path.display());
    //     }
    // }

    if !dir {
        let more = walk_dir(dirs.clone(), this_exention.as_str());
        //println!("dirs {:?}", dirs.clone());
        let mut ddidy = more;
        ddidy.sort();
        ddidy.iter().for_each(|f| all_this.push(f.to_owned()));
        // for m in ddidy {
        //     //println!("more");
        //     all_this.push(m);
        // }
    };

    // for m in more {
    //     println!("m file {},folder {}", m.full_path.display(), m.extenstion);
    // }

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
