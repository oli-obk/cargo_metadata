//! Create the `Error`, `ErrorKind`, `ResultExt`, and `Result` types

error_chain!{
    foreign_links {
        Io(::std::io::Error) #[doc = "Error during execution of `cargo metadata`"];
        Utf8(::std::str::Utf8Error) #[doc = "Output of `cargo metadata` was not valid utf8"];
        Json(::serde_json::Error) #[doc = "Deserialization error (structure of json did not match expected structure)"];
    }
}
