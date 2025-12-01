use chrono::{FixedOffset, NaiveDate, TimeDelta, Utc};
use clap::{ArgAction, Command, Parser, ValueEnum};
use clap_complete::Shell;
use notify::{RecommendedWatcher, Watcher};
use regex::bytes::Regex;
use solver_interface::{BenchResult, ParentSolver};
use ureq::http::Response;
use ureq::{Agent, Body};

use std::borrow::Cow;
use std::fmt::Display;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write, stderr, stdin, stdout};
use std::iter::Sum;
use std::ops::Div;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, SyncSender, channel, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::{AocError, Res};

/// User agent (see [Eric's post on the
/// subreddit](https://www.reddit.com/r/adventofcode/comments/z9dhtd))
const USER_AGENT: &str = "github.com/drewtato/aoc2025";

/// How long to wait between polls in watch mode
const WATCH_POLL_TIME: Duration = Duration::from_millis(20);

const YEAR: u32 = 2025;

/// Settings for running AoC. Usually created with [`clap::Parser::parse`].
#[derive(Debug, Parser)]
#[command(about = "A runner for Advent of Code", version = clap::crate_version!())]
pub struct Settings {
    /// Specify which days to run.
    ///
    /// Passing 0 will run all 12. To run a specific part, pass `day.part`, like
    /// `2.1` for part 1 of day 2, or `2.1.2` for both parts of day 2 (same as
    /// `2`).
    pub days: Vec<String>,

    /// Select which mode to run in.
    #[arg(short, long, value_enum, default_value_t = Mode::Run)]
    pub mode: Mode,

    /// Specify a number of milliseconds.
    ///
    /// Ignored if `--bench-count` is nonzero. When in bench mode, you can
    /// specify how long to repeatedly run each day. This runs for one second by
    /// default.
    #[arg(short = 's', long = "bench-time", default_value_t = 1000)]
    pub bench_time: u64,

    /// Specify a number of iterations.
    ///
    /// Overrides `--bench-time`. When in bench mode, specify to do a set number
    /// of iteratons instead of running as many as possible in a certain amount
    /// of time.
    #[arg(short = 'c', long = "bench-count", default_value_t = 0)]
    pub bench_count: u32,

    /// Hide answers in output.
    #[arg(short = 'a', long)]
    pub hide_answers: bool,

    /// Exit on incorrect answers in validation mode.
    #[arg(short, long)]
    pub exit_on_incorrect: bool,

    // /// Runs days in parallel.
    // #[arg(long, short)]
    // pub parallel: bool,
    /// Enables debug mode for the days.
    ///
    /// Pass this flag multiple times to enable more debug info.
    #[arg(short, long, action = ArgAction::Count)]
    pub debug: u8,

    /// Run with the specified test input.
    ///
    /// Best used with one day selected. 0 corresponds to the real input.
    #[arg(short, long, default_value_t = 0)]
    pub test: u8,

    /// Enables debug info for the runner.
    #[arg(short, long, action = ArgAction::Count)]
    pub runner_debug: u8,

    /// Builds the solver with the release profile.
    #[arg(short = 'l', long)]
    pub release: bool,

    /// Runs the solver anew whenever a change is detected in the input, days,
    /// or helper directories.
    ///
    /// Panics and errors in the solver are logged, but do not abort the runner.
    /// When the watcher is running, the following commands can be provided:
    ///
    /// - Any mode argument: replace the current mode
    /// - x: run the solver again manually
    /// - q: exit
    #[arg(short, long)]
    pub watch: bool,

    #[arg(skip = OnceLock::new())]
    client: OnceLock<Agent>,
    #[arg(skip = OnceLock::new())]
    regex: OnceLock<Regex>,
    #[arg(skip = OnceLock::new())]
    input_channel: OnceLock<(Receiver<InputMessage>, JoinHandle<()>)>,
    #[arg(skip = OnceLock::new())]
    watcher_channel: OnceLock<(Receiver<Res<()>>, RecommendedWatcher)>,

