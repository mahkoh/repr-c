// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::{anyhow, Context, Result};
use clap::{App, Arg};
use repr_c_impl::target::{Target, TARGETS};
use std::fs::File;
use std::io::{stdin, Read};
use std::process;

fn args() -> (Target, Option<String>) {
    let matches = App::new("c-layout")
        .arg(
            Arg::with_name("print-targets")
                .long("print-targets")
                .help("Prints all available targets"),
        )
        .arg(
            Arg::with_name("target")
                .long("target")
                .takes_value(true)
                .help("Sets the target"),
        )
        .arg(Arg::with_name("input").required(false))
        .get_matches();
    if matches.is_present("print-targets") {
        for t in TARGETS {
            println!("{}", t.name());
        }
        process::exit(0);
    }
    let target = match matches.value_of("target") {
        None => match repr_c_impl::target::HOST_TARGET {
            Some(t) => t,
            _ => {
                eprintln!("The host target {} is not implemented.", env!("TARGET"));
                eprintln!("Specify a different target with the --target option.");
                eprintln!("Print all available targets with the --print-targets flag.");
                process::exit(1);
            }
        },
        Some(target) => match TARGETS.iter().copied().find(|t| t.name() == target) {
            Some(t) => t,
            _ => {
                eprintln!("Invalid target {}.", target);
                process::exit(1);
            }
        },
    };
    (target, matches.value_of("input").map(|s| s.to_owned()))
}

fn main() {
    if let Err(e) = main_() {
        eprintln!("{:#}", e);
        process::exit(1);
    }
}

fn main_() -> Result<()> {
    let (target, file) = args();
    let mut input = String::new();
    match file {
        Some(p) => File::open(&p)
            .with_context(|| anyhow!("cannot open {}", p))?
            .read_to_string(&mut input)
            .with_context(|| anyhow!("cannot read from {}", p))?,
        _ => stdin()
            .read_to_string(&mut input)
            .context("cannot read from stdin")?,
    };
    let res = c_layout_impl::parse(&input).context("Parsing failed")?;
    let layouts = c_layout_impl::compute_layouts(&input, &res, target)
        .context("Layout computation failed")?;
    let res = c_layout_impl::enhance_declarations(&res, &layouts);
    print!("{}", c_layout_impl::printer(&input, &res));
    Ok(())
}
