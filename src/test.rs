use super::{wordexp, Wordexp, WordexpErrorType};

#[test]
fn no_changes_001() {
    let s = "hello";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            if let Some(s) = w_iter.next() {
                assert_eq!("hello", s);
            } else {
                assert!(false);
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn home_substitution_001() {
    std::env::set_var("HOME", "/home/wordexp");
    let s = "~";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("/home/wordexp", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn home_substitution_002() {
    std::env::set_var("HOME", "/home/wordexp");
    let s = "~/";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("/home/wordexp/", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn variable_substitution_001() {
    std::env::set_var("HOME", "/home/wordexp");
    let s = "$HOME";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("/home/wordexp", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn variable_substitution_002() {
    std::env::set_var("HOME", "/home/wordexp");
    let s = "${HOME}documents";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("/home/wordexpdocuments", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn variable_substitution_003() {
    let s = "${KDLAMDLADJKFKFNDSJKFKDSN}";
    let p = Wordexp::new(0);
    let flags = super::WRDE_UNDEF;

    match wordexp(s, p, flags) {
        Ok(_) => assert!(false),
        Err(e) => assert_eq!(e.error_type, WordexpErrorType::BadVal),
    }
}

#[test]
fn command_substitution_001() {
    let s = "`echo hello`";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("hello", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn command_substitution_002() {
    let s = "$(echo hello)";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("hello", s),
                None => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

#[test]
fn command_substitution_003() {
    let s = "$(echo hello)";
    let p = Wordexp::new(0);
    let flags = super::WRDE_NOCMD;

    match wordexp(s, p, flags) {
        Ok(s) => {
            assert_eq!(0, s.we_offs);
            assert_eq!(1, s.we_wordc);
            let mut w_iter = s.iter();
            match w_iter.next() {
                Some(s) => assert_eq!("$(echo hello)", s),
                None => assert!(false),
            }
        }
        Err(e) => assert_eq!(e.error_type, WordexpErrorType::CmdSub),
    }
}

#[test]
fn bad_char_001() {
    let s = "||||";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(_) => assert!(false),
        Err(e) => assert_eq!(e.error_type, WordexpErrorType::BadChar),
    }
}

#[test]
fn bad_char_002() {
    let s = "cat file.txt | grep hello";
    let p = Wordexp::new(0);
    let flags = 0;

    match wordexp(s, p, flags) {
        Ok(_) => assert!(false),
        Err(e) => assert_eq!(e.error_type, WordexpErrorType::BadChar),
    }
}