    /// Print a shell completion script.
    #[arg(long)]
    pub completions: Option<Shell>,

    /// Override the year.
    #[arg(long, default_value_t = YEAR)]
    pub year: u32,
}

/// Mode to run [`Settings`] in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, ValueEnum)]
pub enum Mode {
    #[default]
    /// Run the specified days and generate their output.
    #[value(alias("r"))]
    Run,
    /// Benchmark the specified days.
    ///
    /// This ignores part info and will always run initialization, part one, and
    /// part two as fast as possible.
    #[value(alias("b"))]
    Bench,
    /// Save the specified days' output as validation files, to be used with
    /// `--validate`.
    #[value(alias("s"))]
    Save,
    /// Validate that the output of the specified days equals the saved output
    /// in validation files.
    #[value(alias("v"))]
    Validate,
    /// Retrieve the prompt and test cases
    #[value(alias("p"))]
    Prompt,
}

macro_rules! debug_println {
	($dbg:expr, $level:expr, $($tok:expr),*$(,)?) => {
		if $dbg >= $level {
			eprintln!($($tok),*);
		}
	};
}

impl Settings {
    pub fn run(&mut self) -> Res<()> {
        if let Some(shell) = self.completions {
            use clap::CommandFactory;
            let mut cmd: Command = Self::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut stdout());
            return Ok(());
        }

        let mut runner_start = Instant::now();
        let mut solver_time = Duration::ZERO;

        debug_println!(self.runner_debug, 2, "{:?}", self);
        debug_println!(self.runner_debug, 1, "Starting runner");

        let mut day_parts = Vec::new();
        // Can't use collect because I need to flatten the Vec inside the Result
        for item in self.days.iter().map(|word| parse_day(word)) {
            day_parts.extend_from_slice(&item?);
        }

        solver_time += loop {
            let res = match self.mode {
                Mode::Run => self.run_days(&day_parts),
                Mode::Bench => self.benchmark(&day_parts),
                Mode::Save => self.save(&day_parts),
                Mode::Validate => self.validate(&day_parts),
                Mode::Prompt => self.prompt(&day_parts),
            };

            if !self.watch {
                break res?;
            }

            if let Err(e) = res {
                eprintln!("{e}");
            }

            let watch_start = Instant::now();
            let done = self.watch()?;
            // Don't count time watching
            runner_start += watch_start.elapsed();

            if done {
                break Duration::ZERO;
            }
        };

