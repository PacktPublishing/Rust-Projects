use clap::{clap_app, crate_version};
use failure::{Error, Fail};
use regex::Regex;

use std::path::Path;

#[derive(Debug, Fail)]
#[fail(display = "Argument not provided '{}'", arg)]
struct ArgErr {
    arg: &'static str,
}

#[derive(Debug)]
struct Record {
    line: usize,
    tx: String,
}

fn process_file<P: AsRef<Path>>(p: P, re: &Regex) -> Result<Vec<Record>, Error> {
    let mut res = Vec::new();
    let bts = std::fs::read(p)?;
    if let Ok(ss) = String::from_utf8(bts) {
        for (i, l) in ss.lines().enumerate() {
            if re.is_match(l) {
                res.push(Record {
                    line: i,
                    tx: l.to_string(),
                });
            }
        }
    }
    Ok(res)
}

fn process_path<P, FF, EF>(p: P, re: &Regex, ff: &FF, ef: &EF) -> Result<(), Error>
where
    P: AsRef<Path>,
    FF: Fn(&Path, Vec<Record>),
    EF: Fn(Error),
{
    let p = p.as_ref();
    let md = p.metadata()?;
    let ft = md.file_type();
    if ft.is_file() {
        let dt = process_file(p, re)?;
        ff(p, dt);
    }

    if ft.is_dir() {
        let dd = std::fs::read_dir(p)?;
        for d in dd {
            if let Err(e) = process_path(d?.path(), re, ff, ef) {
                ef(e);
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("There was an error: {}", e);
    }
}

fn run() -> Result<(), Error> {
    let cp = clap_app!(
        pgrep =>
        (version : crate_version!())
        (about : "A Grep like program")
        (author : "Matt")
        (@arg pattern : +required "The regex pattern to search for")
        (@arg file : -f --file +takes_value "The file to test")
    )
    .get_matches();

    let re = Regex::new(cp.value_of("pattern").unwrap())?;

    let p = process_path(
        cp.value_of("file").ok_or(ArgErr { arg: "file" })?,
        &re,
        &|pt, v| {
            println!("{:?}", pt);
            println!("{:?}", v);
        },
        &|e| {
            println!("Error:{}", e);
        },
    );

    println!("{:?}", p);
    Ok(())
}
