use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use gtk4::gdk::Key;
use gtk4::{
    Application, ApplicationWindow, Box, EventControllerKey, Label, Orientation, Paned, Picture,
    ScrolledWindow, glib,
};
use gtk4::{ListBox, prelude::*};
use gtk4_layer_shell::{Layer, LayerShell};
use rayon::prelude::*;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id("photo").build();

    app.connect_activate(build_ui);
    app.run()
}

type SharedVec<T> = Rc<RefCell<Vec<T>>>;

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(800)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();
    let listbox = ListBox::new();

    listbox.add_css_class("boxed-list");

    let right_pane = Box::new(Orientation::Vertical, 6);
    right_pane.set_margin_top(12);
    right_pane.set_margin_end(12);
    let display_pic = Picture::new();
    right_pane.append(&display_pic);

    let start = Instant::now();
    println!("Starting timer {:?}", start);
    println!("yeap");
    let path = "/home/koushikk/Downloads/SHOWS/Friren";

    let mut files: Vec<FilePlus> = Vec::new();

    {
        files = dir_list_one(path, "mkv".to_string(), false);
    }
    let duration_til_now = start.elapsed();
    println!("Duration after dir_list_one {:?}", duration_til_now);

    let job_dude = files.clone();
    let s_files: SharedVec<FilePlus> = Rc::new(RefCell::new(job_dude));

    let files1 = s_files.clone().borrow().to_owned();
    for file in files1 {
        let mut dd = file.clone();
        let fawda = dd.full_path.as_mut_os_str().to_str().expect("fail unwarp");
        let label = Label::new(Some(fawda));
        label.set_halign(gtk4::Align::Start);
        label.set_width_request(750);
        label.set_margin_end(10);
        listbox.append(&label);
    }

    let key_ctl = EventControllerKey::new();
    window.add_controller(key_ctl.clone());

    let window2 = window.clone();
    key_ctl.connect_key_pressed(move |_, key, _code, _modifier| {
        println!("pressed : {key:?}");
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
        } else {
            println!("nah bruh");
        }
        let files2 = sel_files.borrow().to_owned();

        for file in &files2 {
            if i == row.expect("FUCK").index() {
                let wdad = file.full_path.display();
                let uri = give_me_uis_diddy(file.full_path.as_os_str().to_str().unwrap());

                display_pic.set_filename(Some(uri.as_str()));

                println!("{}", wdad);
            }
            i = i + 1
        }
    });
    listbox.connect_row_activated(move |_, row| {
        println!("clicked");
        let mut i = 0;

        for file in &files {
            if i == row.index() {
                let wdad = file.full_path.display();

                println!("{}", wdad);
            }
            i = i + 1
        }
        println!("Selected row {}", row.index());
    });

    let duration = start.elapsed();
    println!("Done it took {:?}", duration);

    scrolled.set_child(Some(&listbox));

    let hbox = Box::new(Orientation::Horizontal, 0);
    hbox.set_hexpand(true);
    hbox.set_vexpand(true);

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

    let path = format!(
        "{}/.cache/thumbnails/normal/{hash}.png",
        std::env::var("HOME").unwrap()
    );
    println!("{}", &path);

    path
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

    let unique: HashMap<String, i32> = files
        .into_keys()
        .filter_map(|p| {
            let ext = p.extension()?.to_string_lossy().into_owned();

            Some((ext, 0))
        })
        .collect();

    let mut fp: Vec<FilePlus> = Vec::new();
    {
        fp = check_dupes_comp(&files_clone);
    }

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilePlus {
    pub full_path: PathBuf,
    pub extenstion: String,
}
