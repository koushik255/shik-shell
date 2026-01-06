use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use gtk4::gdk::Key;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, EventControllerKey, Label, Orientation,
    Picture, ScrolledWindow, glib,
};
use gtk4::{ListBox, gdk, prelude::*};
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
    folder_path: String,
}

fn main() -> glib::ExitCode {
    let cli = Cli::parse();

    let folder_path = cli.folder_path;

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
        .default_width(1400)
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

    let files = dir_list_one(path, "mkv".to_string(), false);

    let job_dude = files.clone();
    let s_files: SharedVec<FilePlus> = Rc::new(RefCell::new(job_dude));

    let files1 = s_files.clone().borrow().to_owned();
    for file in files1 {
        let filename = file
            .full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| file.full_path.to_str().unwrap_or(""));
        let label = Label::new(Some(filename));
        label.set_halign(gtk4::Align::Start);
        label.set_width_request(700);
        label.set_margin_end(10);
        label.add_css_class("file-label");
        listbox.append(&label);
    }

    let key_ctl = EventControllerKey::new();
    window.add_controller(key_ctl.clone());

    let window2 = window.clone();
    key_ctl.connect_key_pressed(move |_, key, _code, _modifier| {
        if key == Key::Escape {
            window2.clone().close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    if let Some(row) = listbox.row_at_index(0) {
        listbox.select_row(Some(&row));
    }

    let sel_files = s_files.clone();
    listbox.connect_row_selected(move |_, row| {
        let mut i = 0;
        let mut row = row;
        if row.is_some() {
            row = Some(row.unwrap());
        }
        let files2 = sel_files.borrow().to_owned();

        for file in &files2 {
            if i == row.expect("FUCK").index() {
                let uri = give_me_uis_diddy(file.full_path.as_os_str().to_str().unwrap());

                display_pic.set_filename(Some(uri.as_str()));
                // eprintln!("DEBUG: PICTURE {:?}", uri);
                // should check if the path exists
                check_file(uri);
            }
            i = i + 1
        }
    });
    listbox.connect_row_activated(move |_, row| {
        eprintln!("DEBUG: Row activated at index {}", row.index());
        let mut i = 0;

        for file in &files {
            if i == row.index() {
                println!("{}", file.full_path.display());
                eprintln!(
                    "DEBUG: Printed path to stdout: {}",
                    file.full_path.display()
                );
                io::stdout().flush().expect("Failed to flush stdout");
                eprintln!("DEBUG: Stdout flushed successfully");
            }
            i = i + 1
        }
    });

    scrolled.set_child(Some(&listbox));

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

    window.set_child(Some(&hbox));

    window.present();
}

fn give_me_uis_diddy(path: &str) -> String {
    let uri = format!("file://{}", std::fs::canonicalize(path).unwrap().display());
    let hash = format!("{:x}", md5::compute(uri));

    format!(
        "{}/.cache/thumbnails/normal/{hash}.png",
        std::env::var("HOME").unwrap()
    )
}

use std::fs::read_dir;

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

    fp.iter().for_each(|full| {
        if full.extenstion.ends_with(&this_exention) {
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
pub fn check_file(file: String) {
    let file_as_path_buf = PathBuf::from(file);

    if file_as_path_buf.exists() {
        eprintln!(
            "
                DEBUG:PHOTO FILE EXISTS {}
            ",
            file_as_path_buf.display()
        )
    } else {
        eprintln!(
            "
                DEBUG:PHOTO DOES NOT EXIST {}
            ",
            file_as_path_buf.display()
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePlus {
    pub full_path: PathBuf,
    pub extenstion: String,
}
