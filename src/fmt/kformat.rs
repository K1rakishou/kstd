use crate::alloc::{kallocator::KAllocator, kstring::KString};

pub fn kformatln_args_fn<'allocator, A: KAllocator>(allocator: &'allocator A, add_new_line: bool, args: core::fmt::Arguments<'_>) -> KString<'allocator, A> {
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

#[allow(dead_code)]
pub fn kformat_fn<'allocator, A: KAllocator>(allocator: &'allocator A, args: core::fmt::Arguments<'_>) -> KString<'allocator, A> {
    return kformatln_args_fn(allocator, false, args);
}

#[allow(dead_code)]
pub fn kformatln_fn<'allocator, A: KAllocator>(allocator: &'allocator A, args: core::fmt::Arguments<'_>) -> KString<'allocator, A> {
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

#[allow(unused_imports)]
mod test {
    use crate::alloc::kglobal_allocator::KGlobalAllocator;

    #[test]
    fn test_kformat() {
        let allocator = KGlobalAllocator::new();
        let result_string = kformat!(&allocator, "Hello, {}! Pi: {}", "World", 3.1415);
        assert_eq!("Hello, World! Pi: 3.1415", result_string.as_str());
    }

    #[test]
    #[should_panic]
    fn test_kformat_display_panics() {
        struct Test {}

        impl core::fmt::Display for Test {
            fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                return Err(core::fmt::Error)
            }
        }
        
        let allocator = KGlobalAllocator::new();
        let _ = kformat!(&allocator, "Hello, {}", Test{});
    }
    
}
