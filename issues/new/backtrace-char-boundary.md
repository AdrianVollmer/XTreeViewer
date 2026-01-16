When expanding the tree, i got this error:

    byte index 40 is not a char boundary; it is inside 'ü' (bytes 39..41) of `Vertrieb: Zum Bestimmen des Hausbonus für Kunden`
    stack backtrace:
    0:     0x7f7b35e443e2 - <std::sys::backtrace::BacktraceLock::print::DisplayBacktrace as core::fmt::Display>::fmt::hc2e317dce4a0e3db
    1:     0x7f7b35e7086f - core::fmt::write::h12366295254a6fa5
    2:     0x7f7b35e1a051 - std::io::Write::write_fmt::hc42b6ced36631c09
    3:     0x7f7b35e240c2 - std::sys::backtrace::BacktraceLock::print::he883addbc7bf0a59
    4:     0x7f7b35e26e8c - std::panicking::default_hook::{{closure}}::hfec786027ccf7cdc
    5:     0x7f7b35e26ce6 - std::panicking::default_hook::h553d7071a5acc362
    6:     0x7f7b35e27515 - std::panicking::panic_with_hook::h8d62108d1ca0eec6
    7:     0x7f7b35e273aa - std::panicking::panic_handler::{{closure}}::hd9d3e436b1cebdf6
    8:     0x7f7b35e241f9 - std::sys::backtrace::__rust_end_short_backtrace::h295393bd322f0d28
    9:     0x7f7b35e0d42d - __rustc[de0091b922c53d7e]::rust_begin_unwind
    10:     0x7f7b35c71b10 - core::panicking::panic_fmt::heeb1afcf099a17a5
    11:     0x7f7b35e78710 - core::str::slice_error_fail_rt::h7b1dc541d49a4ed1
    12:     0x7f7b35c712da - core::str::slice_error_fail::h9dd166ceeb9a4231
    13:     0x7f7b35cbf844 - <core::iter::adapters::map::Map<I,F> as core::iter::traits::iterator::Iterator>::fold::hf23ac2a54ea29bc4
    14:     0x7f7b35cd662c - <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter::h0463078981c1b2ac
    15:     0x7f7b35cc12e0 - xtv::ui::tree_view::TreeView::render::h2d779dc1e0e450c1
    16:     0x7f7b35c7bcba - xtv::ui::app::App::render::he71ee8dfa6fd2f9c
    17:     0x7f7b35cc498b - ratatui::terminal::terminal::Terminal<B>::draw::hebf9d45ec31e9264
    18:     0x7f7b35c79cd5 - xtv::ui::app::App::run::h614177e1f2124a3e
    19:     0x7f7b35c7575d - xtv::main::h7b0e36a0603e888f
    20:     0x7f7b35c73f33 - std::sys::backtrace::__rust_begin_short_backtrace::hfe700bff4d0c4184
    21:     0x7f7b35c73f29 - std::rt::lang_start::{{closure}}::h24aaaf7c11277403
    22:     0x7f7b35e1b81a - std::rt::lang_start_internal::hb340d7a1bb586dd2
    23:     0x7f7b35c765a5 - main