        let runner_time = runner_start.elapsed();
        debug_println!(
            self.runner_debug,
            1,
            "Total time: {:?}\nRunner time: {:?}",
            runner_time,
            runner_time - solver_time,
        );
        Ok(())
    }

    /// Returns `true` when asked to exit
    fn watch(&mut self) -> Res<bool> {
        // OnceLock guarantees only one function will be executed, so only one thread
        // will spawn
        let (input_recv, _) = self.input_channel.get_or_init(|| {
            let (send, recv) = sync_channel(0);
            let handle = std::thread::spawn(self.create_input_watcher(send));
            (recv, handle)
        });

        let (watcher_recv, _) = self.watcher_channel.get_or_init(|| {
            let (send, recv) = channel();
            let watcher = self.create_file_watcher(send).unwrap();
            (recv, watcher)
        });

        loop {
            match input_recv.recv_timeout(WATCH_POLL_TIME) {
                Ok(InputMessage::Exit) => return Ok(true),
                Ok(InputMessage::Rerun) => break,
                Ok(InputMessage::SetMode(mode)) => {
                    self.mode = mode;
                    break;
                }
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => panic!("input reader thread stopped"),
            }

            if let Some(ok) = watcher_recv.try_iter().last() {
                ok?;
                break;
            }
        }

        Ok(false)
    }

    fn create_file_watcher(&self, send: Sender<Res<()>>) -> Res<RecommendedWatcher> {
        use notify::EventKind::*;

        let mut watcher =
            notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                match event {
                    Err(e) => send.send(Err(e.into())).unwrap(),
                    Ok(event) => match event.kind {
                        Create(_) | Modify(_) | Remove(_) => send.send(Ok(())).unwrap(),
                        _ => (),
                    },
                }
            })?;

        watcher.watch("inputs".as_ref(), notify::RecursiveMode::Recursive)?;
        watcher.watch("days".as_ref(), notify::RecursiveMode::Recursive)?;
        watcher.watch("helpers".as_ref(), notify::RecursiveMode::Recursive)?;

        Ok(watcher)
    }

    fn create_input_watcher(&self, send: SyncSender<InputMessage>) -> impl FnOnce() + 'static {
        let mut buf = String::new();
        let stdin = stdin();
        move || loop {
            buf.clear();
            stdin.read_line(&mut buf).unwrap();
            let line = buf.trim();
            let message = match Mode::from_str(line, true) {
                Ok(mode) => InputMessage::SetMode(mode),
                Err(_) => match line {
                    "q" | "Q" => InputMessage::Exit,
                    "x" | "X" => InputMessage::Rerun,
                    _ => continue,
                },
            };
            send.send(message).unwrap();
        }
    }

    fn run_days(&mut self, day_parts: &[(u32, Vec<u32>)]) -> Res<Duration> {
        let mut test_time = Duration::ZERO;
        let mut buffer = String::new();
        for &(day, ref parts) in day_parts {
            debug_println!(self.runner_debug, 1, "Starting day {day}");

            let mut day_time = Duration::ZERO;

            if !(1..=12).contains(&day) {
                eprintln!("Day {day} not found, skipping");
                continue;
            }

            let file = self.get_input(day)?;
            let mut solver = self.day_to_solver(day, file)?;

            if parts.is_empty() {
                let time = solver.part_one(&mut buffer)?;
                day_time += time;

                if !self.hide_answers {
                    print_times(day, 1, &buffer, time);
                } else {
                    print_times(day, 1, "", time);
                }
                buffer.clear();

                let time = solver.part_two(&mut buffer)?;
                day_time += time;

                if !self.hide_answers {
                    print_times(day, 2, &buffer, time);
                } else {
                    print_times(day, 2, "", time);
                }
                buffer.clear();
            }

            for &part in parts {
                let time = match part {
                    1 => solver.part_one(&mut buffer),
                    2 => solver.part_two(&mut buffer),
                    p => solver.run_any(p, &mut buffer),
                }?;
                day_time += time;

                if !self.hide_answers {
                    print_times(day, part, &buffer, time);
                } else {
                    print_times(day, part, "", time);
                }
                buffer.clear();
            }

            eprintln!("d{day:02} total: {day_time:?}\n");
            test_time += day_time;
        }
        eprintln!("All: {test_time:?}");
        Ok(test_time)
    }

    fn get_input(&mut self, day: u32) -> Res<Vec<u8>> {
        let input_main = input_file_name(day, 0);
        if !input_main.exists() {
            let time_until_release = time_until_input_is_released(day, self.year);
            // If the puzzle is very far out
            if time_until_release > TimeDelta::hours(1) {
                // eprintln!(
                // 	"Puzzle doesn't release for {:?}",
                // 	time_until_release.to_std().unwrap()
                // );
                return Err(AocError::has_not_released_yet(day, time_until_release));
            }

            // If the puzzle hasn't been out for at least 5 seconds
            if time_until_release > TimeDelta::seconds(-5) {
                use std::io::Write;
                let delay = time_until_release + TimeDelta::seconds(5);

                {
                    let mut stderr = stderr();
                    write!(stderr, "Puzzle releases in ").unwrap();
                    readable_time(&stderr, time_until_release.to_std().unwrap_or_default(), 0)
                        .unwrap();
                    write!(stderr, ", waiting ").unwrap();
                    readable_time(&stderr, delay.to_std().unwrap(), 0).unwrap();
                    writeln!(stderr).unwrap();
                    stderr.flush().unwrap();
                }

                std::thread::sleep(delay.to_std().unwrap());
            }

            self.get_input_network(day)?;
        }

        let input = if self.test == 0 {
            std::fs::read(input_main)
        } else {
            std::fs::read(input_file_name(day, self.test))
        }?;

        Ok(input)
    }

    /// Get the input from the network and write it to the filesystem. Will
    /// overwrite any existing input files.
    fn get_input_network(&mut self, day: u32) -> Res<()> {
        let api_key = api_key()?;
        let api_key = api_key.trim();

        // Get main input
        let url = format!("https://adventofcode.com/{}/day/{day}/input", self.year);
        eprintln!("Fetching {url}");

        let req = self
            .client()
            .get(&url)
            .header("cookie", &format!("session={api_key}"))
            .call()?;
        if !req.status().is_success() {
            return Err(AocError::input_response(
                req.status(),
                req.into_body().read_to_string()?,
            ));
        }
        let data = read_to_vec(req)?;

        let path = input_base_name(day);
        create_dir_all(path)?;
        let input_path = input_file_name(day, 0);
        std::fs::write(input_path, data)?;

        self.get_prompt(day, api_key)?;

        Ok(())
    }

    fn client(&self) -> &Agent {
        self.client.get_or_init(|| {
            Agent::config_builder()
                .user_agent(USER_AGENT)
                .build()
                .new_agent()
        })
    }

    fn get_prompt(&mut self, day: u32, api_key: &str) -> Result<(), AocError> {
        let url = format!("https://adventofcode.com/{}/day/{day}", self.year);
        if self.runner_debug > 0 {
            eprintln!("Fetching {url}");
        }
        let req = self
            .client()
            .get(&url)
            .header("cookie", &format!("session={api_key}"))
            .call()?;
        if !req.status().is_success() {
            return Err(AocError::prompt_response(
                req.status(),
                req.into_body().read_to_string()?,
            ));
        }
        let text = read_to_vec(req)?;
        let prompt_path = prompt(day);
        std::fs::write(prompt_path, &text)?;

        // Save each code block as a test case
        let regex = self.regex();

        for (i, code) in regex.captures_iter(&text).enumerate() {
            let Ok(i) = (i + 1).try_into() else {
                eprintln!("{}, skipping the rest", AocError::TooManyTestCases);
                break;
            };

            debug_println!(self.runner_debug, 1, "Got a code match, making a test {i}");

            let code = &code[1];

            let test_path = input_file_name(day, i);
            let file = File::create(test_path)?;
            let mut file = BufWriter::new(file);

            html_escape::decode_html_entities_to_writer(
                std::str::from_utf8(code).map_err(|_| AocError::NonUtf8InPromptCodeBlock)?,
                &mut file,
            )?;
        }

        Ok(())
    }

    fn regex(&self) -> &Regex {
        self.regex
            .get_or_init(|| Regex::new(r"<pre>\s*<code>([^<]+)</code>\s*</pre>").unwrap())
    }

    fn benchmark(&mut self, day_parts: &[(u32, Vec<u32>)]) -> Res<Duration> {
        let mut solver_time = Duration::ZERO;

        for &(day, ref parts) in day_parts {
            debug_println!(self.runner_debug, 2, "starting bencher for day {day}");
            let input = self.get_input(day)?;
            let mut bencher = self.day_to_bencher(day, input)?;
            debug_println!(self.runner_debug, 2, "ParentSolver started");

            for &part in parts {
                debug_println!(self.runner_debug, 1, "Benching part {part}");
                let (mut times, answer) = if self.bench_count == 0 {
                    debug_println!(self.runner_debug, 2, "this is a timed bench");
                    let bench_time = Duration::from_millis(self.bench_time);

                    let BenchResult { times, answer } = bencher.bench(part, 1)?;
                    debug_println!(self.runner_debug, 2, "got {} results", times.len());

                    if times[0] > bench_time {
                        (times, answer.into_owned())
                    } else {
                        let first_answer = answer.into_owned();

                        let test_times = if times[0] > bench_time / 10 {
                            times
                        } else {
                            let BenchResult { mut times, answer } = bencher.bench(part, 10)?;
                            if answer != first_answer {
                                return Err(AocError::IncorrectAnswer);
                            }
                            times.sort_unstable();
                            // Remove the largest times
                            times.truncate(7);
                            times
                        };

                        let avg_test_time = average(&test_times);
                        let iters_to_do = bench_time.as_nanos() / avg_test_time.as_nanos();

                        let BenchResult { times, answer } =
                            bencher.bench(part, iters_to_do as _)?;
                        if answer != first_answer {
                            return Err(AocError::IncorrectAnswer);
                        }
                        (times, first_answer)
                    }
                } else {
                    debug_println!(self.runner_debug, 2, "this is a counted bench");
                    let BenchResult { times, answer } = bencher.bench(part, self.bench_count)?;
                    (times, answer.into_owned())
                };

                debug_println!(self.runner_debug, 2, "got {} results", times.len());

                let mut to_remove = (times.len().ilog2() as usize).saturating_sub(1) * 2;
                if times.len() > 1 {
                    to_remove += 1;
                }
                let new_len = times.len() - to_remove;
                times.truncate(new_len);

                let avg = average(&times);
                let median = times[times.len() / 2];
                let stderr = stderr();
                eprint!("d{day:02}p{part:02}: avg ");
                readable_time(&stderr, avg, 3).unwrap();
                eprint!(", med ");
                readable_time(&stderr, median, 3).unwrap();
                eprintln!(" ({answer:?})");
                solver_time += avg;
            }
        }

        Ok(solver_time)
    }

    fn save(&mut self, day_parts: &[(u32, Vec<u32>)]) -> Res<Duration> {
        let mut time = Duration::ZERO;
        for &(day, ref parts) in day_parts {
            time += self.save_day(day, parts)?;
        }
        Ok(time)
    }

    fn save_day(&mut self, day: u32, parts: &[u32]) -> Res<Duration> {
        let file = self.get_input(day)?;

        let ans_file_name = answer_file_name(day, self.test);
        let answers = if ans_file_name.exists() {
            std::fs::read_to_string(&ans_file_name)?
        } else {
            String::new()
        };
        let mut answer_vec: Vec<_> = answers.lines().map(Cow::Borrowed).collect();

        let mut solver = self.day_to_solver(day, file)?;
        let mut total_time = Duration::ZERO;

        let mut buf = String::new();

        let parts = if parts.is_empty() { &[1, 2] } else { parts };

        for &part in parts {
            let time = match part {
                1 => solver.part_one(&mut buf),
                2 => solver.part_two(&mut buf),
                p => solver.run_any(p, &mut buf),
            }?;
            total_time += time;

            let part = part as usize - 1;
            if part >= answer_vec.len() {
                answer_vec.resize(part + 1, String::new().into());
            }
            let saved = answer_vec[part].to_mut();

            eprint!("d{day:02}p{:02}: ", part + 1);

            if !saved.is_empty() {
                if buf.eq(saved) {
                    if self.test > 0 {
                        eprintln!("Test {:02} answer is still {:?}", self.test, buf);
                    } else {
                        eprintln!("Answer is still {buf:?}");
                    }
                } else {
                    if self.test > 0 {
                        eprint!("Replacing test {:02} answer", self.test);
                    } else {
                        eprint!("Replacing main answer");
                    }
                    eprintln!(" {saved:?} with {buf:?}");
                }
                saved.clear();
            } else {
                eprint!("Saving ");
                if self.test > 0 {
                    eprint!("test {:02} answer", self.test);
                } else {
                    eprint!("main answer");
                }
                eprintln!(" {buf:?}");
            }

            *saved += &buf;
            buf.clear();
        }

        std::fs::write(ans_file_name, answer_vec.join("\n") + "\n")?;

        Ok(total_time)
    }

    fn validate(&mut self, day_parts: &[(u32, Vec<u32>)]) -> Res<Duration> {
        let mut times = Duration::ZERO;
        let mut incorrect = 0;

        for &(day, ref parts) in day_parts {
            let (t, i) = self.validate_day(day, parts)?;
            times += t;
            incorrect += i;
        }

        if incorrect == 0 {
            eprintln!("All answers were correct!");
            Ok(times)
        } else {
            Err(AocError::MultipleIncorrect(incorrect))
        }
    }

    fn validate_day(&mut self, day: u32, parts: &[u32]) -> Res<(Duration, u32)> {
        let file = self.get_input(day)?;

        let ans_file_name = answer_file_name(day, self.test);
        let answers = if ans_file_name.exists() {
            std::fs::read_to_string(&ans_file_name)?
        } else {
            debug_println!(
                self.runner_debug,
                1,
                "Answer file {:?} missing, saving current answers",
                ans_file_name
            );
            let t = self.save_day(day, parts)?;
            return Ok((t, 0));
        };
        let mut answer_vec: Vec<_> = answers.lines().map(Cow::Borrowed).collect();

        let mut solver = self.day_to_solver(day, file)?;
        let mut total_time = Duration::ZERO;
        let mut buf = String::new();
        let mut incorrect = 0;

        let parts = if parts.is_empty() { &[1, 2] } else { parts };

        for &part in parts {
            let time = match part {
                1 => solver.part_one(&mut buf),
                2 => solver.part_two(&mut buf),
                p => solver.run_any(p, &mut buf),
            }?;
            total_time += time;

            let part = part as usize - 1;
            if part >= answer_vec.len() {
                answer_vec.resize(part + 1, String::new().into());
            }
            let saved = answer_vec[part].to_mut();

            eprint!("d{day:02}p{:02}: ", part + 1);

            if !saved.is_empty() {
                if buf.eq(saved) {
                    if self.test > 0 {
                        eprintln!("Test {:02} answer is correct: {:?}", self.test, buf);
                    } else {
                        eprintln!("Answer is correct: {buf:?}");
                    }
                } else {
                    if self.test > 0 {
                        eprint!("Test {:02} answer", self.test);
                    } else {
                        eprint!("main answer");
                    }
                    eprintln!(" {buf:?} did not match saved answer {saved:?}");
                    if self.exit_on_incorrect {
                        return Err(AocError::IncorrectAnswer);
                    }
                    incorrect += 1;
                }
            } else {
                eprint!("Saving ");
                if self.test > 0 {
                    eprint!("test {:02} answer", self.test);
                } else {
                    eprint!("main answer");
                }
                eprintln!(" {buf:?}");
                saved.clear();
                *saved += &buf;
            }
            buf.clear();
        }

        std::fs::write(ans_file_name, answer_vec.join("\n") + "\n")?;

        Ok((total_time, incorrect))
    }

    fn prompt(&mut self, day_parts: &[(u32, Vec<u32>)]) -> Result<Duration, AocError> {
        let api_key = &api_key()?;
        for &(day, _) in day_parts {
            self.get_prompt(day, api_key)?;
        }
        Ok(Duration::ZERO)
    }

    fn day_to_solver(&self, day: u32, file: Vec<u8>) -> Res<ParentSolver> {
        Ok(ParentSolver::new(day, &file, self.debug, self.release)?)
    }

    fn day_to_bencher(&self, day: u32, file: Vec<u8>) -> Res<ParentSolver> {
        Ok(ParentSolver::new(day, &file, self.debug, true)?)
    }
}

