//! Create the `Error`, `ErrorKind`, `ResultExt`, and `Result` types

error_chain!{
    errors {
        /// Error during execution of `cargo metadata`
        CargoMetadata(stderr: String) {
            description("execution of `cargo metadata` failed")
            display("error during execution of `cargo metadata`: {}", stderr)
        }
    }
    foreign_links {
        Io(::std::io::Error) #[doc = "IO Error during execution of `cargo metadata`"];
        Utf8(::std::str::Utf8Error) #[doc = "Output of `cargo metadata` was not valid utf8"];
        ErrUtf8(::std::string::FromUtf8Error) #[doc = "Error output of `cargo metadata` was not valid utf8"];
        Json(::serde_json::Error) #[doc = "Deserialization error (structure of json did not match expected structure)"];
    }
}
