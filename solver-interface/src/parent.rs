use super::bincode_config;
use super::{Bench, BenchResult, ChildToParent, Initialization, ParentToChild, Run, SolverError};
use bincode::{config::Configuration, decode_from_reader, encode_into_std_write};
use std::io::{BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

pub struct ParentSolver {
    process: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    config: Configuration,
}

impl ParentSolver {
    /// Start up the manager of a child solver.
    ///
    /// Compiles, runs, and sends input to the child.
    pub fn new(day: u32, input: &[u8], debug: u8, release: bool) -> Result<Self, SolverError> {
        let mut process = Command::new("cargo");
        process
            .args(["run", "--package"])
            .arg(format!("day{day:02}"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        if release {
            process.args(["--profile", "day-release"]);
        } else {
            process.args(["--profile", "day-dev"]);
        }

        let mut process = process.spawn()?;
        let stdin = BufWriter::new(process.stdin.take().unwrap());
        let stdout = BufReader::new(process.stdout.take().unwrap());
        let config = bincode_config();

        let mut this = Self {
            process,
            stdin,
            stdout,
            config,
        };

        let init = Initialization {
            input: input.into(),
            debug,
        };
        this.initialize(init)?;

        Ok(this)
    }

    /// Send new input to the child, overwriting the previous input.
    pub fn initialize(&mut self, init: Initialization) -> Result<(), SolverError> {
        self.send(ParentToChild::Initialize(init))
    }

    fn send(&mut self, message: ParentToChild) -> Result<(), SolverError> {
        encode_into_std_write(message, &mut self.stdin, self.config)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn receive(&mut self) -> Result<ChildToParent<'static>, SolverError> {
        let msg = decode_from_reader(&mut self.stdout, self.config)?;
        Ok(msg)
    }

    /// Run part one.
    pub fn part_one(&mut self, buffer: &mut String) -> Result<Duration, SolverError> {
        self.run_any(1, buffer)
    }

    /// Run part two.
    pub fn part_two(&mut self, buffer: &mut String) -> Result<Duration, SolverError> {
        self.run_any(2, buffer)
    }

    /// Run any part besides part one or two.
    pub fn run_any(&mut self, part: u32, buffer: &mut String) -> Result<Duration, SolverError> {
        self.send(ParentToChild::Run(Run { part }))?;
        let ans = match self.receive()? {
            ChildToParent::Answer(ans) => ans,
            ChildToParent::Err(err) => return Err(SolverError::ChildError(err)),
            msg => return Err(SolverError::ParentExpectedAnswer { received: msg }),
        };
        buffer.clear();
        buffer.push_str(&ans.answer);
        Ok(ans.time)
    }

    /// Run a benchmark.
    pub fn bench(&mut self, part: u32, iters: u32) -> Result<BenchResult<'static>, SolverError> {
        self.send(ParentToChild::Bench(Bench {
            run: Run { part },
            iters,
        }))?;

        match self.receive()? {
            ChildToParent::BenchResult(br) => {
                let benches = br.times.len() as u32;
                if benches != iters {
                    return Err(SolverError::WrongNumberOfBenches {
                        asked: iters,
                        received: benches,
                    });
                }
                Ok(br)
            }
            ChildToParent::Err(e) => Err(SolverError::ChildError(e)),
            msg => Err(SolverError::ParentExpectedAnswer { received: msg }),
        }
    }
}

impl Drop for ParentSolver {
    fn drop(&mut self) {
        self.send(ParentToChild::End).unwrap();
        self.process.wait().unwrap();
    }
}
