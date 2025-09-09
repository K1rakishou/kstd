use crate::alloc::{kallocator::KAllocator, kstring::KString};

pub fn kformatln_args_fn<'a, A: KAllocator>(allocator: &'a A, add_new_line: bool, args: core::fmt::Arguments<'_>) -> KString<'a, A> {
    // TODO: use estimated_capacity once it's stable
    // let cap = args.estimated_capacity();
    
    let mut out = KString::new(allocator);

    match args.as_str() {
        Some(s) => out.push_str(s),
        None => core::fmt::write(&mut out, args).unwrap(),
    }

    if add_new_line {
        out.push_char('\n');
    }
    
    return out;
}

pub fn kformat_fn<'a, A: KAllocator>(allocator: &'a A, args: core::fmt::Arguments<'_>) -> KString<'a, A> {
    return kformatln_args_fn(allocator, false, args);
}

pub fn kformatln_fn<'a, A: KAllocator>(allocator: &'a A, args: core::fmt::Arguments<'_>) -> KString<'a, A> {
    return kformatln_args_fn(allocator, true, args);
}

#[macro_export]
macro_rules! kformat {
    ($allocator:expr, $($arg:tt)*) => {{
        $crate::fmt::kformat::kformat_fn($allocator, core::format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! kformatln {
    ($allocator:expr, $($arg:tt)*) => {{
        $crate::fmt::kformat::kformatln_fn($allocator, core::format_args!($($arg)*))
    }};
}


mod test {
    use crate::alloc::global::GlobalAllocator;

    #[test]
    fn test_kformat() {
        let allocator = GlobalAllocator::new();
        let result_string = kformat!(&allocator, "Hello, {}! Pi: {}", "World", 3.1415);
        assert_eq!("Hello, World! Pi: 3.1415", result_string.as_str());
    }

    #[test]
    #[should_panic]
    fn test_kformat_display_panics() {
        struct Test {}

        impl core::fmt::Display for Test {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                return Err(core::fmt::Error)
            }
        }
        
        let allocator = GlobalAllocator::new();
        let _ = kformat!(&allocator, "Hello, {}", Test{});
    }
    
}
