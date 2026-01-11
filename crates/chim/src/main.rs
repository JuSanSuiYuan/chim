use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use chim_lexer::{tokenize, TokenStream};
use chim_parser::parse;
use chim_semantic::SemanticAnalyzer;
use chim_codegen::{CodeGen, CodegenTarget, GeneratedCode};
use chim_span::{SourceMap, FileId, Span};
use chim_error::ErrorReporter;

#[derive(Parser, Debug)]
#[command(name = "chim")]
#[command(author = "Chim Team")]
#[command(version = "0.1.0")]
#[command(about = "Chim Compiler - A modern programming language with Chinese keywords", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long, default_value = "c")]
    target: String,

    #[arg(short, long, default_value = "1")]
    opt_level: u32,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    verbose: bool,

    #[arg(long)]
    dump_ast: bool,

    #[arg(long)]
    dump_ir: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
        #[arg()]
        input: String,
    },
    Run {
        #[arg()]
        input: String,
    },
    Check {
        #[arg()]
        input: String,
    },
    Lsp,
}

fn main() {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Compile { input } => {
                compile_file(&input, &args.target, args.opt_level, args.output.as_deref(), args.verbose);
            }
            Commands::Run { input } => {
                eprintln!("Run command not yet implemented");
                std::process::exit(1);
            }
            Commands::Check { input } => {
                check_file(&input, args.verbose);
            }
            Commands::Lsp => {
                eprintln!("LSP server not yet implemented");
                std::process::exit(1);
            }
        }
    } else if let Some(input) = args.input {
        compile_file(&input, &args.target, args.opt_level, args.output.as_deref(), args.verbose);
    } else {
        print_usage();
    }
}

fn print_usage() {
    println!("Chim Compiler v0.1.0");
    println!();
    println!("Usage:");
    println!("  chim compile <input.chim> [options]");
    println!("  chim run <input.chim>");
    println!("  chim check <input.chim>");
    println!();
    println!("Options:");
    println!("  -t, --target <target>    Target language/backend (default: c)");
    println!("  -O, --opt-level <level>  Optimization level 0-3 (default: 1)");
    println!("  -o, --output <file>      Output file");
    println!("  -v, --verbose            Verbose output");
    println!("  --dump-ast               Print AST");
    println!("  --dump-ir                Print IR");
    println!();
    println!("Number Literals:");
    println!("  Decimal: 42, 1_000_000");
    println!("  Hexadecimal: 0xFF, 0XFF");
    println!("  Binary: 0b1010, 0B1010");
    println!("  Octal: 0o755, 0O755");
    println!("  Ternary: 0t120, 0T120 (digits: 0, 1, 2)");
    println!("  Balanced Ternary: 0e1-0, 0E1-0 (digits: -, 0, 1)");
    println!("  Duodecimal (12): 0d10, 0D10 (digits: 0-9, a, b)");
    println!("  Tetravigesimal (24): 0h10, 0H10 (digits: 0-9, a-n)");
    println!("  Sexagesimal (60): 0s10, 0S10 (digits: 0-9, a-z)");
    println!();
    println!("Supported targets:");
    println!("  Core (4): native, llvm, cranelift, qbe");
    println!("  Systems (8): c, cpp, rust, go, zig, d, nim, v");
    println!("  Web (4): wasm, js, ts, rust");
    println!("  JVM (3): java, kotlin, scala");
    println!("  .NET (2): cs, fsharp");
    println!("  Mobile (3): swift, objc, kotlin");
    println!("  Scripting (8): python, ruby, lua, php, perl, js, lua, julia");
    println!("  Scientific (4): r, matlab, julia, python");
    println!("  Functional (5): haskell, scala, clojure, erlang, elixir");
    println!("  GPU (6): cuda, vulkan, metal, opencl, glsl, hlsl");
    println!("  Assemblers (5): asm, nasm, masm, gas, llvm");
    println!();
    println!("  Total: 61+ targets!");
}

fn compile_file(input_file: &str, target: &str, opt_level: u32, output_file: Option<&str>, verbose: bool) {
    if verbose {
        println!("Chim Compiler v0.1.0");
        println!("Input: {}", input_file);
        println!("Target: {}", target);
        println!("Optimization: {}", opt_level);
        println!();
    }

    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(Arc::from(input_file), Arc::from(&source));

    if verbose {
        println!("Source file: {} ({} bytes)", input_file, source.len());
        println!();
    }

    let mut interner = lasso::Rodeo::new();
    let tokens = chim_lexer::tokenize(&source, &mut interner, file_id);

    if verbose {
        println!("Lexing: {} tokens", tokens.len());
    }

    let ast = match parse(&source, file_id) {
        Ok(ast) => {
            if verbose {
                println!("Parsing: OK ({} items)", ast.items.len());
            }
            if args.dump_ast {
                println!("\n=== AST ===");
                println!("{:#?}", ast);
            }
            ast
        }
        Err(errors) => {
            eprintln!("Parsing failed with {} errors:", errors.len());
            for error in errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    let analyzed_program = match analyzer.analyze(&ast) {
        Ok(program) => {
            if verbose {
                println!("Semantic analysis: OK");
                println!("  Types: {}", program.pool.type_count());
                println!("  Structs: {}", program.pool.struct_count());
                println!("  Enums: {}", program.pool.enum_count());
            }
            program
        }
        Err(errors) => {
            eprintln!("Semantic analysis failed with {} errors:", errors.len());
            for error in errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    };

    let ir_module = chim_ir::generate_ir(&ast, &analyzed_program);

    let codegen = CodeGen::new();
    let generated_code = match codegen.generate(&ir_module, &analyzed_program, target) {
        Ok(code) => {
            if verbose {
                println!("Code generation: OK ({} bytes)", code.source.len());
            }
            code
        }
        Err(e) => {
            eprintln!("Code generation failed: {}", e);
            std::process::exit(1);
        }
    };

    let output_path = output_file.unwrap_or_else(|| {
        let stem = PathBuf::from(input_file).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = generated_code.extension;
        format!("{}.{}", stem, ext)
    });

    match fs::write(&output_path, &generated_code.source) {
        Ok(_) => {
            if verbose {
                println!("Output written to: {}", output_path);
            } else {
                println!("Generated: {}", output_path);
            }
        }
        Err(e) => {
            eprintln!("Error writing output: {}", e);
            std::process::exit(1);
        }
    }

    if args.dump_ir {
        println!("\n=== IR ===");
        println!("IR generation not yet implemented");
    }
}

fn check_file(input_file: &str, verbose: bool) {
    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(Arc::from(input_file), Arc::from(&source));

    let ast = match parse(&source, file_id) {
        Ok(ast) => ast,
        Err(errors) => {
            eprintln!("Parsing failed with {} errors:", errors.len());
            for error in errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(&ast) {
        Ok(_) => {
            if verbose {
                println!("Type checking: OK");
            } else {
                println!("OK");
            }
        }
        Err(errors) => {
            eprintln!("Type checking failed with {} errors:", errors.len());
            for error in errors {
                eprintln!("  {}", error);
            }
            std::process::exit(1);
        }
    }
}

fn args() -> Args {
    Args::parse()
}
