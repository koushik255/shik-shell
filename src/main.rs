use std::time::Instant;

use gtk4::gdk::Key;
mod lib;
use gtk4::{
    Application, ApplicationWindow, EventController, EventControllerKey, Label, Picture,
    ScrolledWindow, glib,
};
use gtk4::{ListBox, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id("photo").build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(732)
        .default_height(800)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();
    let listbox = ListBox::new();
    listbox.add_css_class("boxed-list");

    let start = Instant::now();
    println!("Starting timer {:?}", start);
    println!("yeap");
    let path = "/home/koushikk/Downloads";
    // need to make sure no .var
    // i would just need to receive this information via a channel
    // from a \
    scrolled.set_child(Some(&listbox));
    window.set_child(Some(&scrolled));
    //window.present();

    let files = lib::dir_list_one(path, "mkv".to_string(), false);

    //let files1 = files.clone();
    for file in &files {
        let mut dd = file.clone();
        let fawda = dd.full_path.as_mut_os_str().to_str().expect("fail unwarp");
        let label = Label::new(Some(fawda));
        label.set_margin_start(10);
        label.set_margin_end(10);
        label.set_margin_top(10);
        label.set_margin_bottom(10);
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
    listbox.connect_row_activated(move |_, row| {
        println!("clicked");
        let mut i = 0;
        // for loop probably only thing slowing it down
        for file in &files {
            if i == row.index() {
                let wdad = file.full_path.display();
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
    // let uri = "file:///home/chris/Videos/foo.mkv";
    // let uri = format!(
    //     "file://{}",
    //     std::fs::canonicalize(
    //         "/home/koushikk/Downloads/SHOWS/Fate_Zero/MiniMTBBFateZero-049DD446ED.mkv"
    //     )
    //     .unwrap()
    //     .display()
    // );
    // let hash = format!("{:x}", md5::compute(uri));
    //
    // let path = format!(
    //     "{}/.cache/thumbnails/normal/{hash}.png",
    //     std::env::var("HOME").unwrap()
    // );
    // let path = give_me_uis_diddy(
    //     "/home/koushikk/Downloads/SHOWS/Fate_Zero/MiniMTBBFateZero-049DD446ED.mkv",
    // );
    // println!("{}", path);

    //let picture = Picture::for_filename(path);
    // let picture = Picture::for_filename(path);
    //
    // picture.set_can_shrink(true);

    /*   window.set_child(Some(&picture)); */

    // scrolled.set_child(Some(&listbox));
    // window.set_child(Some(&scrolled));

    let duration = start.elapsed();
    println!("Done it took {:?}", duration);

    window.present();
}

fn give_me_uis_diddy(path: &str) -> String {
    let uri = format!("file://{}", std::fs::canonicalize(path).unwrap().display());
    let hash = format!("{:x}", md5::compute(uri));

    let path = format!(
        "{}/.cache/thumbnails/normal/{hash}.png",
        std::env::var("HOME").unwrap()
    );

    path
}
