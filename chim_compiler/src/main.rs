use std::env;
use std::fs;
use std::process;

use logos::Logos;

mod ast;
mod lexer;
mod parser;
mod semantic;
mod ir;
mod codegen;
mod wasm_codegen;
mod optimizer;

use codegen::CodeGenerator;
use wasm_codegen::TargetCodeGenerator;
use optimizer::Optimizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.chim> [target] [--opt <level>]", args[0]);
        eprintln!("  Targets: wasm (default), native, ir");
        eprintln!("  Optimization levels: 0 (none), 1 (basic), 2 (aggressive)");
        process::exit(1);
    }

    let input_file = &args[1];
    let mut target = args.get(2).map(|s| s.as_str()).unwrap_or("wasm");
    let mut opt_level: u32 = 1;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--opt" if i + 1 < args.len() => {
                opt_level = args[i + 1].parse().unwrap_or(1);
                i += 2;
            },
            "--opt=0" | "-O0" => {
                opt_level = 0;
                i += 1;
            },
            "--opt=1" | "-O1" => {
                opt_level = 1;
                i += 1;
            },
            "--opt=2" | "-O2" => {
                opt_level = 2;
                i += 1;
            },
            _ if args[i].starts_with("--") => {
                i += 1;
            },
            _ if target == "wasm" || target == "native" || target == "ir" => {
                if args[i] != "wasm" && args[i] != "native" && args[i] != "ir" {
                    target = "wasm";
                } else {
                    target = args[i].as_str();
                }
                i += 1;
            },
            _ => {
                i += 1;
            },
        }
    }
    
    println!("Chim Compiler - Optimization Level: {}", opt_level);
    println!("Target: {}", target);
    println!("---");
    
    let source = match fs::read_to_string(input_file) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("Error reading file {}: {}", input_file, err);
            process::exit(1);
        },
    };

    // 词法分析
    let mut lex = lexer::Token::lexer(&source);
    let mut tokens = Vec::new();
    while let Some(token) = lex.next() {
        match token {
            Ok(token) => {
                tokens.push(token);
            },
            Err(_) => {
                eprintln!("Lexical error at position {}", lex.span().start);
                process::exit(1);
            },
        }
    }

    // 语法分析
    let mut parser = parser::Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("Successfully parsed program:");
            println!("{}", ast);
            
            // 语义分析
            let mut analyzer = semantic::SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(_) => {
                    println!("Semantic analysis completed successfully!");
                },
                Err(errors) => {
                    eprintln!("Semantic errors found:");
                    for error in errors {
                        eprintln!("{}", error);
                    }
                    process::exit(1);
                },
            }
            
            // 代码生成
            println!("\nGenerating code for target: {}", target);
            
            match target {
                "wasm" => {
                    let mut ir_gen = codegen::IRGenerator::new();
                    let mut module = ir_gen.generate_module(&ast);
                    
                    let mut optimizer = Optimizer::new(opt_level);
                    optimizer.optimize_module(&mut module);
                    
                    let wasm_gen = wasm_codegen::WASMGenerator::new();
                    let wasm_code = wasm_gen.generate(&module);
                    
                    // 输出WASM代码
                    let output_file = input_file.replace(".chim", ".wat");
                    match fs::write(&output_file, &wasm_code) {
                        Ok(_) => {
                            println!("Generated WebAssembly text format: {}", output_file);
                            println!("\nWASM output preview:");
                            println!("{}", &wasm_code[..wasm_code.len().min(1000)]);
                        },
                        Err(err) => {
                            eprintln!("Error writing output file: {}", err);
                            process::exit(1);
                        },
                    }
                },
                "native" => {
                    let mut ir_gen = codegen::IRGenerator::new();
                    let mut module = ir_gen.generate_module(&ast);
                    
                    let mut optimizer = Optimizer::new(opt_level);
                    optimizer.optimize_module(&mut module);
                    
                    let native_gen = wasm_codegen::NativeGenerator::default();
                    let native_code = native_gen.generate(&module);
                    
                    // 输出C代码
                    let output_file = input_file.replace(".chim", ".c");
                    match fs::write(&output_file, &native_code) {
                        Ok(_) => {
                            println!("Generated C code: {}", output_file);
                        },
                        Err(err) => {
                            eprintln!("Error writing output file: {}", err);
                            process::exit(1);
                        },
                    }
                },
                "ir" => {
                    // 只输出IR
                    let mut ir_gen = codegen::IRGenerator::new();
                    let module = ir_gen.generate_module(&ast);
                    
                    println!("\nGenerated IR:");
                    for func in &module.functions {
                        println!("Function: {}", func.name);
                        for param in &func.params {
                            println!("  Param: {} : {}", param.0, param.1);
                        }
                        for inst in &func.body {
                            println!("  {:?}", inst);
                        }
                    }
                },
                _ => {
                    eprintln!("Unknown target: {}", target);
                    eprintln!("Supported targets: wasm, native, ir");
                    process::exit(1);
                },
            }
        },
        Err(err) => {
            eprintln!("Syntax error: {}", err);
            process::exit(1);
        },
    }
}
