use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use gtk4::gdk::Key;
mod lib;
use gtk4::{
    Application, ApplicationWindow, Box, EventControllerKey, Label, Orientation, Paned, Picture,
    ScrolledWindow, glib,
};
use gtk4::{ListBox, prelude::*};
use gtk4_layer_shell::{Layer, LayerShell};

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
    let path = "/home/koushikk/Downloads";
    // need to make sure no .var
    // i would just need to receive this information via a channel
    // from a \

    //window.present();

    let files = lib::dir_list_one(path, "mkv".to_string(), false);
    let job_dude = files.clone();
    let s_files: SharedVec<lib::FilePlus> = Rc::new(RefCell::new(job_dude));

    //let files1 = files.clone();
    let files1 = s_files.clone().borrow().to_owned();
    for file in files1 {
        let mut dd = file.clone();
        let fawda = dd.full_path.as_mut_os_str().to_str().expect("fail unwarp");
        let label = Label::new(Some(fawda));
        //label.set_margin_start(0);
        label.set_halign(gtk4::Align::Start);
        label.set_width_request(750);
        label.set_margin_end(10);
        //label.set_margin_top(10);
        // label.set_margin_bottom(10);
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

    //plan
    //what folder are we running in
    //have default folder and one you can start from term
    //:
    //now just probe to get the metadata image?

    if let Some(row) = listbox.row_at_index(0) {
        listbox.select_row(Some(&row));
    }

    //let files2 = files.clone();
    // let counter = Rc::new(RefCell::new(Picture::default()));

    //let p = counter.clone();
    let sel_files = s_files.clone();
    listbox.connect_row_selected(move |_, row| {
        let mut i = 0;
        let mut row = row;
        if row.is_some() {
            row = Some(row.unwrap());
        } else {
            println!("nah bruh");
        }
        // for loop probably only thing slowing it down
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
        // for loop probably only thing slowing it down
        for file in &files {
            if i == row.index() {
                let wdad = file.full_path.display();
                // let uri = give_me_uis_diddy(file.full_path.as_os_str().to_str().unwrap());
                //
                // display_pic.set_filename(Some(uri.as_str()));
                //
                println!("{}", wdad);
            }
            i = i + 1
        }
        // if i want it updateing live i would probably need to use channels,
        // i could just feed each row into the channel and receive it on the main thread so that
        // the ui would instantly boot up
        println!("Selected row {}", row.index());
    });

    // Optional: Set anchors (remove these to make it freely movable)
    // window.set_anchor(Edge::Top, true);
    // window.set_anchor(Edge::Right, true);
    //
    // window.set_margin(Edge::Top, 20);
    // window.set_margin(Edge::Right, 20);
    // println!("{}", path);

    // let picture = Picture::for_filename(path);
    //
    // picture.set_can_shrink(true);

    // let p2 = counter.borrow().to_owned();
    // right_pane.append(&p2);

    // scrolled.set_child(Some(&listbox));
    // window.set_child(Some(&scrolled));

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
