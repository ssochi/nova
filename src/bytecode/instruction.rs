#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub package_name: String,
    pub entry_function: String,
    pub local_names: Vec<String>,
    pub instructions: Vec<Instruction>,
}

impl Program {
    pub fn render(&self) -> String {
        let mut lines = vec![
            format!("package: {}", self.package_name),
            format!("entry: {}", self.entry_function),
            format!("locals: {}", self.local_names.join(", ")),
            "instructions:".to_string(),
        ];

        for (index, instruction) in self.instructions.iter().enumerate() {
            lines.push(format!("  {:>3}: {}", index, instruction.render()));
        }

        format!("{}\n", lines.join("\n"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    PushInt(i64),
    LoadLocal(usize),
    StoreLocal(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Pop,
    CallBuiltin(Builtin, usize),
    Return,
}

impl Instruction {
    pub fn render(&self) -> String {
        match self {
            Instruction::PushInt(value) => format!("push-int {value}"),
            Instruction::LoadLocal(index) => format!("load-local {index}"),
            Instruction::StoreLocal(index) => format!("store-local {index}"),
            Instruction::Add => "add".to_string(),
            Instruction::Subtract => "subtract".to_string(),
            Instruction::Multiply => "multiply".to_string(),
            Instruction::Divide => "divide".to_string(),
            Instruction::Pop => "pop".to_string(),
            Instruction::CallBuiltin(builtin, arity) => {
                format!("call-builtin {} {arity}", builtin.render())
            }
            Instruction::Return => "return".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Println,
}

impl Builtin {
    pub fn render(&self) -> &'static str {
        match self {
            Builtin::Println => "println",
        }
    }
}
