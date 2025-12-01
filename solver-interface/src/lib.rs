use std::borrow::Cow;
use std::time::{Duration, Instant};

use bincode::config::Configuration;
use bincode::error::{DecodeError, EncodeError};
use bincode::{Decode, Encode};
use thiserror::Error;

mod parent;
pub use parent::ParentSolver;

mod child;
pub use child::{ChildSolver, ChildSolverExt};

fn bincode_config() -> Configuration {
    bincode::config::standard()
}

/// Time a single function.
pub fn time_fn<F: FnOnce() -> T, T>(f: F) -> (Duration, T) {
    let start = Instant::now();
    let t = f();
    let end = start.elapsed();
    (end, t)
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ParentToChild<'a> {
    Initialize(Initialization<'a>),
    Run(Run),
    Bench(Bench),
    End,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Initialization<'a> {
    pub input: Cow<'a, [u8]>,
    pub debug: u8,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Run {
    pub part: u32,
}

impl Run {
    fn time_solver<'a, S>(self, input: &[u8], buf: &'a mut String, debug: u8) -> (Duration, &'a str)
    where
        S: ChildSolver + ?Sized,
    {
        use std::fmt::Write;
        buf.clear();

        let d = match self.part {
            1 => {
                let (d, ans) = time_fn(|| S::part_one(input, debug));
                write!(buf, "{ans}").unwrap();
                d
            }
            2 => {
                let (d, ans) = time_fn(|| S::part_two(input, debug));
                write!(buf, "{ans}").unwrap();
                d
            }
            part => {
                let (d, ans) = time_fn(|| S::run_any(input, part, debug));
                write!(buf, "{ans}").unwrap();
                d
            }
        };

        (d, buf.as_str())
    }
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Bench {
    pub run: Run,
    pub iters: u32,
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ChildToParent<'a> {
    Answer(Answer<'a>),
    BenchResult(BenchResult<'a>),
    Err(Box<str>),
}

impl ChildToParent<'_> {
    pub fn message_kind(&self) -> &'static str {
        match self {
            ChildToParent::Answer(_) => "Answer",
            ChildToParent::BenchResult(_) => "BenchResult",
            ChildToParent::Err(_) => "Err",
        }
    }
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Answer<'a> {
    answer: Cow<'a, str>,
    time: Duration,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct BenchResult<'a> {
    pub times: Vec<Duration>,
    pub answer: Cow<'a, str>,
}

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("bincode decode\n{0}")]
    Decoding(#[from] DecodeError),
    #[error("bincode encode\n{0}")]
    Encoding(#[from] EncodeError),

    #[error("child encountered error: {0}")]
    ChildError(Box<str>),
    #[error("bencher found wrong answer: {0:?} != {1:?}")]
    WrongAnswerInBench(Box<str>, Box<str>),

    #[error("child was not sent an initialzation message")]
    ChildWasNotInitialized,

    #[error("child quit before sending a response")]
    ChildQuit,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("parent expected an answer but received {}\n{received:?}", .received.message_kind())]
    ParentExpectedAnswer { received: ChildToParent<'static> },

    #[error("parent asked for {asked} benches but received {received}")]
    WrongNumberOfBenches { asked: u32, received: u32 },
}
