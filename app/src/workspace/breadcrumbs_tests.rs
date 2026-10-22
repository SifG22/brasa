use std::path::Path;

use super::breadcrumbs;

#[test]
fn shows_home_as_tilde() {
    let home = Path::new("/home/sifg");
    assert_eq!(
        breadcrumbs(Path::new("/home/sifg/proyectos/brasa"), Some(home)),
        ("~ › proyectos › ".to_owned(), "brasa".to_owned())
    );
    assert_eq!(
        breadcrumbs(home, Some(home)),
        (String::new(), "~".to_owned())
    );
}

#[test]
fn shows_paths_outside_home_in_full() {
    assert_eq!(
        breadcrumbs(Path::new("/var/log"), Some(Path::new("/home/sifg"))),
        ("var › ".to_owned(), "log".to_owned())
    );
    assert_eq!(
        breadcrumbs(Path::new("/"), None),
        (String::new(), "/".to_owned())
    );
}

#[cfg(windows)]
#[test]
fn keeps_the_drive_on_windows() {
    assert_eq!(
        breadcrumbs(
            Path::new(r"D:\obras\brasa"),
            Some(Path::new(r"C:\Users\rodri"))
        ),
        ("D: › obras › ".to_owned(), "brasa".to_owned())
    );
}
