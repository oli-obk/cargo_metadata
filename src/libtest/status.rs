use serde::Deserialize;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Test is ok (corresponds to `TestResult::TrOk` in libtest)
    Ok,
    /// Test started
    Started,
    /// Test failed (corresponds to {`TestResult::TrFailed`, `TestResult::TrTimedFail`, `TestResult::TrFailedMsg` in libtest})
    Failed,
    /// Test ignored (corresponds to `TestResult::TrIgnored` in libtest)
    Ignored,
}
