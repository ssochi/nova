use crate::builtin::BuiltinFunction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub package_name: String,
    pub entry_function: String,
    pub entry_function_index: usize,
    pub functions: Vec<CompiledFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledFunction {
    pub name: String,
    pub parameter_count: usize,
    pub returns_value: bool,
    pub local_names: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl Program {
    pub fn render(&self) -> String {
        let mut lines = vec![
            format!("package: {}", self.package_name),
            format!("entry: {}", self.entry_function),
            format!("entry-index: {}", self.entry_function_index),
        ];

        for (function_index, function) in self.functions.iter().enumerate() {
            lines.push(format!(
                "function {}: {} (params={}, returns={}, locals={})",
                function_index,
                function.name,
                function.parameter_count,
                if function.returns_value { "value" } else { "void" },
                function.local_names.join(", ")
            ));
            for (instruction_index, instruction) in function.instructions.iter().enumerate() {
                lines.push(format!(
                    "  {:>3}.{:>3}: {}",
                    function_index,
                    instruction_index,
                    instruction.render()
                ));
            }
        }

        format!("{}\n", lines.join("\n"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    PushInt(i64),
    PushBool(bool),
    PushString(String),
    LoadLocal(usize),
    StoreLocal(usize),
    Add,
    Concat,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Jump(usize),
    JumpIfFalse(usize),
    Pop,
    CallBuiltin(BuiltinFunction, usize),
    CallFunction(usize, usize),
    Return,
}

impl Instruction {
    pub fn render(&self) -> String {
        match self {
            Instruction::PushInt(value) => format!("push-int {value}"),
            Instruction::PushBool(value) => format!("push-bool {value}"),
            Instruction::PushString(value) => format!("push-string {}", render_string_literal(value)),
            Instruction::LoadLocal(index) => format!("load-local {index}"),
            Instruction::StoreLocal(index) => format!("store-local {index}"),
            Instruction::Add => "add".to_string(),
            Instruction::Concat => "concat".to_string(),
            Instruction::Subtract => "subtract".to_string(),
            Instruction::Multiply => "multiply".to_string(),
            Instruction::Divide => "divide".to_string(),
            Instruction::Equal => "equal".to_string(),
            Instruction::NotEqual => "not-equal".to_string(),
            Instruction::Less => "less".to_string(),
            Instruction::LessEqual => "less-equal".to_string(),
            Instruction::Greater => "greater".to_string(),
            Instruction::GreaterEqual => "greater-equal".to_string(),
            Instruction::Jump(target) => format!("jump {target}"),
            Instruction::JumpIfFalse(target) => format!("jump-if-false {target}"),
            Instruction::Pop => "pop".to_string(),
            Instruction::CallBuiltin(builtin, arity) => {
                format!("call-builtin {} {arity}", builtin.render())
            }
            Instruction::CallFunction(index, arity) => format!("call-function {index} {arity}"),
            Instruction::Return => "return".to_string(),
        }
    }
}

fn render_string_literal(value: &str) -> String {
    let mut rendered = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\t' => rendered.push_str("\\t"),
            '\r' => rendered.push_str("\\r"),
            other => rendered.push(other),
        }
    }
    rendered.push('"');
    rendered
}
