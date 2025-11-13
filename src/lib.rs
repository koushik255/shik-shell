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

        udo.sort();
    }

    let this_exention = extention.clone();
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
    let mut files_clone: Vec<PathBuf> = files
        .iter()
        .filter_map(|(f, _)| Some(f.to_owned().to_owned()))
        .by_ref()
        .collect();
    files_clone.sort();
    let mut files_extenstion_unique = HashMap::new();
    let mut suf2: Vec<_> = files
        .iter()
        .filter_map(|(f, _)| {
            let didwa = f
                .extension()
                .map(|e| e.to_os_string().into_string().unwrap());
            let wtbr = match didwa.clone() {
                Some(t) => t,
                None => "nada".to_string(),
            };
            files_extenstion_unique.insert(wtbr.clone(), 0);
            didwa
        })
        .collect();

    suf2.sort();
    let fp = check_dupes_comp(&files_clone);

    let mut all_this: Vec<FilePlus> = Vec::new();

    for full in fp {
        for s in files_extenstion_unique.clone().into_keys() {
            if *s == full.extenstion {
                if full.extenstion.ends_with(&this_exention) {
                    all_this.push(full.clone());
                    // so this does work
                }
            }
        }
    }

    // if dir == false {
    for this in all_this.clone() {
        println!("o file {}", this.full_path.display(),);
    }
    // } else {
    //     for this in all_this.clone() {
    //         println!("m file {}", this.full_path.display());
    //     }
    // }

    if !dir {
        let mut more = walk_dir(dirs.clone(), this_exention.as_str());
        //println!("dirs {:?}", dirs.clone());
        let mut ddidy = more;
        ddidy.sort();
        for m in ddidy {
            //println!("more");
            all_this.push(m);
        }
    }
    // for m in more {
    //     println!("m file {},folder {}", m.full_path.display(), m.extenstion);
    // }

    all_this
}
// what if the walk dir had shared multabillity so i wouldnt need to look over the stuff in diddy
// again?
// ok thats a buns idea i should just have one vec where both files give me that the recursion is
// what would make this fast right? then would apppending to the list wait thats not even what
// takes long its not the appending to the list after which takes long its the filtering right?
// use channels to instead add all to a vec then return the vec and again and again each
// i need to learn rayon fully because that parrlelism is fucking goated

pub fn check_dupes_comp<T: Eq + std::hash::Hash + Clone>(vec: &[T]) -> Vec<FilePlus>
where
    PathBuf: From<T>,
{
    let vec = vec;

    let mut fp_vec = Vec::new();
    for file in vec {
        let path = PathBuf::from(file.to_owned().clone());
        //println!("{}", path.display());

        let extention = match path.extension() {
            Some(e) => e.to_string_lossy().into_owned(),
            None => "DONOT".to_string(),
        };

        let f = FilePlus {
            full_path: path.clone(),
            extenstion: extention,
        };
        // println!(
        //     "FILES PLUS {} EXTENSTION{:?}",
        //     f.full_path.display(),
        //     f.extenstion
        // );
        fp_vec.push(f);
    }
    //
    // let mut counts = HashMap::new();
    // // i need to make a struct which has both the full path and the
    // // extension
    // for item in vec {
    //     *counts.entry(item).or_insert(0) += 1;
    //     // dude hashmaps must be unique
    //     // its deadass just insert the entry but if you dont
    //     // (beacause its already in there) then add 1 to the value
    // }
    // let bomba = counts.clone();
    //
    // let herediddy = counts
    //     .into_iter()
    //     .filter(|(_, count)| *count > 1)
    //     .map(|(item, _)| item.clone())
    //     .collect();
    //
    // let folders = bomba.into_iter().map(|(item, _)| item.clone()).collect();
    //
    // return (herediddy, folders, fp_vec);
    return fp_vec;
}

// pub fn walk_dir(vec: HashMap<&PathBuf, i32>, ext: String) -> Vec<FilePlus> {
//     let dirs = vec;
//     let mut togeth: Vec<FilePlus> = Vec::new();
//
//     for dir in dirs.into_keys() {
//         //println!("Dir {}", dir.display());
//         //println!("{}", ext);
//         //
//         let files = dir_list_one(
//             dir.as_os_str().to_owned().into_string().as_ref().unwrap(),
//             ext.clone(),
//             false,
//         );
//         files.iter().for_each(|f| togeth.push(f.to_owned()));
//         // should make a check for if a dir has mkv i should run that async i think
//         // this needs a whole re work/ i need to make the dir_list function smaller, im doing so
//         // much work on every single iteration blud its ridiculus
//         // i should use channels
//
//         // files.sort();
//
//         // for file in files {
//         //     togeth.push(file);
//         // }
//     }
//     togeth
// }
// but wouldnt the check if file has mkv run read_dir again?
// but instead of listing it out it would be a bool right?
// or do i hashmap it and make it so if its a mkv then put into hash map
//

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
