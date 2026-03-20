use crate::builtin::BuiltinFunction;
use crate::conversion::ConversionKind;
use crate::package::PackageFunction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Byte,
    Bool,
    String,
    Any,
    Slice(Box<ValueType>),
    Chan(Box<ValueType>),
    Map {
        key: Box<ValueType>,
        value: Box<ValueType>,
    },
}

impl ValueType {
    pub fn render(&self) -> String {
        match self {
            ValueType::Int => "int".to_string(),
            ValueType::Byte => "byte".to_string(),
            ValueType::Bool => "bool".to_string(),
            ValueType::String => "string".to_string(),
            ValueType::Any => "any".to_string(),
            ValueType::Slice(element) => format!("[]{}", element.render()),
            ValueType::Chan(element) => format!("chan {}", element.render()),
            ValueType::Map { key, value } => format!("map[{}]{}", key.render(), value.render()),
        }
    }

    pub fn map_value_type(&self) -> Option<&ValueType> {
        match self {
            ValueType::Map { value, .. } => Some(value.as_ref()),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceKind {
    Slice,
    String,
}

impl SequenceKind {
    pub fn render(self) -> &'static str {
        match self {
            SequenceKind::Slice => "slice",
            SequenceKind::String => "string",
        }
    }
}

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
    pub variadic_element_type: Option<ValueType>,
    pub return_types: Vec<ValueType>,
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
            let parameter_description = match &function.variadic_element_type {
                Some(element_type) => {
                    format!(
                        "{} + ...{}",
                        function.parameter_count - 1,
                        element_type.render()
                    )
                }
                None => function.parameter_count.to_string(),
            };
            lines.push(format!(
                "function {}: {} (params={}, returns={}, locals={})",
                function_index,
                function.name,
                parameter_description,
                if function.return_types.is_empty() {
                    "void".to_string()
                } else {
                    function
                        .return_types
                        .iter()
                        .map(ValueType::render)
                        .collect::<Vec<_>>()
                        .join(", ")
                },
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
    PushByte(u8),
    PushBool(bool),
    PushString(String),
    PushNilInterface,
    PushNilSlice,
    PushNilChan,
    PushNilMap,
    BuildSlice(usize),
    BuildMap {
        map_type: ValueType,
        entry_count: usize,
    },
    MakeSlice {
        element_type: ValueType,
        has_capacity: bool,
    },
    MakeChan {
        element_type: ValueType,
        has_buffer: bool,
    },
    MakeMap {
        map_type: ValueType,
        has_hint: bool,
    },
    Convert(ConversionKind),
    BoxAny(ValueType),
    TypeAssert(ValueType),
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
    Index(SequenceKind),
    Slice {
        target_kind: SequenceKind,
        has_low: bool,
        has_high: bool,
    },
    Receive(ValueType),
    IndexMap(ValueType),
    LookupMap(ValueType),
    MapKeys(ValueType),
    SetIndex,
    SetMapIndex,
    Send,
    Jump(usize),
    JumpIfFalse(usize),
    Pop,
    CallBuiltin(BuiltinFunction, usize),
    CallBuiltinSpread(BuiltinFunction, usize),
    Panic(ValueType),
    PanicNil,
    CallPackage(PackageFunction, usize),
    CallPackageSpread(PackageFunction, usize),
    CallFunction(usize, usize),
    CallFunctionSpread(usize, usize),
    DeferBuiltin(BuiltinFunction, usize),
    DeferPanic(ValueType),
    DeferPanicNil,
    DeferPackage(PackageFunction, usize),
    DeferPackageSpread(PackageFunction, usize),
    DeferFunction(usize, usize),
    DeferFunctionSpread(usize, usize),
    Return,
}

impl Instruction {
    pub fn render(&self) -> String {
        match self {
            Instruction::PushInt(value) => format!("push-int {value}"),
            Instruction::PushByte(value) => format!("push-byte {value}"),
            Instruction::PushBool(value) => format!("push-bool {value}"),
            Instruction::PushString(value) => {
                format!("push-string {}", render_string_literal(value))
            }
            Instruction::PushNilInterface => "push-nil-interface".to_string(),
            Instruction::PushNilSlice => "push-nil-slice".to_string(),
            Instruction::PushNilChan => "push-nil-chan".to_string(),
            Instruction::PushNilMap => "push-nil-map".to_string(),
            Instruction::BuildSlice(count) => format!("build-slice {count}"),
            Instruction::BuildMap {
                map_type,
                entry_count,
            } => format!("build-map {} {}", map_type.render(), entry_count),
            Instruction::MakeSlice {
                element_type,
                has_capacity,
            } => {
                format!(
                    "make-slice {} cap={}",
                    element_type.render(),
                    if *has_capacity { "explicit" } else { "len" }
                )
            }
            Instruction::MakeChan {
                element_type,
                has_buffer,
            } => {
                format!(
                    "make-chan {} buffer={}",
                    element_type.render(),
                    if *has_buffer { "explicit" } else { "none" }
                )
            }
            Instruction::MakeMap { map_type, has_hint } => {
                format!(
                    "make-map {} hint={}",
                    map_type.render(),
                    if *has_hint { "explicit" } else { "none" }
                )
            }
            Instruction::Convert(conversion) => format!("convert {}", conversion.render()),
            Instruction::BoxAny(value_type) => format!("box-any {}", value_type.render()),
            Instruction::TypeAssert(value_type) => {
                format!("type-assert {}", value_type.render())
            }
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
            Instruction::Index(target_kind) => format!("index {}", target_kind.render()),
            Instruction::Slice {
                target_kind,
                has_low,
                has_high,
            } => {
                format!(
                    "slice {} low={} high={}",
                    target_kind.render(),
                    has_low,
                    has_high
                )
            }
            Instruction::Receive(element_type) => format!("receive {}", element_type.render()),
            Instruction::IndexMap(map_type) => format!("index-map {}", map_type.render()),
            Instruction::LookupMap(map_type) => format!("lookup-map {}", map_type.render()),
            Instruction::MapKeys(key_type) => format!("map-keys {}", key_type.render()),
            Instruction::SetIndex => "set-index".to_string(),
            Instruction::SetMapIndex => "set-map-index".to_string(),
            Instruction::Send => "send".to_string(),
            Instruction::Jump(target) => format!("jump {target}"),
            Instruction::JumpIfFalse(target) => format!("jump-if-false {target}"),
            Instruction::Pop => "pop".to_string(),
            Instruction::CallBuiltin(builtin, arity) => {
                format!("call-builtin {} {arity}", builtin.render())
            }
            Instruction::CallBuiltinSpread(builtin, prefix_arity) => {
                format!("call-builtin-spread {} {prefix_arity}", builtin.render())
            }
            Instruction::Panic(value_type) => format!("panic {}", value_type.render()),
            Instruction::PanicNil => "panic-nil".to_string(),
            Instruction::CallPackage(function, arity) => {
                format!("call-package {} {arity}", function.render())
            }
            Instruction::CallPackageSpread(function, prefix_arity) => {
                format!("call-package-spread {} {prefix_arity}", function.render())
            }
            Instruction::CallFunction(index, arity) => format!("call-function {index} {arity}"),
            Instruction::CallFunctionSpread(index, prefix_arity) => {
                format!("call-function-spread {index} {prefix_arity}")
            }
            Instruction::DeferBuiltin(builtin, arity) => {
                format!("defer-builtin {} {arity}", builtin.render())
            }
            Instruction::DeferPanic(value_type) => {
                format!("defer-panic {}", value_type.render())
            }
            Instruction::DeferPanicNil => "defer-panic-nil".to_string(),
            Instruction::DeferPackage(function, arity) => {
                format!("defer-package {} {arity}", function.render())
            }
            Instruction::DeferPackageSpread(function, prefix_arity) => {
                format!("defer-package-spread {} {prefix_arity}", function.render())
            }
            Instruction::DeferFunction(index, arity) => format!("defer-function {index} {arity}"),
            Instruction::DeferFunctionSpread(index, prefix_arity) => {
                format!("defer-function-spread {index} {prefix_arity}")
            }
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
