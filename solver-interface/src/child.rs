use super::{
    bincode_config, time_fn, Answer, Bench, BenchResult, ChildToParent, ParentToChild, Run,
    SolverError,
};

use bincode::{decode_from_reader, encode_into_std_write, error::DecodeError};

use std::borrow::Cow;
use std::fmt::Display;
use std::hint::black_box;
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::time::Duration;

/// A type that can accept instructions from a parent process on which parts
/// and inputs to run.
pub trait ChildSolver {
    fn part_one(input: &[u8], _debug: u8) -> impl Display + 'static;
    fn part_two(input: &[u8], _debug: u8) -> impl Display + 'static;
    fn run_any(_input: &[u8], _part: u32, _debug: u8) -> impl Display + 'static {
        panic!("Not implemented");
        #[allow(unreachable_code)]
        ""
    }
}

/// The interface used to run a `ChildSolver`.
pub trait ChildSolverExt: ChildSolver {
    /// Starts the message receiver and sender, which calls the other
    /// methods in the trait when the appropriate messages are received.
    fn run() -> Result<(), SolverError> {
        let mut stdin = BufReader::new(stdin());
        let mut stdout = BufWriter::new(stdout());
        let config = bincode_config();
        let mut buf = String::new();

        // First message must be initialization
        let ParentToChild::Initialize(mut init) =
            decode_from_reader::<ParentToChild, _, _>(&mut stdin, config)?
        else {
            return Err(SolverError::ChildWasNotInitialized);
        };

        loop {
            let received = decode_from_reader::<ParentToChild, _, _>(&mut stdin, config);
            let received = match received {
                Ok(m) => m,
                Err(DecodeError::UnexpectedEnd { .. }) => break Ok(()),
                Err(DecodeError::Io { inner, .. }) => break Err(inner.into()),
                Err(e) => break Err(e.into()),
            };

            let msg = match received {
                ParentToChild::Initialize(begin) => {
                    init = begin;
                    continue;
                }
                ParentToChild::Run(run) => {
                    let (time, answer) = run.time_solver::<Self>(&init.input, &mut buf, init.debug);
                    ChildToParent::Answer(Answer {
                        time,
                        answer: answer.into(),
                    })
                }
                ParentToChild::Bench(bench) => {
                    let times = Self::bench(bench, &init.input, &mut buf, init.debug)?;
                    ChildToParent::BenchResult(BenchResult {
                        times,
                        answer: Cow::Borrowed(&buf),
                    })
                }
                ParentToChild::End => break Ok(()),
            };

            encode_into_std_write(msg, &mut stdout, config)?;
            stdout.flush()?;
        }
    }

    fn bench(
        bench: Bench,
        input: &[u8],
        buf: &mut String,
        debug: u8,
    ) -> Result<Vec<Duration>, SolverError> {
        let Bench {
            run: Run { part },
            iters,
        } = bench;

        let mut times = Vec::with_capacity(iters as usize);

        match part {
            1 => bench_iters(|| Self::part_one(black_box(input), debug), buf, &mut times),
            2 => bench_iters(|| Self::part_two(black_box(input), debug), buf, &mut times),
            _ => bench_iters(
                || Self::run_any(black_box(input), part, debug),
                buf,
                &mut times,
            ),
        }?;
        Ok(times)
    }
}

fn bench_iters<D: Display>(
    f: impl Fn() -> D,
    buf: &mut String,
    times: &mut Vec<Duration>,
) -> Result<(), SolverError> {
    use std::fmt::Write;

    let first_ans = f();
    buf.clear();
    write!(buf, "{first_ans}").unwrap();
    let ans_len = buf.len();

    for _ in 0.. {
        let (d, ans) = time_fn(&f);
        write!(buf, "{ans}").unwrap();
        if buf[..ans_len] != buf[ans_len..] {
            let b = buf.split_off(ans_len);
            return Err(SolverError::WrongAnswerInBench(
                buf.clone().into(),
                b.into(),
            ));
        }
        buf.truncate(ans_len);
        times.push(d);
    }

    Ok(())
}

impl<T: ChildSolver + ?Sized> ChildSolverExt for T {}
