#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("data/icon/pich8.ico");
    res.compile().expect("compiling windows resource failed");
}

#[cfg(not(windows))]
fn main() {}