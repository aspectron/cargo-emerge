use crate::context::Context;
use crate::error::Error;
use crate::result::Result;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Execute a command and stream output to stdout if verbose mode is enabled
pub fn execute(ctx: &Context, program: &str, args: &[&str]) -> Result<()> {
    if ctx.verbose {
        println!("Executing: {} {}", program, args.join(" "));
    }

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(|l| l.ok()) {
            if ctx.verbose {
                println!("{}", line);
            }
        }
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(|l| l.ok()) {
            if ctx.verbose {
                eprintln!("{}", line);
            }
        }
    }

    let status = child.wait()?;

    if !status.success() {
        return Err(Error::CommandFailed(format!(
            "{} {} failed with exit code: {}",
            program,
            args.join(" "),
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Execute a command and capture its output
pub fn execute_with_output(ctx: &Context, program: &str, args: &[&str]) -> Result<String> {
    if ctx.verbose {
        println!("Executing: {} {}", program, args.join(" "));
    }

    let output = Command::new(program).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::CommandFailed(format!(
            "{} {} failed: {}",
            program,
            args.join(" "),
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
