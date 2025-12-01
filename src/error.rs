use std::error::Error;
use std::fmt::Display;

use chrono::TimeDelta;
use solver_interface::SolverError;
use thiserror::Error;
use ureq::http::StatusCode;

#[derive(Debug, Error)]
pub enum AocError {
    #[error("part not found")]
    PartNotFound,
    #[error(transparent)]
    HasNotReleasedYet(HasNotReleasedYet),
    #[error("no test input found with the name {path}")]
    NoTestInputFound { path: Box<str> },
    #[error("the file API_KEY was not found")]
    NoApiKey,
    #[error("I/O problem while reading API_KEY file: {source}")]
    ApiKeyIo { source: std::io::Error },
    #[error("couldn't fetch prompt from network. Status {status}, content:\n{response}")]
    PromptResponse {
        status: StatusCode,
        response: Box<str>,
    },
    #[error("couldn't fetch input from network. Status: {status}\nContent:\n{response}")]
    InputResponse {
        status: StatusCode,
        response: Box<str>,
    },
    #[error("no day specified in argument `{arg}`")]
    NoDaySpecified { arg: Box<str> },
    #[error(transparent)]
    Parse(Parse),
    #[error("non-UTF-8 data found in code block on the prompt page")]
    NonUtf8InPromptCodeBlock,
    #[error("non-UTF-8 data found in solution")]
    NonUtf8InSolution,
    #[error("day {0} not found")]
    DayNotFound(u32),
    #[error("argument was empty")]
    EmptyArgument,
    #[error("part was empty in {arg}")]
    EmptyPart { arg: Box<str> },
    #[error("too many test cases were generated from the prompt")]
    TooManyTestCases,
    #[error("answers did not match, exiting run")]
    IncorrectAnswer,
    #[error("{0} answers were incorrect.")]
    MultipleIncorrect(u32),

    #[error("request: {source}")]
    Request {
        #[from]
        source: Box<ureq::Error>,
    },
    #[error("solver: {source}")]
    Solver {
        #[from]
        source: Box<SolverError>,
    },
    #[error("notify: {source}")]
    Watcher {
        #[from]
        source: Box<notify::Error>,
    },
    #[error("fmt: {source}")]
    FmtError {
        #[from]
        source: std::fmt::Error,
    },
    #[error("io: {source}")]
    File {
        #[from]
        source: std::io::Error,
    },

    #[error("other: {source}")]
    OtherError {
        #[from]
        source: Box<dyn std::error::Error + Send + 'static>,
    },
}

#[derive(Debug)]
pub struct Parse {
    part_arg: Box<str>,
    part_len: usize,
}

impl Parse {
    pub fn new(part: impl Into<String>, arg: impl AsRef<str>) -> Self {
        let mut part = part.into();
        let part_len = part.len();
        part.push_str(arg.as_ref());
        Parse {
            part_arg: part.into(),
            part_len,
        }
    }

    pub fn part(&self) -> &str {
        &self.part_arg[..self.part_len]
    }

    pub fn arg(&self) -> &str {
        &self.part_arg[self.part_len..]
    }
}

impl Error for Parse {}

impl Display for Parse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part = self.part();
        let arg = self.arg();
        write!(f, "could not parse `{part}` as integer in argument `{arg}`")
    }
}

#[derive(Debug)]
pub struct HasNotReleasedYet {
    day: u32,
    duration: TimeDelta,
}
impl HasNotReleasedYet {
    pub fn new(day: u32, duration: TimeDelta) -> Self {
        Self { day, duration }
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn duration(&self) -> TimeDelta {
        self.duration
    }
}

impl Error for HasNotReleasedYet {}

impl Display for HasNotReleasedYet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "day {} hasn't released yet. It releases {}:{:02}:{:02}:{:02} from now.",
            self.day,
            self.duration.num_days(),
            self.duration.num_hours() - self.duration.num_days() * 24,
            self.duration.num_minutes() - self.duration.num_hours() * 60,
            self.duration.num_seconds() - self.duration.num_minutes() * 60
        )
    }
}

macro_rules! from_error_boxed {
	($($from:ty, $var:ident;)*) => {
		$(
			impl From<$from> for AocError {
				fn from(value: $from) -> Self {
					Self::$var {
						source: Box::new(value),
					}
				}
			}
		)*
	};
}

from_error_boxed! {
    ureq::Error, Request;
    SolverError, Solver;
    notify::Error, Watcher;
}

impl AocError {
    pub fn no_test_input_found(s: impl Into<Box<str>>) -> Self {
        Self::NoTestInputFound { path: s.into() }
    }

    pub fn parse(part: impl Into<String>, arg: impl AsRef<str>) -> Self {
        Self::Parse(Parse::new(part, arg))
    }

    pub fn empty_part(arg: impl Into<Box<str>>) -> Self {
        Self::EmptyPart { arg: arg.into() }
    }

    pub fn prompt_response(status: StatusCode, response: impl Into<Box<str>>) -> Self {
        Self::PromptResponse {
            status,
            response: response.into(),
        }
    }

    pub fn input_response(status: StatusCode, response: impl Into<Box<str>>) -> Self {
        Self::InputResponse {
            status,
            response: response.into(),
        }
    }

    pub fn no_day_specified(arg: impl Into<Box<str>>) -> Self {
        Self::NoDaySpecified { arg: arg.into() }
    }

    pub(crate) fn has_not_released_yet(day: u32, duration: TimeDelta) -> AocError {
        Self::HasNotReleasedYet(HasNotReleasedYet::new(day, duration))
    }
}
