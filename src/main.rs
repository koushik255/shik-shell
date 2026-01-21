use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use gtk4::gdk::Key;
use gtk4::{gdk, glib, prelude::*, ListBox};
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, EventControllerKey, Label, Orientation,
    Picture, ScrolledWindow, SearchEntry,
};
use gtk4_layer_shell::{Layer, LayerShell};
use rayon::prelude::*;

const CSS: &str = r#"
window.main-window {
    background-color: rgba(30, 30, 30, 0.92);
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
}

window {
    background-color: rgba(30, 30, 30, 0.92);
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
}

listbox {
    background-color: transparent;
    border-radius: 12px;
    padding: 8px;
}

listbox row {
    background-color: rgba(20, 20, 20, 0.95);
    border-radius: 8px;
    margin: 4px 0;
    padding: 12px 16px;
    transition: background-color 200ms ease, transform 200ms ease;
}

listbox row:hover {
    background-color: rgba(35, 35, 35, 0.95);
    transform: translateX(4px);
}

listbox row:selected {
    background-color: rgba(100, 149, 237, 0.6);
    box-shadow: 0 0 0 2px rgba(100, 149, 237, 0.7);
}

listbox row label {
    color: #4a4a4a;
    font-size: 14px;
}

label.file-label {
    color: #3a3a3a;
    font-family: system-ui, -apple-system, sans-serif;
    font-weight: 500;
    letter-spacing: 0.3px;
}

scrolledwindow {
    background-color: transparent;
    border-radius: 12px;
}

scrolledwindow > scrollbar {
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
}

scrolledwindow > scrollbar slider {
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    min-width: 8px;
    min-height: 8px;
}

scrolledwindow > scrollbar slider:hover {
    background-color: rgba(255, 255, 255, 0.35);
}

