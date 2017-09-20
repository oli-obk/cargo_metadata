//! Create the `Error`, `ErrorKind`, `ResultExt`, and `Result` types
#![allow(missing_docs)] // see https://github.com/rust-lang-nursery/error-chain/issues/63

error_chain!{
    foreign_links {
        // Error during execution of `cargo metadata`
        Io(::std::io::Error);
        // Output of `cargo metadata` was not valid utf8
        Utf8(::std::str::Utf8Error);
        // Deserialization error (structure of json did not match expected structure)
        Json(::serde_json::Error);
    }
}
