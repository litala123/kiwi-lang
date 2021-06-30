use std::env;
use std::process;
#[cfg(test)]
mod tests;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub parser);

enum ArgParseStates {
    General,
    OutputFile,
}

fn main() {
    println!("Kiwi Compiler\n");

    let args: Vec<String> = env::args().collect();
    let mut out_file = "build.kasm";

    if args.len() == 1 {
        eprintln!("Error: not enough arguments.");
        process::exit(1);
    }

    let mut arg_parse_state = ArgParseStates::General;
    let mut sources = vec![];

    for arg in &args[1..args.len()] {
        match arg_parse_state {
            ArgParseStates::General => {
                // println!("General State {:?}", arg);

                match arg.as_str() {
                    "-o" => {
                        arg_parse_state = ArgParseStates::OutputFile;
                        // println!("\tMoving to the OutputFile State\n");
                        continue;
                    },
                    _ => {
                        sources.push(arg);
                        // println!("\tAdded \"{}\" to source list\n", arg);
                        continue;
                    }
                }
            },
            ArgParseStates::OutputFile => {
                // println!("OutputFile State {:?}", arg);
                out_file = arg;
                arg_parse_state = ArgParseStates::General;
                // println!("\tMoving to the General State\n");
                continue;
            },
        }
    }
    
    print!("Source files:\n");
    for s in sources {
        print!("\t{}\n", s);
    }

    println!("\nOutput file:\n\t{}\n", out_file);
    
    // parser
    let res = parser::I32LiteralParser::new().parse("123");
    match res {
        Ok(_) => println!("Parsing completed with no errors"),
        Err(_) => println!("There was a parsing error"),
    }
}
