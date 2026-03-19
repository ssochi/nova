use std::fmt;

use crate::bytecode::compiler::compile_program;
use crate::bytecode::instruction::Program;
use crate::cli::{self, Command};
use crate::frontend::lexer::lex;
use crate::frontend::parser::parse_source_file;
use crate::runtime::vm::VirtualMachine;
use crate::source::SourceFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    Usage(String),
    Io(String),
    Lex(String),
    Parse(String),
    Compile(String),
    Runtime(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::Usage(message)
            | DriverError::Io(message)
            | DriverError::Lex(message)
            | DriverError::Parse(message)
            | DriverError::Compile(message)
            | DriverError::Runtime(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DriverError {}

pub fn run_cli<I>(args: I) -> Result<String, DriverError>
where
    I: IntoIterator<Item = String>,
{
    let command = cli::parse(args).map_err(DriverError::Usage)?;
    execute(command)
}

pub fn execute(command: Command) -> Result<String, DriverError> {
    match command {
        Command::Check { path } => {
            let source = load_source(&path)?;
            let tokens = lex(&source).map_err(|error| DriverError::Lex(error.to_string()))?;
            parse_source_file(&tokens).map_err(|error| DriverError::Parse(error.to_string()))?;
            Ok(format!("ok: {}\n", source.path.display()))
        }
        Command::DumpTokens { path } => {
            let source = load_source(&path)?;
            let tokens = lex(&source).map_err(|error| DriverError::Lex(error.to_string()))?;
            let output = tokens
                .iter()
                .map(|token| token.render())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(format!("{output}\n"))
        }
        Command::DumpAst { path } => {
            let source = load_source(&path)?;
            let ast = parse(&source)?;
            Ok(ast.render())
        }
        Command::DumpBytecode { path, config } => {
            let source = load_source(&path)?;
            let ast = parse(&source)?;
            let program =
                compile_program(&ast, &config).map_err(|error| DriverError::Compile(error.to_string()))?;
            Ok(program.render())
        }
        Command::Run { path, config } => {
            let source = load_source(&path)?;
            let ast = parse(&source)?;
            let program =
                compile_program(&ast, &config).map_err(|error| DriverError::Compile(error.to_string()))?;
            run_program(&program)
        }
    }
}

fn load_source(path: &std::path::Path) -> Result<SourceFile, DriverError> {
    SourceFile::load(path).map_err(|error| {
        DriverError::Io(format!("failed to read {}: {error}", path.display()))
    })
}

fn parse(source: &SourceFile) -> Result<crate::frontend::ast::SourceFileAst, DriverError> {
    let tokens = lex(source).map_err(|error| DriverError::Lex(error.to_string()))?;
    parse_source_file(&tokens).map_err(|error| DriverError::Parse(error.to_string()))
}

fn run_program(program: &Program) -> Result<String, DriverError> {
    let mut vm = VirtualMachine::new(program.local_names.len());
    vm.execute(program)
        .map(|result| result.render_output())
        .map_err(|error| DriverError::Runtime(error.to_string()))
}
