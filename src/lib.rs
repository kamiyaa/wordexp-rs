extern crate libc;

mod ll;

use std::ffi::{CStr, CString};

/// Append the words found to the array resulting from a previous call.
pub const WRDE_DOOFFS: i32 = 1;
/// Insert we_offs initial Nones in the array we_wordv. (These are not counted in the returned we_wordc.)
pub const WRDE_APPEND: i32 = (1 << 1);
/// Don't do command substitution.
pub const WRDE_NOCMD: i32 = (1 << 2);
/// The argument p resulted from a previous call to wordexp(), and wordfree() was not called. Reuse the allocated storage.
pub const WRDE_REUSE: i32 = (1 << 3);
/// Normally during command substitution stderr is redirected to /dev/null. This flag specifies that stderr is not to be redirected.
pub const WRDE_SHOWERR: i32 = (1 << 4);
/// Consider it an error if an undefined shell variable is expanded.
pub const WRDE_UNDEF: i32 = (1 << 5);

/// Out of memory.
pub const WRDE_NOSPACE: i32 = 1;
/// Illegal occurrence of newline or one of |, &, ;, <, >, (, ), {, }.
pub const WRDE_BADCHAR: i32 = 2;
/// An undefined shell variable was referenced, and the WRDE_UNDEF flag told us to consider this an error.
pub const WRDE_BADVAL: i32 = 3;
/// Command substitution occurred, and the WRDE_NOCMD flag told us to consider this an error.
pub const WRDE_CMDSUB: i32 = 4;
/// Shell syntax error, such as unbalanced parentheses or unmatched quotes.
pub const WRDE_SYNTAX: i32 = 5;

trait ToCStr {
    fn to_c_str(&self) -> CString;
}

impl<'a> ToCStr for &'a str {
    fn to_c_str(&self) -> CString {
        CString::new(*self).unwrap()
    }
}

/// Errors types for WordexpError
#[derive(Clone, Debug)]
pub enum WordexpErrorType {
    /// Illegal occurrence of newline or one of |, &, ;, <, >, (, ), {, }.
    BadChar,
    /// An undefined shell variable was referenced, and the WRDE_UNDEF flag told us to consider this an error.
    BadVal,
    /// Command substitution occurred, and the WRDE_NOCMD flag told us to consider this an error.
    CmdSub,
    /// Out of memory.
    NoSpace,
    /// Shell syntax error, such as unbalanced parentheses or unmatched quotes.
    Syntax,
    /// Unknown Error, most likely caused by wrapper code between Rust and C
    Unknown,
}

impl WordexpErrorType {
    /// converts a C wordexp error code to a Rust enum
    pub fn from(error_code: i32) -> Self {
        match error_code {
            WRDE_BADCHAR => WordexpErrorType::BadChar,
            WRDE_BADVAL => WordexpErrorType::BadVal,
            WRDE_CMDSUB => WordexpErrorType::CmdSub,
            WRDE_NOSPACE => WordexpErrorType::NoSpace,
            WRDE_SYNTAX => WordexpErrorType::Syntax,
            _ => WordexpErrorType::Unknown,
        }
    }
}

/// Errors returned from wordexp()
#[derive(Clone, Debug)]
pub struct WordexpError {
    pub error_type: WordexpErrorType,
}

impl WordexpError {
    pub fn new(error_type: WordexpErrorType) -> Self {
        WordexpError { error_type }
    }
}

impl std::fmt::Display for WordexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.error_type)
    }
}

impl std::error::Error for WordexpError {}

/// Wrapper for C struct: wordexp_t
pub struct Wordexp<'a> {
    pub we_offs: usize,
    pub we_wordv: Vec<Option<&'a str>>,
    // for memory deallocation
    pub wordexp_ref: Option<ll::wordexp_t>,
    // for iterator
    counter: usize,
}

impl<'a> Wordexp<'a> {
    /// Creates an empty Wordexp struct to be used with wordexp()
    pub fn new(we_offs: usize) -> Self {
        Wordexp {
            we_offs,
            we_wordv: Vec::default(),
            wordexp_ref: None,
            counter: usize::default(),
        }
    }

    pub fn update(&mut self) {
        if let Some(wordexp_ref) = self.wordexp_ref.as_mut() {
            let we_wordc: usize = wordexp_ref.we_wordc as usize;
            let we_offs: usize = wordexp_ref.we_offs as usize;
            let we_wordv: Vec<Option<&str>> = unsafe {
                let ptr: *const *const libc::c_char = wordexp_ref.we_wordv;

                (0..we_wordc)
                    .map(|i| {
                        let nptr = ptr.add(i);
                        if nptr == std::ptr::null() {
                            None
                        } else {
                            let cstr = CStr::from_ptr(*nptr);
                            if let Ok(s) = cstr.to_str() {
                                Some(s)
                            } else {
                                None
                            }
                        }
                    })
                    .collect()
            };
            self.we_wordv = we_wordv;
            self.we_offs = we_offs;
        }
    }
}

impl<'a> std::iter::Iterator for Wordexp<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= self.we_wordv.len() {
            self.counter = 0;
            None
        } else {
            let item = self.we_wordv[self.counter];
            self.counter += 1;
            item
        }
    }
}

/// Should works exactly like how wordexp() works in C
pub fn wordexp<'a>(s: &str, mut p: Wordexp<'a>, flags: i32) -> Result<Wordexp<'a>, WordexpError> {
    if p.wordexp_ref.is_none() {
        let wordexp_c = ll::wordexp_t {
            we_wordc: 0,
            we_wordv: std::ptr::null(),
            we_offs: p.we_offs,
        };
        p.wordexp_ref = Some(wordexp_c);
    }

    let result = unsafe {
        let result = ll::wordexp(
            s.to_c_str().as_ptr(),
            p.wordexp_ref.as_mut().unwrap(),
            flags,
        );
        if result == 0 {
            p.update();
            Ok(Wordexp::from(p))
        } else {
            let err_type = WordexpErrorType::from(result);
            Err(WordexpError::new(err_type))
        }
    };
    result
}

#[cfg(test)]
mod tests {
    use super::{wordexp, Wordexp};
    #[test]
    fn it_works() {
        std::env::set_var("HOME", "/home/wordexp");
        let s = "~/";
        let p = Wordexp::new(0);
        let flags = 0;

        match wordexp(s, p, flags) {
            Ok(s) => {
                assert_eq!(0, s.we_offs);
                assert_eq!(1, s.we_wordv.len());
                match s.we_wordv[0] {
                    None => assert!(false),
                    Some(s) => assert_eq!("/home/wordexp/", s),
                }
            }
            Err(_) => assert!(false),
        }
    }
}