#[derive(Debug, Clone, Copy)]
enum InputMessage {
    SetMode(Mode),
    Rerun,
    Exit,
}

fn average<N>(test_times: &[N]) -> N::Output
where
    N: for<'a> Sum<&'a N> + Div<u32>,
{
    test_times.iter().sum::<N>() / test_times.len() as u32
}

fn api_key() -> Result<String, AocError> {
    use std::io::ErrorKind;
    let mut key = match std::fs::read_to_string("./API_KEY") {
        Ok(key) => key,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(AocError::NoApiKey),
            _ => return Err(AocError::ApiKeyIo { source: e }),
        },
    };
    while key.chars().last().is_some_and(|c| c.is_whitespace()) {
        key.pop();
    }
    Ok(key)
}

fn read_to_vec(req: Response<Body>) -> Res<Vec<u8>> {
    Ok(req.into_body().read_to_vec()?)
}

fn readable_time<W: Write>(
    mut writer: W,
    duration: Duration,
    places: usize,
) -> std::io::Result<()> {
    match duration.as_millis() {
        0 => write!(writer, "{:.places$}μs", duration.as_nanos() as f32 / 1e3),
        1..=999 => write!(writer, "{:.places$}ms", duration.as_nanos() as f32 / 1e6),
        1_000..=119_999 => write!(writer, "{:.places$}s", duration.as_nanos() as f32 / 1e9),
        120_000.. => write!(
            writer,
            "{:.places$} minutes",
            duration.as_nanos() as f32 / 1e9 / 60.0
        ),
    }
}

