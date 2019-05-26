extern crate libc;

mod ll;
#[cfg(test)]
mod test;

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
#[derive(Clone, Debug, PartialEq)]
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
pub struct Wordexp {
    pub we_offs: usize,
    pub we_wordc: usize,
    // for memory deallocation
    pub wordexp_ref: Option<ll::wordexp_t>,
}

impl Wordexp {
    /// Creates an empty Wordexp struct to be used with wordexp()
    pub fn new(we_offs: usize) -> Self {
        Wordexp {
            we_offs,
            we_wordc: usize::default(),
            wordexp_ref: None,
        }
    }

    pub fn update(&mut self) {
        if let Some(wordexp_ref) = self.wordexp_ref.as_mut() {
            let we_offs: usize = wordexp_ref.we_offs as usize;
            let we_wordc: usize = wordexp_ref.we_wordc as usize;

            self.we_offs = we_offs;
            self.we_wordc = we_wordc;
        }
    }

    pub fn iter(&self) -> WordexpIterator {
        WordexpIterator::new(&self)
    }
}

pub struct WordexpIterator<'a> {
    wordexp_ref: &'a Wordexp,
    index: usize,
}

impl<'a> WordexpIterator<'a> {
    pub fn new(wordexp_ref: &'a Wordexp) -> Self {
        WordexpIterator {
            wordexp_ref,
            index: 0,
        }
    }
}

impl<'a> std::iter::Iterator for WordexpIterator<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.wordexp_ref.we_wordc + self.wordexp_ref.we_offs {
            None
        } else {
            match self.wordexp_ref.wordexp_ref.as_ref() {
                Some(s) => {
                    let item = unsafe {
                        let ptr: *const *const libc::c_char = s.we_wordv;
                        let nptr = ptr.add(self.index);
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
                    };
                    self.index += 1;
                    item
                },
                None => None,
            }
        }
    }
}

/// Should works exactly like how wordexp() works in C
pub fn wordexp(s: &str, mut p: Wordexp, flags: i32) -> Result<Wordexp, WordexpError> {
    if p.wordexp_ref.is_none() {
        let wordexp_c = ll::wordexp_t {
            we_wordc: p.we_wordc,
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
