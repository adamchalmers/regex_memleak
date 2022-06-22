#![feature(bench_black_box)]
use regex::bytes::{RegexSet, RegexSetBuilder};
use std::hint::black_box;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
static SIGNATURES: &[((u32, u32), &str)] = &[(
    (5, 5012),
    r"^\x00\x01\x00\x00\x53\x74\x61\x6e\x64\x61\x72\x64\x20\x4a\x65\x74\x20\x44\x42",
)];

lazy_static::lazy_static! {
    static ref SIGNATURE_REGEX_SET: RegexSet = build_regex_set(SIGNATURES);
}

fn build_regex_set(src: &[((u32, u32), &str)]) -> RegexSet {
    black_box(
        RegexSetBuilder::new(src.iter().map(|t| t.1))
            .dot_matches_new_line(true)
            .unicode(false)
            .build()
            .expect("file classifier's patterns should be correct"),
    )
}

#[test]
fn test_leak_in_regexset() {
    test_memory_leak(|| {
        let out = black_box(classify_with(SIGNATURES, &SIGNATURE_REGEX_SET, b"foobar"));
        println!("{out:?}");
    })
}

/// Assert that calling `f` doesn't leak any memory.
#[cfg(test)]
fn test_memory_leak(f: impl Fn() -> ()) {
    let _profiler = dhat::Profiler::builder().testing().build();

    f(); // Run the function once, to initialize static vars.

    let stats_before = dhat::HeapStats::get().curr_bytes;
    f();
    let stats_after = dhat::HeapStats::get().curr_bytes;

    let delta = stats_before - stats_after;
    dhat::assert_eq!(delta, 0);
}

#[cfg(test)]
#[inline]
fn classify_with(
    type_db: &[((u32, u32), &str)],
    set: &RegexSet,
    field: &[u8],
) -> Option<(u32, u32)> {
    black_box(set)
        .matches(field)
        .into_iter()
        .next()
        .map(|i| black_box(type_db[i].0))
}
