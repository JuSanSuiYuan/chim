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
mod memory_layout;
mod group_manager;
mod allocation;
mod rvo;
mod backend;
mod backends;

use codegen::CodeGenerator;
use wasm_codegen::TargetCodeGenerator;
use optimizer::Optimizer;
use rvo::RVOOptimizer;
use backend::{BackendType, create_backend};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input.chim> [-t <target>] [-O <level>]", args[0]);
        eprintln!("\n核心后端 (8): wasm, native, llvm, qbe, tinycc, cranelift, fortran, asm");
        eprintln!("工业级 (8): clang, flang, java, js, ts, cs, v, nim");
        eprintln!("移动平台 (3): kotlin, swift, objc");
        eprintln!("编译器工具链 (11): 8cc, gcc, rustc, zig, ucc, selfie, 9cc, pgi, msvc, compcert, lcc");
        eprintln!("最新添加 (1): chibicc");
        eprintln!("\nGPU后端 (6): cuda, vulkan, metal, opencl, mojo, tilelang");
        eprintln!("  ★ TileLang: 国产AI编程语言（北大杨智团队，DeepSeek v3.2内核）");
        eprintln!("  ★ 支持CUDA和国产算力芯片（昇腾、寒武纪等）");
        eprintln!("\n  Optimization levels: 0 (none), 1 (basic), 2 (aggressive)");
        eprintln!("\nExamples:");
        eprintln!("  {} test.chim -t tilelang -O 2  # 🇨🇳 国产TileLang（推荐）", args[0]);
        eprintln!("  {} test.chim -t cuda -O 2      # NVIDIA GPU", args[0]);
        eprintln!("  {} test.chim -t vulkan -O 2    # 跨平台GPU", args[0]);
        eprintln!("  {} test.chim -t mojo -O 2      # AI原生", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    let mut target = "wasm";
    let mut opt_level: u32 = 1;
    let mut output_file: Option<String> = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--target" | "-b" | "--backend" if i + 1 < args.len() => {
                target = &args[i + 1];
                i += 2;
            },
            "-o" | "--output" if i + 1 < args.len() => {
                output_file = Some(args[i + 1].clone());
                i += 2;
            },
            "--opt" if i + 1 < args.len() => {
                opt_level = args[i + 1].parse().unwrap_or(1);
                i += 2;
            },
            "-O0" | "--opt=0" => {
                opt_level = 0;
                i += 1;
            },
            "-O1" | "--opt=1" => {
                opt_level = 1;
                i += 1;
            },
            "-O2" | "--opt=2" => {
                opt_level = 2;
                i += 1;
            },
            "-O" if i + 1 < args.len() => {
                opt_level = args[i + 1].parse().unwrap_or(1);
                i += 2;
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
            println!("\n生成代码，目标后端: {}", target);
            
            // 生成IR
            let mut ir_gen = codegen::IRGenerator::new();
            let mut module = ir_gen.generate_module(&ast);
            
            // 应用优化
            let mut optimizer = Optimizer::new(opt_level);
            optimizer.optimize_module(&mut module);
            
            // 应用RVO优化
            let mut rvo = RVOOptimizer::new();
            rvo.optimize_module(&mut module);
            
            match target {
                "ir" => {
                    // 只输出IR
                    println!("\n生成的IR:");
                    for func in &module.functions {
                        println!("Function: {}", func.name);
                        for param in &func.params {
                            println!("  Param: {} : {:?}", param.0, param.1);
                        }
                        for inst in &func.body {
                            println!("  {:?}", inst);
                        }
                    }
                },
                _ => {
                    // 使用新的后端架构
                    if let Some(backend_type) = BackendType::from_str(target) {
                        let backend = create_backend(backend_type);
                        println!("使用后端: {}", backend.name());
                        
                        match backend.generate(&module) {
                            Ok(code) => {
                                let ext = backend.file_extension();
                                let output_path = if let Some(ref out) = output_file {
                                    out.clone()
                                } else {
                                    input_file.replace(".chim", &format!(".{}", ext))
                                };
                                
                                match fs::write(&output_path, &code) {
                                    Ok(_) => {
                                        println!("生成代码成功: {}", output_path);
                                        println!("\n代码预览:");
                                        let preview = if code.len() > 1000 {
                                            &code[..1000]
                                        } else {
                                            &code
                                        };
                                        println!("{}", preview);
                                        if code.len() > 1000 {
                                            println!("\n... (还有 {} 字节)", code.len() - 1000);
                                        }
                                    },
                                    Err(err) => {
                                        eprintln!("写入输出文件失败: {}", err);
                                        process::exit(1);
                                    },
                                }
                            },
                            Err(err) => {
                                eprintln!("代码生成失败: {}", err);
                                process::exit(1);
                            },
                        }
                    } else {
                        eprintln!("未知的目标后端: {}", target);
                        eprintln!("支持的后端: wasm, native, llvm, qbe, tinycc, cranelift, fortran, asm, clang, flang, java, js, ts, cs, v, nim, kotlin, swift, objc, 8cc, gcc, rustc, zig, ucc, selfie, 9cc, pgi, msvc, compcert, lcc, chibicc, ir");
                        process::exit(1);
                    }
                },
            }
        },
        Err(err) => {
            eprintln!("Syntax error: {}", err);
            process::exit(1);
        },
    }
}
