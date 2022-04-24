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

fn output_data_string(data_strings: &mut Vec<String>, gen_file: &mut fs::File,) -> std::io::Result<()> {
    
    gen_file.write_all("\t.data\n".as_bytes())?;
    for i in 0..data_strings.len() {
        gen_file.write_all(data_strings[i].as_bytes())?;
    }
    
    Ok(())
}

fn output_function(
    func: pest::iterators::Pair<Rule>,
    gen_file: &mut fs::File,
    data_strings: &mut Vec<String>,
    pc: &mut u64,
) -> std::io::Result<()> {
    let mut func_inner_rules = func.into_inner();
    
    // let registers = ["$t0", "$t1", "$t2", "$t3", "$t4", "$t5", "$t6", "$t7", "$t8", "$t9", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$s8", "$s9"];
    let registers = ["$t0", "$t1", "$s0", "$s1"];

    func_inner_rules.next(); // fn
    let func_name: &str = func_inner_rules.next().unwrap().as_str(); // function name
    func_inner_rules.next(); // (
    let arg_list = func_inner_rules.next().unwrap(); // arguments
    func_inner_rules.next(); // )
    func_inner_rules.next(); // {

    gen_file.write_all(format!("__func_{}:\n", func_name).as_bytes())?;

    let mut args = Vec::new();
    let mut strings_in_func = 0;

    for arg in arg_list.into_inner() {
        match arg.as_rule() {
            Rule::fd_arg => {
                let mut inner_rules = arg.into_inner();
                let arg_name = inner_rules.next().unwrap().as_str();
                inner_rules.next();
                let arg_type = inner_rules.next().unwrap().as_str();

                args.push((
                    arg_name,
                    arg_type,
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
            if args[i].0.eq(args[j].0) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Found duplicated argument name in {}: {}",
                        func_name, args[i].0
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
                            format!("\tdecl {}\n", inner_rules.next().unwrap().as_str()).as_bytes()
                        )?;
                        
                        match inner_rules.peek().unwrap().as_rule() {
                            Rule::expr => {
                                let mut temp1 = inner_rules.next().unwrap().into_inner();
                                match temp1.peek().unwrap().as_rule() {
                                    Rule::string_literal => {
                                        let temp : String = temp1.next().unwrap().as_str().to_string();
                                        data_strings.push(format!("{}-strlit-{}:\n\tasciiz \"{}\"\n", func_name, strings_in_func, temp[1..temp.len()-1].to_string()));
                                        strings_in_func += 1;
                                    }
                                    _ => {
                                    }
                                }
                            }
                            _ => {
                            }
                        }
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
                        let fc_args = fc.next().unwrap();
                        let fc_arg_strs = fc_args.as_str();
                        let fc_args = fc_args.into_inner();
                        let mut fc_arg_count = 0;
                        
                        for arg in fc_args.clone() {
                            match arg.as_rule() {
                                Rule::fc_arg => {
                                    // println!("arg {}", arg.as_str());
                                    fc_arg_count += 1;
                                },
                                _ => {}
                            }
                        }
                        
                        let frame_size = registers.len() * 4 + fc_arg_count * 4 + 8;
                        let mut bytes = 0;
                        
                        /*
                        
                        
                        sub $sp, $sp, frame_size      # set new stack pointer
                        # store all the arguments
                        # store all the sX and tX registers (b bytes)
                        sw $ra, 8($sp)      # save return address in stack
                        sw $fp, 4($sp)      # save old frame pointer in stack
                        add $fp, $sp, b + 8      # set new frame pointer
                        
                        */
                        // sub $sp, $sp, frame_size      # set new stack pointer
                        gen_file.write_all(
                            format!("\tsub $sp, $sp, {}\n", frame_size)
                                .as_bytes(),
                        )?;
                        // store all the arguments
                        for arg in fc_args {
                            match arg.as_rule() {
                                Rule::fc_arg => {
                                    gen_file.write_all(
                                        format!("\tsw $sp, {}(%expr value% {})\n", frame_size - bytes, arg.as_str())
                                            .as_bytes(),
                                    )?;
                                    bytes += 4;
                                },
                                _ => {}
                            }
                        }
                        // store all the sX and tX registers
                        for reg in &registers {
                            gen_file.write_all(
                                format!("\tsw $sp, {}({})\n", frame_size - bytes, reg)
                                    .as_bytes(),
                            )?;
                            bytes += 4;
                        }
                        // sw $ra, 8($sp)           # save return address in stack
                        // sw $fp, 4($sp)           # save old frame pointer in stack
                        // add $fp, $sp, b + 8      # set new frame pointer
                        gen_file.write_all(
                            format!("\tsw $ra, 8($sp)\n\tsw $fp, 4($sp)\n\tadd $fp, $sp, {}\n\tjal __func_{} ( {} )\n\tsub $fp, $sp, {}\n\tlw $fp, 4($sp)\n\tlw $ra, 8($sp)\n", frame_size, fc_name, fc_arg_strs, frame_size)
                                .as_bytes(),
                        )?;
                        // store all the sX and tX registers
                        for reg in &registers {
                            gen_file.write_all(
                                format!("\tlw {}, {}($sp)\n", reg, frame_size - bytes)
                                    .as_bytes(),
                            )?;
                            bytes -= 4;
                        }
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

fn print_tree(top: pest::iterators::Pair<Rule>, indent: u32) {
    for _ in 0..indent {
        print!("  ");
    }
    println!("{:?}\t{}", top.as_rule(), top.as_str());
    for rule in top.into_inner() {
        print_tree(rule, indent + 1);
    }
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
    
    let mut data_strings: Vec<String> = vec![];
    
    println!("Printing parsed tree from {}", file_string.clone());
    print_tree(file.clone(), 0);
    println!("\n\n\n");
    
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
                match output_function(i, &mut gen_file, &mut data_strings, &mut pc) {
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
    
    output_data_string(&mut data_strings, &mut gen_file).unwrap();

    println!("FINISHED ANALYSIS");

    Ok(())
}