picture {
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

picture.preview-image {
    background-color: rgba(0, 0, 0, 0.3);
    border-radius: 12px;
    padding: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

box {
    background-color: transparent;
}
"#;

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(CSS);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[derive(Parser)]
struct Cli {
    #[arg(default_value = ".")]
    folder_path: Option<String>,
}

fn main() -> glib::ExitCode {
    let cli = Cli::parse();

    let folder_path = cli.folder_path.expect("error option");

    let app = Application::builder()
        .application_id("com.example.gtkshell")
        .build();

    app.connect_activate(move |app| build_ui(app, &folder_path));
    app.run_with_args(&[] as &[&str])
}

type SharedVec<T> = Rc<RefCell<Vec<T>>>;

fn build_ui(app: &Application, path: &str) {
    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(1200)
        .default_height(800)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.add_css_class("main-window");

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();
    scrolled.set_margin_start(12);
    scrolled.set_margin_end(12);
    scrolled.set_margin_top(12);
    scrolled.set_margin_bottom(12);

    let listbox = ListBox::new();
    listbox.add_css_class("boxed-list");
    listbox.add_css_class("rich-list");

    let right_pane = Box::new(Orientation::Vertical, 12);
    right_pane.set_margin_start(12);
    right_pane.set_margin_end(12);
    right_pane.set_margin_top(12);
    right_pane.set_margin_bottom(12);
    right_pane.add_css_class("preview-pane");

    let display_pic = Picture::new();
    display_pic.add_css_class("preview-image");
    right_pane.append(&display_pic);

    // empty state holder
    // how to init as nothing?
    //let s_files: SharedVec<FilePlus> = Rc::new(RefCell::new(None.unwrap()));

    // make this function or sometihng
    // what this would open would depend on the command flag opening
    // you would have recursive open, open passive, and open default (Downloads)/userset
    //
    let mut files: Vec<FilePlus> = Vec::new();
    //let files = dir_list_one(path, "mkv".to_string(), false);
    // so either set that as it or if no flag is passed have it do orther
    if path == "." {
        let current_dir = std::env::current_dir().unwrap();
        println!("{}", current_dir.display());
        files = list_self_dir(current_dir.to_str().unwrap());
    } else {
        files = dir_list_one(path, "mkv".to_string(), false);
    }

    let job_dude = files.clone();
    let all_files: Rc<RefCell<Vec<FilePlus>>> = Rc::new(RefCell::new(job_dude.clone()));
    let s_files: SharedVec<FilePlus> = Rc::new(RefCell::new(job_dude));
    let path_control = Rc::new(RefCell::new(PathBuf::new()));
    let dirs_only: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search files..."));
    search_entry.set_margin_bottom(8);

    let all_files_search = all_files.clone();
    let s_files_search = s_files.clone();
    let listbox_search = listbox.clone();
    search_entry.connect_search_changed(move |entry| {
        let search_text = entry.text().to_string();
        let files = all_files_search.borrow().to_owned();
        files.update_screen_with_search(&search_text, &listbox_search, s_files_search.clone());
    });

    // job_dude
    //     .into_iter()
    //     .for_each(|e| s_files.borrow_mut().push(e));

    let files1 = s_files.clone().borrow().to_owned();
    for file in files1 {
        file.add_to_listbox(&listbox);
    }

    //
    let key_ctl = EventControllerKey::new();
    window.add_controller(key_ctl.clone());

    let window2 = window.clone();
    let listbox_remove = listbox.clone();
    let s_files_remove_all = s_files.clone();
    let all_files_remove = all_files.clone();
    let glob_path_cl = path_control.clone();
    let search_entry_key = search_entry.clone();
    let dirs_only_key = dirs_only.clone();
    key_ctl.connect_key_pressed(move |_, key, _code, _modifier| {
        if key == Key::t || key == Key::T {
            if search_entry_key.has_focus() {
                listbox_remove.grab_focus();
            } else {
                search_entry_key.grab_focus();
            }
            return glib::Propagation::Stop;
        }
        if key == Key::Escape {
            if search_entry_key.has_focus() {
                search_entry_key.set_text("");
                listbox_remove.grab_focus();
                return glib::Propagation::Stop;
            }
            window2.clone().close();
            return glib::Propagation::Stop;
        }
        if key == Key::S {
            // let replace_file = list_self_dir("/home/koushikk/Downloads");
            // all i would need to do now is just make it so that the current directory im in is
            // this file patha
            let mut path_set = false;
            if glob_path_cl.clone().borrow().exists() {
                listbox_remove.remove_all();
                s_files_remove_all.borrow_mut().clear();
                println!("Removed all Listbox elements!");

                println!(
                    "this exist fake past test btw {}",
                    glob_path_cl.clone().borrow().display()
                );
                path_set = true;
            } else {
                println!("nope fucking you path KEY:S");
            }
            let mut current_dir = PathBuf::new();
            if path_set {
                current_dir = glob_path_cl.clone().borrow().to_path_buf();
            } else {
                current_dir = std::env::current_dir().unwrap();
            }
            let replace_file = list_self_dir(current_dir.to_str().unwrap());
            *all_files_remove.borrow_mut() = replace_file.clone();
            println!("Using trait");
            replace_file.append_to_screen(&listbox_remove, s_files_remove_all.clone());
        }
        if key == Key::D {
            s_files_remove_all.borrow_mut().clear();
            listbox_remove.remove_all();
            //s_files_remove_all.borrow_mut().clear();
            println!("Removed all Listbox elements!");
        }
        if key == Key::P {
            println!(
                "GLOBAL PATH FROM INPUT CLOSURE {}",
                glob_path_cl.clone().borrow().display()
            );
        }
        if key == Key::R {
            let new_state = !*dirs_only_key.borrow();
            *dirs_only_key.borrow_mut() = new_state;

            println!("Directory-only mode: {}", new_state);

            s_files_remove_all.borrow_mut().clear();
            listbox_remove.remove_all();

            let current_path = if glob_path_cl.clone().borrow().exists() {
                glob_path_cl.clone().borrow().to_path_buf()
            } else {
                std::env::current_dir().unwrap()
            };

            let replace_file = list_self_dir(current_path.to_str().unwrap());
            *all_files_remove.borrow_mut() = replace_file.clone();

            if new_state {
                replace_file
                    .append_to_screen_selective(&listbox_remove, s_files_remove_all.clone());
            } else {
                replace_file.append_to_screen(&listbox_remove, s_files_remove_all.clone());
            }
        }
        if key == Key::j || key == Key::J {
            if !search_entry_key.has_focus() {
                if let Some(current_row) = listbox_remove.selected_row() {
                    let next_index = current_row.index() + 1;
                    if let Some(next_row) = listbox_remove.row_at_index(next_index) {
                        listbox_remove.select_row(Some(&next_row));
                    }
                } else if let Some(first_row) = listbox_remove.row_at_index(0) {
                    listbox_remove.select_row(Some(&first_row));
                }
            }
            return glib::Propagation::Stop;
        }
        if key == Key::k || key == Key::K {
            if !search_entry_key.has_focus() {
                if let Some(current_row) = listbox_remove.selected_row() {
                    let prev_index = current_row.index() - 1;
                    if prev_index >= 0 {
                        if let Some(prev_row) = listbox_remove.row_at_index(prev_index) {
                            listbox_remove.select_row(Some(&prev_row));
                        }
                    }
                }
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    if let Some(row) = listbox.row_at_index(0) {
        listbox.select_row(Some(&row));
    }

    let sel_files = s_files.clone();
    listbox.connect_row_selected(move |_, row| {
        let Some(row) = row else {
            return;
        };

        let mut i = 0;
        let files2 = sel_files.borrow().to_owned();

        for file in &files2 {
            if i == row.index() {
                let uri = give_me_uis_diddy(file.full_path.as_os_str().to_str().unwrap());

                display_pic.set_filename(Some(uri.as_str()));
                // eprintln!("DEBUG: PICTURE {:?}", uri);
                // should check if the path exists
                check_file(uri);
            }
            i = i + 1
        }
    });
    let remove_singular_row = listbox.clone();
    let s_files_remove = s_files.clone();
    let all_files_activated = all_files.clone();
    let path_univ = path_control.clone();
    let dirs_only_activated = dirs_only.clone();
    // clones are like this because of the gtkshell thing not because of Rust language semantics
    listbox.connect_row_activated(move |_, row| {
        eprintln!("DEBUG: Row activated at index {}", row.index());
        let mut i = 0;
        let row_index = row.index();
        let mut file_path_dir_true = ".";

        let files_for_loop = s_files_remove.borrow().to_owned();
        for file in &files_for_loop {
            if i == row.index() {
                println!("{}", file.full_path.display());
                if file.full_path.is_dir() {
                    println!("THIS IS A DIR");
                    file_path_dir_true = file.full_path.to_str().expect("Error row activate");
                    *path_univ.borrow_mut() = file.full_path.clone();
                    // do i need to clean them here?
                } else {
                    println!("not a dir {}", path_univ.clone().borrow().display());
                }

                eprintln!(
                    "DEBUG: Printed path to stdout: {}",
                    file.full_path.display()
                );
                io::stdout().flush().expect("Failed to flush stdout");
                eprintln!("DEBUG: Stdout flushed successfully");
            }
            i = i + 1
        }

        //s_files_remove.borrow_mut().remove(row_index as usize);
        s_files_remove.borrow_mut().clear();
        remove_singular_row.remove_all();
        // works
        //still leavig first indicies
        println!(
            "Path which is replacing the ccurrent {}",
            file_path_dir_true
        );
        let replace_file = list_self_dir(file_path_dir_true);
        *all_files_activated.borrow_mut() = replace_file.clone();
        if *dirs_only_activated.borrow() {
            replace_file.append_to_screen_selective(&remove_singular_row, s_files_remove.clone());
        } else {
            replace_file.append_to_screen(&remove_singular_row, s_files_remove.clone());
        }
    });

    scrolled.set_child(Some(&listbox));

    let vbox = Box::new(Orientation::Vertical, 8);

    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_hexpand(true);
    hbox.set_vexpand(true);
    hbox.set_margin_start(16);
    hbox.set_margin_end(16);
    hbox.set_margin_top(16);
    hbox.set_margin_bottom(16);

    scrolled.set_hexpand(true);
    scrolled.set_halign(gtk4::Align::Fill);
    right_pane.set_hexpand(true);
    right_pane.set_halign(gtk4::Align::Fill);

    scrolled.set_size_request(750, -1);
    right_pane.set_size_request(350, -1);

    hbox.append(&scrolled);
    hbox.append(&right_pane);

    vbox.append(&search_entry);
    vbox.append(&hbox);
    window.set_child(Some(&vbox));

    window.present();
    search_entry.grab_focus();
}

fn give_me_uis_diddy(path: &str) -> String {
    // Use GLib's proper URI encoding to handle special characters
    let canonical_path = std::fs::canonicalize(path).unwrap();
    let uri = glib::filename_to_uri(&canonical_path, None).unwrap();
    let hash = glib::compute_checksum_for_string(glib::ChecksumType::Md5, &uri)
        .unwrap_or_else(|| String::from("00000000000000000000000000000000").into());

    let check_this = format!(
        "{}/.cache/thumbnails/normal/{hash}.png",
        std::env::var("HOME").unwrap()
    );
    // i should make this a function tbh
    let final_thumbnail = if check_file(check_this.clone()) {
        check_this
    } else {
        let check_twice = format!(
            "{}/.cache/thumbnails/large/{hash}.png",
            std::env::var("HOME").unwrap()
        );

        if check_file(check_twice.clone()) {
            check_twice
        } else {
            eprintln!("WARNING: No thumbnail found for: {}", path);
            check_this
        }
    };

    final_thumbnail
}

use std::fs::read_dir;

pub fn list_self_dir(path: &str) -> Vec<FilePlus> {
    let mut udo: Vec<PathBuf> = Vec::new();
    if let Some(wtvr) = read_dir(path).ok() {
        udo = wtvr
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }
    //let mut total_tal: Vec<FilePlus> = Vec::new();

    udo.sort();
    let fp = check_dupes_comp(&udo);
    //since i just want everything from it i gueses its fine to
    // leave like this?

    // for p in udo {
    //     let mut dirs: Vec<PathBuf> = Vec::new();
    //     let mut files: Vec<PathBuf> = Vec::new();
    //     if p.is_dir() {
    //         dirs.push(p);
    //     } else {
    //         files.push(p);
    //     }

    //     for f in files {
    //         println!("files from self {}", f.display());
    //     }
    //     for d in dirs {
    //         println!("dir from self {}", d.display());
    //     }
    // }
    fp
}

pub fn dir_list_one(path: &str, extention: String, dir: bool) -> Vec<FilePlus> {
    let mut udo: Vec<PathBuf> = Vec::new();
    if let Some(entieti) = read_dir(path).ok() {
        udo = entieti
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
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

    let _unique: HashMap<String, i32> = files
        .into_keys()
        .filter_map(|p| {
            let ext = p.extension()?.to_string_lossy().into_owned();

            Some((ext, 0))
        })
        .collect();

    let fp = check_dupes_comp(&files_clone);

    let mut all_this: Vec<FilePlus> = Vec::new();

    // i could make the "exntension" an Enum so it just has like "folder" or "images"
    // "Videos+images" thats comp

    let png = "png".to_string();
    fp.iter().for_each(|full| {
        if full.extenstion.ends_with(&this_exention) || full.extenstion.ends_with(&png) {
            all_this.push(full.to_owned());
        }
    });

    if !dir {
        let more = walk_dir(dirs.clone(), this_exention.as_str());
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

    let mut fp_vec = Vec::new();
    for file in vec {
        let path = PathBuf::from(file.to_owned());

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

    return fp_vec;
}

pub fn walk_dir(dirs: HashMap<&PathBuf, i32>, ext: &str) -> Vec<FilePlus> {
    dirs.keys()
        .par_bridge()
        .flat_map(|dir| {
            dir.to_str()
                .map(|path_str| dir_list_one(path_str, ext.to_string(), false))
                .unwrap_or_default()
        })
        .collect()
}
pub fn check_file(file: String) -> bool {
    let file_as_path_buf = PathBuf::from(file);

    if file_as_path_buf.exists() {
        return true;
    } else {
        return false;
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePlus {
    pub full_path: PathBuf,
    pub extenstion: String,
}

impl FilePlus {
    fn add_to_listbox(&self, listbox: &ListBox) {
        let filename = self
            .full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| self.full_path.to_str().unwrap_or(""));

        let mut label = Label::new(Some(filename));
        if self.full_path.is_dir() {
            let heredude = filename.to_string() + "*diremoji";
            println!("Adding directory {}", heredude);
            label = Label::new(Some(heredude.as_str()));
        }

        label.set_halign(gtk4::Align::Start);
        label.set_width_request(700);
        label.set_margin_end(10);
        label.add_css_class("file-label");
        listbox.append(&label);
    }
}

trait FilePlusVecExt {
    fn append_to_screen(&self, listbox: &ListBox, storage: SharedVec<FilePlus>);
    fn append_to_screen_selective(&self, listbox: &ListBox, storage: SharedVec<FilePlus>);
    fn update_screen_with_search(
        &self,
        search_term: &str,
        listbox: &ListBox,
        storage: SharedVec<FilePlus>,
    );
}

impl FilePlusVecExt for Vec<FilePlus> {
    fn append_to_screen(&self, listbox: &ListBox, storage: SharedVec<FilePlus>) {
        let job_dude = self.clone();
        // since were just appened to the Sharedvec?
        job_dude
            .into_iter()
            .for_each(|e| storage.borrow_mut().push(e));
        let files1 = storage.borrow().to_owned();
        files1.into_iter().for_each(|f| f.add_to_listbox(listbox));
    }

    fn append_to_screen_selective(&self, listbox: &ListBox, storage: SharedVec<FilePlus>) {
        let job_dude = self.clone();
        // since were just appened to the Sharedvec?

        job_dude.into_iter().for_each(|e| {
            if e.full_path.is_dir() {
                println!("dirski");
                e.add_to_listbox(listbox);
                storage.borrow_mut().push(e)
            } else {
                println!("not");
                //
            }
        });

        //let files1 = storage.borrow().to_owned();
        //files1.into_iter().for_each(|f| f.add_to_listbox(listbox));
    }

    fn update_screen_with_search(
        &self,
        search_term: &str,
        listbox: &ListBox,
        storage: SharedVec<FilePlus>,
    ) {
        listbox.remove_all();
        storage.borrow_mut().clear();

        let search_lower = search_term.to_lowercase();

        self.iter().for_each(|file| {
            let filename = file
                .full_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if search_term.is_empty()
                || filename.contains(&search_lower)
                || file.extenstion.to_lowercase().contains(&search_lower)
            {
                storage.borrow_mut().push(file.clone());
                file.add_to_listbox(listbox);
            }
        });
    }
}
