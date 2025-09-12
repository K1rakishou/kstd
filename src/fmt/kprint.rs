use crate::alloc::{kallocator::KAllocator, kstring::KString};
use std::io::{stdout, Write};

#[allow(dead_code)]
pub fn kprint_string_fn<'a, A: KAllocator>(s: KString<'a, A>, with_new_line: bool) {
    let mut lock = stdout().lock();

    if with_new_line {
        writeln!(lock, "{}", s.as_str()).unwrap();
    } else {
        write!(lock, "{}", s.as_str()).unwrap();
    }
}

#[allow(dead_code)]
pub fn kprint_str_fn<'a>(s: &str) {
    let mut lock = stdout().lock();
    write!(lock, "{}", s).unwrap();
}

#[macro_export]
macro_rules! kprint {
    () => {
        $crate::fmt::kprint::kprint_str_fn("\n")
    };
    ($allocator:expr, $($arg:tt)*) => {{
        $crate::fmt::kprint::kprint_string_fn($crate::kformat!($allocator, $($arg)*), false);
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::fmt::kprint::kprint_str_fn("\n")
    };
    ($allocator:expr, $($arg:tt)*) => {{
        $crate::fmt::kprint::kprint_string_fn($crate::kformatln!($allocator, $($arg)*), true);
    }};
}

#[allow(unused_imports)]
mod test {
    use crate::alloc::global::GlobalAllocator;

    #[test]
    fn test_kprint() {
        let allocator = GlobalAllocator::new();
        kprint!(&allocator, "Hello, {}!", "World");
        kprintln!();
    }

    #[test]
    fn test_kprintln() {
        let allocator = GlobalAllocator::new();
        kprintln!(&allocator, "Hello, {}!", "World");
    }
}
