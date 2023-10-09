use serde::{
    de::{Deserializer, Error, MapAccess, Unexpected, Visitor},
    Deserialize,
};

use super::{r#type::Type, status::Status};

#[derive(Debug, PartialEq)]
/// Represents the output of `cargo test -- -Zunstable-options --report-time --show-output`.
///
/// requires --report-time and --show-output
pub enum TestEvent {
    /// emitted on the start of a test run, and the start of the doctests
    SuiteStart {
        /// number of tests in this suite
        test_count: usize,
    },
    /// the suite has finished
    SuiteOk {
        /// the number of tests that passed
        passed: usize,
        /// the number of tests that failed
        failed: usize,
        /// number of tests that were ignored
        ignored: usize,
        /// i think its something to do with benchmarks?
        measured: usize,
        /// i think this is based on what you specify in the cargo test argument
        filtered_out: usize,
        /// how long the suite took to run
        exec_time: f32,
    },
    /// the suite has at least one failing test
    SuiteFail {
        /// the number of tests that passed
        passed: usize,
        /// the number of tests that failed
        failed: usize,
        /// number of tests that were ignored
        ignored: usize,
        /// i think its something to do with benchmarks?
        measured: usize,
        /// i think this is based on what you specify in the cargo test argument
        filtered_out: usize,
        /// how long the suite took to run
        exec_time: f32,
    },
    /// a new test starts
    TestStart {
        /// the name of this test
        name: String,
    },
    /// the test has finished
    TestOk {
        /// which one
        name: String,
        /// in how long
        exec_time: f32,
        /// what did it say?
        stdout: Option<String>,
    },
    /// the test has failed
    TestFail {
        /// which one
        name: String,
        /// in how long
        exec_time: f32,
        /// why?
        stdout: Option<String>,
    },
    /// the test has timed out
    TestTimeout {
        /// which one
        name: String,
        /// how long did it run
        exec_time: f32,
        /// did it say anything
        stdout: Option<String>,
    },
    /// the test has failed, with a message, i am not sure how this works.
    TestFailMessage {
        /// which one
        name: String,
        /// in how long
        exec_time: f32,
        /// stdout?
        stdout: Option<String>,
        /// what message
        message: String,
    },
    /// the test has been ignored
    TestIgnore {
        /// which one
        name: String,
    },
}

impl<'de> Deserialize<'de> for TestEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_map(Visit);
        struct Visit;
        // look ma, no intermediary data structures
        impl<'de> Visitor<'de> for Visit {
            type Value = TestEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a test result")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                macro_rules! take {
                    ($name:ident as $what:ty) => {{
                        let Some((key, value)) = map.next_entry::<&str, $what>()? else {
                            return Err(Error::missing_field(stringify!($name)));
                        };
                        if key != stringify!($name) {
                            return Err(Error::missing_field(stringify!($name)));
                        }
                        value
                    }};
                    ($str:ident { $($name:ident: $what:ty),+ $(,)? }) => {{
                        TestEvent::$str {
                            $($name: take!($name as $what),)+
                        }
                    }}
                }
                let kind = take!(type as Type);
                let mut statuz = None;
                let mut name = None;
                match map
                    .next_key::<&str>()?
                    .ok_or(Error::missing_field("event | name"))?
                {
                    "event" => statuz = Some(map.next_value::<Status>()?),
                    "name" => name = Some(map.next_value::<String>()?),
                    f => return Err(Error::unknown_field(f, &["event", "name"])),
                }

                let status = if let Some(status) = statuz {
                    status
                } else {
                    take!(event as Status)
                };
                Ok(match (kind, status) {
                    (Type::Suite, Status::Started) => TestEvent::SuiteStart {
                        test_count: take!(test_count as usize),
                    },
                    (Type::Suite, Status::Ok) => take!(SuiteOk {
                        passed: usize,
                        failed: usize,
                        ignored: usize,
                        measured: usize,
                        filtered_out: usize,
                        exec_time: f32
                    }),
                    (Type::Suite, Status::Failed) => take!(SuiteFail {
                        passed: usize,
                        failed: usize,
                        ignored: usize,
                        measured: usize,
                        filtered_out: usize,
                        exec_time: f32,
                    }),
                    (Type::Suite, Status::Ignored) => {
                        return Err(Error::custom("test suite's cannot be ignored"));
                    }
                    (Type::Test, Status::Started) => take!(TestStart { name: String }),
                    (Type::Test, Status::Ok) => TestEvent::TestOk {
                        name: name.ok_or(Error::missing_field("name"))?,
                        exec_time: take!(exec_time as f32),
                        stdout: match map.next_key::<&str>()? {
                            Some("stdout") => Some(map.next_value::<String>()?),
                            Some(k) => return Err(Error::unknown_field(k, &["stdout"])),
                            None => None,
                        },
                    },
                    (Type::Test, Status::Ignored) => TestEvent::TestIgnore {
                        name: name.ok_or(Error::missing_field("name"))?,
                    },
                    (Type::Test, Status::Failed) => {
                        let exec_time = take!(exec_time as f32);
                        let name = name.ok_or(Error::missing_field("name"))?;
                        let stdout = match map.next_key::<&str>()? {
                            Some("stdout") => Some(map.next_value::<String>()?),
                            Some(k) => return Err(Error::unknown_field(k, &["stdout"])),
                            None => None,
                        };
                        match map.next_key::<&str>()? {
                            Some("reason") => {
                                let reason = map.next_value::<&str>()?;
                                if reason != "time limit exceeded" {
                                    return Err(Error::invalid_value(
                                        Unexpected::Str(reason),
                                        &"time limit exceeded",
                                    ));
                                }
                                TestEvent::TestTimeout {
                                    name,
                                    exec_time,
                                    stdout,
                                }
                            }
                            Some("message") => {
                                let message = map.next_value::<String>()?;
                                TestEvent::TestFailMessage {
                                    name,
                                    exec_time,
                                    stdout,
                                    message,
                                }
                            }
                            _ => TestEvent::TestFail {
                                name,
                                exec_time,
                                stdout,
                            },
                        }
                    }
                    (Type::Bench, _) => {
                        todo!()
                    }
                })
            }
        }
    }
}

