use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Test,
    /// Occurs usually 4 times in a `cargo test` lifetime:
    /// - once at the start, how many normal tests
    /// - once at the end of the normal tests, how did it go
    /// - once at the start of the doc tests
    /// - once at the end of the doc tests
    Suite,
    /// `#[bench]` type tests.
    Bench,
}
