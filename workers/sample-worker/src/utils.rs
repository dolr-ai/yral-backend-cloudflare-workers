use cfg_if::cfg_if;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;

    } else {
        #[inline]
        #[allow(dead_code)]
        pub fn set_panic_hook() {}
    }
}