#[test]
fn deser() {
    macro_rules! run {
        ($($input:literal parses to $output:expr),+) => {
            $(assert_eq!(dbg!(serde_json::from_str::<TestEvent>($input)).unwrap(), $output);)+
        };
    }
    run![
        r#"{ "type": "suite", "event": "started", "test_count": 2 }"# parses to TestEvent::SuiteStart { test_count: 2 },
        r#"{ "type": "test", "event": "started", "name": "fail" }"# parses to TestEvent::TestStart { name: "fail".into() },
        r#"{ "type": "test", "name": "fail", "event": "ok", "exec_time": 0.000003428, "stdout": "hello world" }"# parses to TestEvent::TestOk { name: "fail".into(), exec_time: 0.000003428, stdout: Some("hello world".into()) } ,
        r#"{ "type": "test", "event": "started", "name": "nope" }"# parses to TestEvent::TestStart { name: "nope".into() },
        r#"{ "type": "test", "name": "nope", "event": "ignored" }"# parses to TestEvent::TestIgnore { name: "nope".into() },
        r#"{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "ignored": 1, "measured": 0, "filtered_out": 0, "exec_time": 0.000684028 }"# parses to TestEvent::SuiteOk { passed: 1, failed: 0, ignored: 1, measured: 0, filtered_out: 0, exec_time: 0.000684028 }
    ];

    run![
        r#"{ "type": "suite", "event": "started", "test_count": 1 }"# parses to TestEvent::SuiteStart { test_count: 1 },
        r#"{ "type": "test", "event": "started", "name": "fail" }"# parses to TestEvent::TestStart { name: "fail".into() },
        r#"{ "type": "test", "name": "fail", "event": "failed", "exec_time": 0.000081092, "stdout": "thread 'fail' panicked" }"# parses to TestEvent::TestFail { name: "fail".into(), exec_time: 0.000081092, stdout: Some("thread 'fail' panicked".into()) },
        r#"{ "type": "suite", "event": "failed", "passed": 0, "failed": 1, "ignored": 0, "measured": 0, "filtered_out": 0, "exec_time": 0.000731068 }"# parses to TestEvent::SuiteFail { passed: 0, failed: 1, ignored: 0, measured: 0, filtered_out: 0, exec_time: 0.000731068 }
    ];
}
