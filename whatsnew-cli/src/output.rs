use std::io::{Write, stdout};
use std::sync::LazyLock;

use anyhow::{Ok, Result};
use jiff::fmt::friendly::{self, SpanPrinter};
use jiff::{SpanRound, SpanTotal, Timestamp, Unit, Zoned};
use owo_colors::{OwoColorize, Stream};
use whatsnew_core::repos::CommitInfo;

static TIME_NOW: LazyLock<Zoned> = LazyLock::new(Zoned::now);

fn print_new_commits<W: Write>(
    writer: &mut W,
    reponame: &str,
    commits: &[CommitInfo],
) -> Result<Option<String>> {
    if commits.is_empty() {
        writeln!(
            writer,
            "{}: No new commits",
            reponame.if_supports_color(Stream::Stdout, |text| text.bright_green())
        )?;
        Ok(None)
    } else {
        writeln!(
            writer,
            "{}: {} new commits:",
            reponame.if_supports_color(Stream::Stdout, |text| text.bright_green()),
            commits.len()
        )?;
        for commit in commits {
            let mut message_lines = commit.message.lines();
            writeln!(
                writer,
                "\n  {}, {}: {}\n    {}\n  {}\n -----------------------------------------------",
                commit
                    .author
                    .if_supports_color(Stream::Stdout, |text| text.bright_blue()),
                get_friendly_time_until_app_start(&commit.commit_time)?,
                message_lines
                    .next()
                    .unwrap_or_default()
                    .if_supports_color(Stream::Stdout, |text| text.yellow()),
                message_lines.fold(String::new(), |output, line| output + "  " + line + "\n"),
                commit
                    .url
                    .if_supports_color(Stream::Stdout, |text| text.bright_black())
            )?;
        }
        Ok(Some(commits[0].sha.clone()))
    }
}

pub fn print_new_commits_to_stdout(
    reponame: &str,
    commits: &[CommitInfo],
) -> Result<Option<String>> {
    let mut stdout = stdout().lock();
    print_new_commits(&mut stdout, reponame, commits)
}

fn get_friendly_time_until_app_start(start_time: &Timestamp) -> Result<String> {
    let diff = start_time.until(&*TIME_NOW)?;

    // Determine the appropriate smallest unit based on how recent the commit is
    let smallest_unit = if diff.total(SpanTotal::from(Unit::Day).days_are_24_hours())? > -1.0 {
        jiff::Unit::Minute
    } else if diff.total(SpanTotal::from(Unit::Month).days_are_24_hours())? > -1.0 {
        jiff::Unit::Hour
    } else if diff.total(SpanTotal::from(Unit::Year).days_are_24_hours())? > -1.0 {
        jiff::Unit::Day
    } else {
        jiff::Unit::Month
    };

    let round = SpanRound::new()
        .largest(jiff::Unit::Year)
        .smallest(smallest_unit)
        .mode(jiff::RoundMode::Trunc)
        .relative(&*TIME_NOW);

    let friendly_printer = SpanPrinter::new()
        .designator(friendly::Designator::Verbose)
        .spacing(friendly::Spacing::BetweenUnitsAndDesignators);

    Ok(friendly_printer.span_to_string(&diff.round(round)?))
}
