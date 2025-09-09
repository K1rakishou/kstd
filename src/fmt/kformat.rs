use crate::alloc::{kallocator::KAllocator, kstring::KString};

pub fn try_kformat_fn<'a, A: KAllocator>(allocator: &'a A, args: core::fmt::Arguments<'_>) -> Result<KString<'a, A>, core::fmt::Error> {
    let mut out = KString::new(allocator);

    if let Some(s) = args.as_str() {
        out.push_str(s);
        return Ok(out);
    }

    core::fmt::write(&mut out, args)?;
    return Ok(out);
}

pub fn kformat_fn<'a, A: KAllocator>(allocator: &'a A, args: core::fmt::Arguments<'_>) -> KString<'a, A> {
    return try_kformat_fn(allocator, args).unwrap();
}

#[macro_export]
macro_rules! kformat {
    ($alloc:expr, $($arg:tt)*) => {{
        $crate::fmt::kformat::kformat_fn($alloc, core::format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! try_kformat {
    ($alloc:expr, $($arg:tt)*) => {{
        $crate::fmt::kformat::try_kformat_fn($alloc, core::format_args!($($arg)*))
    }};
}

mod test {
    use std::fmt::Write;

    use crate::alloc::global::GlobalAllocator;

    #[test]
    fn test_kformat() {
        let allocator = GlobalAllocator::new();
        let result_string = kformat!(&allocator, "Hello, {}! Pi: {}", "World", 3.1415);
        assert_eq!("Hello, World! Pi: 3.1415", result_string.as_str());
    }

    #[test]
    fn test_try_kformat() {
        let allocator = GlobalAllocator::new();
        let result_string = try_kformat!(&allocator, "Hello, {}", "World!");
        assert!(result_string.is_ok());
        assert_eq!("Hello, World!", result_string.unwrap().as_str());
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
    
    #[test]
    fn test_try_kformat_display_panics() {
        struct Test {}

        impl core::fmt::Display for Test {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                return Err(core::fmt::Error)
            }
        }
        
        let allocator = GlobalAllocator::new();
        let result = try_kformat!(&allocator, "Hello, {}", Test{});
        assert!(result.is_err());
    }

}
