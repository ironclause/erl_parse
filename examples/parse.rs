extern crate clap;
extern crate erl_parse;
extern crate erl_pp;
extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use clap::{App, Arg};
use erl_pp::Preprocessor;
use erl_tokenize::Lexer;
use std::env;
use std::fs::File;
use std::io::Read;
use trackable::error::{ErrorKindExt, Failed};

fn main() {
    let matches = App::new("parse")
        .arg(Arg::with_name("ERLANG_FILE").index(1).required(true))
        .arg(
            Arg::with_name("CURRENT_DIR")
                .long("current-dir")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ERL_LIBS")
                .long("libs")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();
    let erlang_file = matches.value_of("ERLANG_FILE").unwrap();
    if let Some(dir) = matches.value_of("CURRENT_DIR") {
        track_try_unwrap!(env::set_current_dir(dir).map_err(|e| Failed.cause(e)));
    }
    let mut file = track_try_unwrap!(File::open(erlang_file).map_err(|e| Failed.cause(e)));
    let mut text = String::new();
    track_try_unwrap!(file.read_to_string(&mut text).map_err(|e| Failed.cause(e)));

    let mut pp = Preprocessor::new(Lexer::new(text));
    if let Some(libs) = matches.values_of("ERL_LIBS") {
        for dir in libs {
            pp.code_paths_mut().push_back(dir.into());
        }
    }

    let reader = &mut erl_parse::TokenReader::new(&mut pp);
    let module = track_try_unwrap!(erl_parse::builtin::parse_module(reader));
    println!("{:?}", module);
}