fn print_times<D: Display>(day: u32, part: u32, ans: D, time: Duration) {
    eprintln!("d{day:02}p{part:02}: ({time:?}) {ans}");
}

fn prompt(day: u32) -> PathBuf {
    let mut name = input_base_name(day);
    name.push("prompt.html");
    name
}

fn answer_file_name(day: u32, test: u8) -> PathBuf {
    let mut name = input_base_name(day);
    if test > 0 {
        name.push(format!("answer{test:02}.txt"));
    } else {
        name.push("answer.txt");
    }
    name
}

fn input_file_name(day: u32, test: u8) -> PathBuf {
    let mut name = input_base_name(day);
    if test > 0 {
        name.push(format!("input{test:02}.txt"));
    } else {
        name.push("input.txt");
    }
    name
}

fn input_base_name(day: u32) -> PathBuf {
    PathBuf::from(format!("./inputs/day{day:02}"))
}

fn parse_day(word: &str) -> Res<Vec<(u32, Vec<u32>)>> {
    let mut nums = word.split('.');
    let day = if let Some(n) = nums.next() {
        if n.is_empty() {
            Err(AocError::no_day_specified(word))
        } else {
            n.parse().map_err(|_| AocError::parse(n, word))
        }
    } else {
        Err(AocError::EmptyArgument)
    }?;

    let rest = nums
        .map(|n| {
            if n.is_empty() {
                Err(AocError::empty_part(word))
            } else {
                n.parse().map_err(|_| AocError::parse(n, word))
            }
        })
        .collect::<Res<Vec<u32>>>()?;

    Ok(if day == 0 {
        (1..=12).map(|n| (n, rest.clone())).collect()
    } else {
        vec![(day, rest)]
    })
}

/// Returns `None` if the input is released, otherwise returns the time until
/// release. Returns `None` if the time cannot be determined.
///
/// # Warning
///
/// This is likely to break (by not allowing downloading of the puzzle for an
/// extra hour) if the United States decides to remove time changes in favor of
/// sticking to Daylight Saving Time, and Eric Wastl continues to keep AoC on
/// US-East time. In such an event, change `ERIC_TIME_OFFSET` to `-4`.
// Note: chrono is actually way more confusing than I thought. Idk if this is
// the correct way to use it but it seems to work.
fn time_until_input_is_released(day: u32, year: u32) -> TimeDelta {
    const ERIC_TIME_OFFSET: i32 = -5;

    let t = Utc::now().naive_utc();

    let release = NaiveDate::from_ymd_opt(year as _, 12, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(FixedOffset::east_opt(ERIC_TIME_OFFSET * 60 * 60).unwrap())
        .unwrap()
        .naive_utc();

    release - t
}
