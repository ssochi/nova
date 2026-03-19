use std::path::PathBuf;

use crate::config::ExecutionConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Check {
        path: PathBuf,
    },
    Run {
        path: PathBuf,
        config: ExecutionConfig,
    },
    DumpTokens {
        path: PathBuf,
    },
    DumpAst {
        path: PathBuf,
    },
    DumpBytecode {
        path: PathBuf,
        config: ExecutionConfig,
    },
}

pub fn parse<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let binary_name = args.next().unwrap_or_else(|| "nova-go".to_string());
    let command = args.next().ok_or_else(|| usage(&binary_name))?;
    let rest: Vec<String> = args.collect();

    match command.as_str() {
        "check" => {
            let (path, _) = parse_path_and_config(&binary_name, &command, rest, false)?;
            Ok(Command::Check { path })
        }
        "run" => {
            let (path, config) = parse_path_and_config(&binary_name, &command, rest, true)?;
            Ok(Command::Run { path, config })
        }
        "dump-tokens" => {
            let (path, _) = parse_path_and_config(&binary_name, &command, rest, false)?;
            Ok(Command::DumpTokens { path })
        }
        "dump-ast" => {
            let (path, _) = parse_path_and_config(&binary_name, &command, rest, false)?;
            Ok(Command::DumpAst { path })
        }
        "dump-bytecode" => {
            let (path, config) = parse_path_and_config(&binary_name, &command, rest, true)?;
            Ok(Command::DumpBytecode { path, config })
        }
        "help" | "--help" | "-h" => Err(usage(&binary_name)),
        other => Err(format!(
            "unknown command `{other}`\n\n{}",
            usage(&binary_name)
        )),
    }
}

fn parse_path_and_config(
    binary_name: &str,
    command_name: &str,
    args: Vec<String>,
    allow_config: bool,
) -> Result<(PathBuf, ExecutionConfig), String> {
    let mut config = ExecutionConfig::default();
    let mut path = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--entry-package" if allow_config => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    format!(
                        "missing value for `--entry-package`\n\n{}",
                        usage(binary_name)
                    )
                })?;
                config.entry_package = value.clone();
            }
            "--entry-function" if allow_config => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    format!(
                        "missing value for `--entry-function`\n\n{}",
                        usage(binary_name)
                    )
                })?;
                config.entry_function = value.clone();
            }
            "--entry-package" | "--entry-function" => {
                return Err(format!(
                    "command `{command_name}` does not accept execution overrides\n\n{}",
                    usage(binary_name)
                ));
            }
            value if value.starts_with('-') => {
                return Err(format!(
                    "unknown option `{value}`\n\n{}",
                    usage(binary_name)
                ));
            }
            value => {
                if path.replace(PathBuf::from(value)).is_some() {
                    return Err(format!(
                        "multiple input files provided\n\n{}",
                        usage(binary_name)
                    ));
                }
            }
        }
        index += 1;
    }

    let path = path.ok_or_else(|| usage(binary_name))?;
    Ok((path, config))
}

fn usage(binary_name: &str) -> String {
    format!(
        "Usage:\n  {binary_name} check <file>\n  {binary_name} run <file> [--entry-package <name>] [--entry-function <name>]\n  {binary_name} dump-tokens <file>\n  {binary_name} dump-ast <file>\n  {binary_name} dump-bytecode <file> [--entry-package <name>] [--entry-function <name>]"
    )
}
