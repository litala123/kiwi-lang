extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::process;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "lang.pest"]
pub struct MiniParser;

enum ArgParseStates {
    General,
    OutputFile,
}

fn output_import(
    stmt: pest::iterators::Pair<Rule>,
    gen_file: &mut fs::File,
) -> std::io::Result<()> {
    let mut inner_rules = stmt.into_inner();
    inner_rules.next(); // import
    let import_name_quotes: &str = inner_rules.next().unwrap().as_str();
    let import_name: &str = &import_name_quotes[1..(import_name_quotes.len() - 1)]; // imported file

    gen_file.write_all(format!("\t.include \"{}\"\n", import_name).as_bytes())?;

    Ok(())
}

fn output_function(
    func: pest::iterators::Pair<Rule>,
    gen_file: &mut fs::File,
    pc: &mut u64,
) -> std::io::Result<()> {
    let mut func_inner_rules = func.into_inner();

    func_inner_rules.next(); // fn
    let func_name: &str = func_inner_rules.next().unwrap().as_str(); // function name
    func_inner_rules.next(); // (
    let arg_list = func_inner_rules.next().unwrap(); // arguments
    func_inner_rules.next(); // )
    func_inner_rules.next(); // {

    gen_file.write_all(format!("__func_{}:\n", func_name).as_bytes())?;

    let mut args = Vec::new();

    for arg in arg_list.into_inner() {
        match arg.as_rule() {
            Rule::fd_arg => {
                let mut inner_rules = arg.into_inner();

                args.push((
                    inner_rules.next().unwrap().as_str(),
                    inner_rules.next().unwrap().as_str(),
                ) as (&str, &str));

                *pc += 4;
            }
            Rule::COMMA => (),
            _ => {
                println!("unreachable: {:?}", arg.as_rule());
            }
        }
    }

    for i in 0..args.len() {
        for j in i + 1..args.len() {
            if args[i].1.eq(args[j].1) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Found duplicated argument name in {}: {}",
                        func_name, args[i].1
                    ),
                ));
            }
        }
    }
    for i in 0..args.len() {
        gen_file.write_all(format!("\tsw ${}, {}($sp)\n", i+4, -4*((i as i16)+1)).as_bytes())?;
    }
    gen_file.write_all(format!("\tsub $sp, $sp, {}\n", 4 * args.len()).as_bytes())?;
    for i in func_inner_rules {
        match i.as_rule() {
            Rule::stmt => {
                let mut inner_rules = i.into_inner();

                match inner_rules.peek().unwrap().as_rule() {
                    Rule::LET_K => {
                        inner_rules.next();
                        gen_file.write_all(
                            format!("\tdecl {}\n", inner_rules.next().unwrap().as_str()).as_bytes(),
                        )?;
                    }
                    Rule::ident => {
                        gen_file.write_all(
                            format!("\tassign {}\n", inner_rules.next().unwrap().as_str())
                            .as_bytes(),
                        )?;
                    }
                    Rule::RET_K => {
                        inner_rules.next();
                        gen_file.write_all(
                            format!("\treturn {}\n", inner_rules.next().unwrap().as_str())
                                .as_bytes(),
                        )?;
                    }
                    Rule::func_call => {
                        let mut fc = inner_rules.next().unwrap().into_inner();
                        let fc_name = fc.next().unwrap().as_str();
                        fc.next();
                        let fc_args = fc.next().unwrap().as_str();
                        
                        gen_file.write_all(
                            format!("\tcall __func_{} ( {} )\n", fc_name, fc_args)
                                .as_bytes(),
                        )?;
                    }
                    Rule::EOI => (),
                    _ => unreachable!(),
                }
                *pc += 4;
            }
            Rule::RBRACK => (),
            Rule::EOI => (),
            _ => {
                println!("unreachable: {}", i.as_str());
            }
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut file_out = "build.kasm";

    if args.len() == 1 {
        eprintln!("Error: not enough arguments.");
        process::exit(1);
    }

    let mut arg_parse_state = ArgParseStates::General;
    let mut sources = vec![];
    
    // COMPILER OPTIONS PARSING

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
                file_out = arg;
                arg_parse_state = ArgParseStates::General;
                // println!("\tMoving to the General State\n");
                continue;
            },
        }
    }
    
    print!("Source files:\n");
    for s in &sources {
        print!("\t{}\n", s);
    }

    let file_in = &sources[0];
    
    // PARSING
    
    let mut gen_file = fs::File::create(file_out)?;

    let file_string = fs::read_to_string(file_in).expect(&format!("Unable to read {}", file_in));

    let file = MiniParser::parse(Rule::file, &file_string)
        .unwrap_or_else(|e| panic!("{}", e))
        .next()
        .unwrap();

    // CODE GENERATION
    
    let mut pc: u64 = 0;
    
    let mut in_text = false;

    for i in file.into_inner() {
        match i.as_rule() {
            Rule::import_stmt => {
                output_import(i, &mut gen_file)?;
            }
            Rule::func_decl => {
                if !in_text {
                    gen_file.write_all(b"\t.text\n")?;
                    in_text = true;
                }
                match output_function(i, &mut gen_file, &mut pc) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        return Ok(());
                    }
                }
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }

    println!("FINISHED ANALYSIS");

    Ok(())
}
