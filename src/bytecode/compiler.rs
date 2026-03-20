use std::fmt;

use crate::builtin::BuiltinFunction;
use crate::bytecode::instruction::{CompiledFunction, Instruction, Program, SequenceKind};
use crate::semantic::model::{
    CallTarget, CheckedBinaryOperator, CheckedBinding, CheckedBlock, CheckedCall,
    CheckedCallArguments, CheckedElseBranch, CheckedExpression, CheckedExpressionKind,
    CheckedForStatement, CheckedFunction, CheckedIfStatement, CheckedMapLiteralEntry,
    CheckedProgram, CheckedStatement, CheckedSwitchClause, CheckedSwitchStatement,
    CheckedValueSource, Type,
};

mod calls;
mod simple_statements;
mod types;

use self::types::{lower_result_types, lower_sequence_kind, lower_value_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    message: String,
}

impl CompileError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CompileError {}

pub fn compile_program(program: &CheckedProgram) -> Result<Program, CompileError> {
    let mut compiled_functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        compiled_functions.push(FunctionCompiler::new(function).compile()?);
    }

    Ok(Program {
        package_name: program.package_name.clone(),
        entry_function: program.functions[program.entry_function].name.clone(),
        entry_function_index: program.entry_function,
        functions: compiled_functions,
    })
}

struct FunctionCompiler<'a> {
    function: &'a CheckedFunction,
    local_names: Vec<String>,
    instructions: Vec<Instruction>,
    control_flow_stack: Vec<ControlFlowContext>,
}

impl<'a> FunctionCompiler<'a> {
    fn new(function: &'a CheckedFunction) -> Self {
        Self {
            function,
            local_names: function.local_names.clone(),
            instructions: Vec::new(),
            control_flow_stack: Vec::new(),
        }
    }

    fn compile(mut self) -> Result<CompiledFunction, CompileError> {
        self.initialize_result_locals()?;
        self.compile_block(&self.function.body)?;
        if !matches!(self.instructions.last(), Some(Instruction::Return)) {
            self.instructions.push(Instruction::Return);
        }

        Ok(CompiledFunction {
            name: self.function.name.clone(),
            parameter_count: self.function.parameter_count,
            variadic_element_type: self
                .function
                .variadic_element_type
                .as_ref()
                .map(lower_value_type)
                .transpose()?,
            return_types: lower_result_types(&self.function.return_types)?,
            local_names: self.local_names,
            instructions: self.instructions,
        })
    }

    fn initialize_result_locals(&mut self) -> Result<(), CompileError> {
        for result in &self.function.result_locals {
            self.compile_expression(&CheckedExpression {
                ty: result.ty.clone(),
                kind: CheckedExpressionKind::ZeroValue,
            })?;
            self.instructions.push(Instruction::StoreLocal(result.slot));
        }
        Ok(())
    }

    fn compile_block(&mut self, block: &CheckedBlock) -> Result<(), CompileError> {
        for statement in &block.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: &CheckedStatement) -> Result<(), CompileError> {
        match statement {
            CheckedStatement::ShortVarDecl { bindings, values } => {
                self.compile_binding_statement(bindings, values, "short declaration")?;
            }
            CheckedStatement::MultiAssign { bindings, values } => {
                self.compile_binding_statement(bindings, values, "assignment")?;
            }
            CheckedStatement::VarDecl { slot, value, .. } => {
                if let Some(value) = value {
                    self.compile_local_store(*slot, value, "variable declaration")?;
                }
            }
            CheckedStatement::Assign { target, value } => {
                self.compile_assignment(target, value, "assignment")?
            }
            CheckedStatement::Send { channel, value } => {
                self.compile_send_statement(channel, value)?
            }
            CheckedStatement::CompoundAssign {
                target,
                operator,
                value,
            } => {
                self.compile_compound_assignment(target, *operator, value, "compound assignment")?
            }
            CheckedStatement::Expr(expression) => {
                self.compile_expression(expression)?;
                if expression.ty.produces_value() {
                    self.instructions.push(Instruction::Pop);
                }
            }
            CheckedStatement::If(if_statement) => self.compile_if_statement(if_statement)?,
            CheckedStatement::Switch(switch_statement) => {
                self.compile_switch_statement(switch_statement)?
            }
            CheckedStatement::For(for_statement) => self.compile_for_statement(for_statement)?,
            CheckedStatement::RangeFor {
                source,
                key_binding,
                value_binding,
                body,
            } => self.compile_range_statement(
                source,
                key_binding.as_ref(),
                value_binding.as_ref(),
                body,
            )?,
            CheckedStatement::MapLookup {
                map,
                key,
                value_binding,
                ok_binding,
            } => self.compile_map_lookup_statement(map, key, value_binding, ok_binding)?,
            CheckedStatement::IncDec {
                target,
                operator,
                operand_type,
            } => self.compile_inc_dec_statement(target, *operator, operand_type)?,
            CheckedStatement::Defer(call) => self.compile_defer_call(call)?,
            CheckedStatement::Break => self.compile_break_statement()?,
            CheckedStatement::Continue => self.compile_continue_statement()?,
            CheckedStatement::Return(values) => {
                self.compile_value_source(values, "return")?;
                self.instructions.push(Instruction::Return);
            }
        }

        Ok(())
    }

    fn compile_for_statement(
        &mut self,
        for_statement: &CheckedForStatement,
    ) -> Result<(), CompileError> {
        if let Some(init) = &for_statement.init {
            self.compile_header_statement(init, "for init")?;
        }

        let loop_start = self.instructions.len();
        let jump_to_end = if let Some(condition) = &for_statement.condition {
            self.compile_expression(condition)?;
            self.expect_value(&condition.ty, "for condition")?;
            Some(self.push_instruction(Instruction::JumpIfFalse(usize::MAX)))
        } else {
            None
        };

        self.control_flow_stack.push(ControlFlowContext::Loop {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
        self.compile_block(&for_statement.body)?;

        let continue_target = self.instructions.len();
        self.patch_loop_continue_jumps(continue_target)?;
        if let Some(post) = &for_statement.post {
            self.compile_for_post_statement(post)?;
        }
        self.instructions.push(Instruction::Jump(loop_start));

        let loop_end = self.instructions.len();
        if let Some(jump_to_end) = jump_to_end {
            self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
        }
        self.patch_loop_break_jumps(loop_end)?;
        self.control_flow_stack.pop();
        Ok(())
    }

    fn compile_if_statement(
        &mut self,
        if_statement: &CheckedIfStatement,
    ) -> Result<(), CompileError> {
        if let Some(header) = &if_statement.header {
            self.compile_header_statement(header, "if header")?;
        }
        self.compile_expression(&if_statement.condition)?;
        let jump_to_else = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));
        self.compile_block(&if_statement.then_block)?;
        if let Some(else_branch) = &if_statement.else_branch {
            let jump_to_end = self.push_instruction(Instruction::Jump(usize::MAX));
            let else_start = self.instructions.len();
            self.patch_jump(jump_to_else, Instruction::JumpIfFalse(else_start));
            match else_branch {
                CheckedElseBranch::Block(else_block) => self.compile_block(else_block)?,
                CheckedElseBranch::If(else_if) => self.compile_if_statement(else_if)?,
            }
            let end = self.instructions.len();
            self.patch_jump(jump_to_end, Instruction::Jump(end));
        } else {
            let end = self.instructions.len();
            self.patch_jump(jump_to_else, Instruction::JumpIfFalse(end));
        }
        Ok(())
    }

    fn compile_switch_statement(
        &mut self,
        switch_statement: &CheckedSwitchStatement,
    ) -> Result<(), CompileError> {
        if let Some(header) = &switch_statement.header {
            self.compile_header_statement(header, "switch header")?;
        }

        let tag_slot = if let Some(expression) = &switch_statement.expression {
            self.compile_expression(expression)?;
            self.expect_value(&expression.ty, "switch expression")?;
            let slot = self.allocate_hidden_local("switch$tag");
            self.instructions.push(Instruction::StoreLocal(slot));
            Some(slot)
        } else {
            None
        };

        let mut end_jumps = Vec::new();
        let default_body = switch_statement
            .clauses
            .iter()
            .find_map(|clause| match clause {
                CheckedSwitchClause::Default(body) => Some(body),
                CheckedSwitchClause::Case { .. } => None,
            });
        self.control_flow_stack.push(ControlFlowContext::Switch {
            break_jumps: Vec::new(),
        });

        for clause in &switch_statement.clauses {
            let CheckedSwitchClause::Case { expressions, body } = clause else {
                continue;
            };
            let mut next_test_jump = None;
            let mut success_jumps = Vec::new();
            for expression in expressions {
                let test_start = self.instructions.len();
                if let Some(previous_jump) = next_test_jump.take() {
                    self.patch_jump(previous_jump, Instruction::JumpIfFalse(test_start));
                }
                if let Some(tag_slot) = tag_slot {
                    self.instructions.push(Instruction::LoadLocal(tag_slot));
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, "switch case expression")?;
                    self.instructions.push(Instruction::Equal);
                } else {
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, "switch case expression")?;
                }
                next_test_jump = Some(self.push_instruction(Instruction::JumpIfFalse(usize::MAX)));
                success_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));
            }

            let body_start = self.instructions.len();
            for jump in success_jumps {
                self.patch_jump(jump, Instruction::Jump(body_start));
            }
            self.compile_block(body)?;
            end_jumps.push(self.push_instruction(Instruction::Jump(usize::MAX)));

            let next_clause_start = self.instructions.len();
            if let Some(jump) = next_test_jump {
                self.patch_jump(jump, Instruction::JumpIfFalse(next_clause_start));
            }
        }

        if let Some(default_body) = default_body {
            self.compile_block(default_body)?;
        }

        let end = self.instructions.len();
        for jump in end_jumps {
            self.patch_jump(jump, Instruction::Jump(end));
        }
        self.patch_switch_break_jumps(end)?;
        self.control_flow_stack.pop();
        Ok(())
    }

    fn compile_break_statement(&mut self) -> Result<(), CompileError> {
        let Some(index) = self
            .control_flow_stack
            .iter()
            .rposition(ControlFlowContext::supports_break)
        else {
            return Err(CompileError::new(
                "`break` lowering requires an enclosing control-flow target",
            ));
        };
        let jump = self.push_instruction(Instruction::Jump(usize::MAX));
        self.control_flow_stack[index].record_break_jump(jump);
        Ok(())
    }

    fn compile_continue_statement(&mut self) -> Result<(), CompileError> {
        let Some(index) = self
            .control_flow_stack
            .iter()
            .rposition(ControlFlowContext::supports_continue)
        else {
            return Err(CompileError::new(
                "`continue` lowering requires an enclosing loop target",
            ));
        };
        let jump = self.push_instruction(Instruction::Jump(usize::MAX));
        self.control_flow_stack[index].record_continue_jump(jump)?;
        Ok(())
    }

    fn compile_expression(&mut self, expression: &CheckedExpression) -> Result<(), CompileError> {
        match &expression.kind {
            CheckedExpressionKind::Integer(value) => {
                self.instructions.push(Instruction::PushInt(*value));
            }
            CheckedExpressionKind::Bool(value) => {
                self.instructions.push(Instruction::PushBool(*value));
            }
            CheckedExpressionKind::String(value) => {
                self.instructions
                    .push(Instruction::PushString(value.clone()));
            }
            CheckedExpressionKind::UntypedNil => {
                return Err(CompileError::new(
                    "untyped `nil` must be resolved before bytecode lowering",
                ));
            }
            CheckedExpressionKind::BoxAny(value) => {
                self.compile_expression(value)?;
                self.expect_value(&value.ty, "interface boxing")?;
                self.instructions
                    .push(Instruction::BoxAny(lower_value_type(&value.ty)?));
            }
            CheckedExpressionKind::TypeAssertion {
                value,
                asserted_type,
            } => {
                self.compile_expression(value)?;
                self.expect_value(&value.ty, "type assertion")?;
                self.instructions
                    .push(Instruction::TypeAssert(lower_value_type(asserted_type)?));
            }
            CheckedExpressionKind::ZeroValue => match &expression.ty {
                Type::Int => self.instructions.push(Instruction::PushInt(0)),
                Type::Byte => self.instructions.push(Instruction::PushByte(0)),
                Type::Bool => self.instructions.push(Instruction::PushBool(false)),
                Type::String => self
                    .instructions
                    .push(Instruction::PushString(String::new())),
                Type::Any => self.instructions.push(Instruction::PushNilInterface),
                Type::UntypedNil => {
                    return Err(CompileError::new(
                        "zero-value synthesis does not support untyped `nil`",
                    ));
                }
                Type::Slice(_) => self.instructions.push(Instruction::PushNilSlice),
                Type::Chan(_) => self.instructions.push(Instruction::PushNilChan),
                Type::Map { .. } => self.instructions.push(Instruction::PushNilMap),
                Type::Void => {
                    return Err(CompileError::new(
                        "zero-value synthesis does not support `void` locals",
                    ));
                }
            },
            CheckedExpressionKind::SliceLiteral { elements } => {
                for element in elements {
                    self.compile_expression(element)?;
                    self.expect_value(&element.ty, "slice literal element")?;
                }
                self.instructions
                    .push(Instruction::BuildSlice(elements.len()));
            }
            CheckedExpressionKind::MapLiteral { entries } => {
                for CheckedMapLiteralEntry { key, value } in entries {
                    self.compile_expression(key)?;
                    self.expect_value(&key.ty, "map literal key")?;
                    self.compile_expression(value)?;
                    self.expect_value(&value.ty, "map literal value")?;
                }
                self.instructions.push(Instruction::BuildMap {
                    map_type: lower_value_type(&expression.ty)?,
                    entry_count: entries.len(),
                });
            }
            CheckedExpressionKind::MakeSlice {
                element_type,
                length,
                capacity,
            } => {
                self.compile_expression(length)?;
                self.expect_value(&length.ty, "make expression")?;
                if let Some(capacity) = capacity {
                    self.compile_expression(capacity)?;
                    self.expect_value(&capacity.ty, "make expression")?;
                }
                self.instructions.push(Instruction::MakeSlice {
                    element_type: lower_value_type(element_type)?,
                    has_capacity: capacity.is_some(),
                });
            }
            CheckedExpressionKind::MakeChan {
                element_type,
                buffer,
            } => {
                if let Some(buffer) = buffer {
                    self.compile_expression(buffer)?;
                    self.expect_value(&buffer.ty, "make expression")?;
                }
                self.instructions.push(Instruction::MakeChan {
                    element_type: lower_value_type(element_type)?,
                    has_buffer: buffer.is_some(),
                });
            }
            CheckedExpressionKind::MakeMap { map_type, hint } => {
                if let Some(hint) = hint {
                    self.compile_expression(hint)?;
                    self.expect_value(&hint.ty, "make expression")?;
                }
                self.instructions.push(Instruction::MakeMap {
                    map_type: lower_value_type(map_type)?,
                    has_hint: hint.is_some(),
                });
            }
            CheckedExpressionKind::Conversion { conversion, value } => {
                self.compile_expression(value)?;
                self.expect_value(&value.ty, "conversion expression")?;
                self.instructions.push(Instruction::Convert(*conversion));
            }
            CheckedExpressionKind::Receive { channel } => {
                self.compile_expression(channel)?;
                self.expect_value(&channel.ty, "receive expression")?;
                self.instructions
                    .push(Instruction::Receive(lower_value_type(&expression.ty)?));
            }
            CheckedExpressionKind::Local { slot, .. } => {
                self.instructions.push(Instruction::LoadLocal(*slot));
            }
            CheckedExpressionKind::Index { target, index } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, "index expression")?;
                self.compile_expression(index)?;
                self.expect_value(&index.ty, "index expression")?;
                if matches!(target.ty, Type::Map { .. }) {
                    self.instructions
                        .push(Instruction::IndexMap(lower_value_type(&target.ty)?));
                } else {
                    self.instructions
                        .push(Instruction::Index(lower_sequence_kind(&target.ty)?));
                }
            }
            CheckedExpressionKind::Slice { target, low, high } => {
                self.compile_expression(target)?;
                self.expect_value(&target.ty, "slice expression")?;
                if let Some(low) = low {
                    self.compile_expression(low)?;
                    self.expect_value(&low.ty, "slice expression")?;
                }
                if let Some(high) = high {
                    self.compile_expression(high)?;
                    self.expect_value(&high.ty, "slice expression")?;
                }
                self.instructions.push(Instruction::Slice {
                    target_kind: lower_sequence_kind(&target.ty)?,
                    has_low: low.is_some(),
                    has_high: high.is_some(),
                });
            }
            CheckedExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left)?;
                self.expect_value(&left.ty, "binary expression")?;
                self.compile_expression(right)?;
                self.expect_value(&right.ty, "binary expression")?;
                self.instructions.push(match operator {
                    CheckedBinaryOperator::Add => Instruction::Add,
                    CheckedBinaryOperator::Concat => Instruction::Concat,
                    CheckedBinaryOperator::Subtract => Instruction::Subtract,
                    CheckedBinaryOperator::Multiply => Instruction::Multiply,
                    CheckedBinaryOperator::Divide => Instruction::Divide,
                    CheckedBinaryOperator::Equal => Instruction::Equal,
                    CheckedBinaryOperator::NotEqual => Instruction::NotEqual,
                    CheckedBinaryOperator::Less => Instruction::Less,
                    CheckedBinaryOperator::LessEqual => Instruction::LessEqual,
                    CheckedBinaryOperator::Greater => Instruction::Greater,
                    CheckedBinaryOperator::GreaterEqual => Instruction::GreaterEqual,
                });
            }
            CheckedExpressionKind::Call(call) => self.compile_call(call, "function call")?,
        }

        Ok(())
    }

    fn compile_call(&mut self, call: &CheckedCall, context: &str) -> Result<(), CompileError> {
        if matches!(call.target, CallTarget::Builtin(BuiltinFunction::Panic)) {
            return self.compile_panic_call(call, context, false);
        }
        self.compile_call_arguments(call, context)?;

        match &call.target {
            CallTarget::Builtin(builtin) => {
                let instruction = match &call.arguments {
                    CheckedCallArguments::Spread { arguments, .. } => {
                        Instruction::CallBuiltinSpread(*builtin, arguments.len())
                    }
                    _ => Instruction::CallBuiltin(*builtin, call.argument_count()),
                };
                self.instructions.push(instruction);
            }
            CallTarget::PackageFunction(function) => {
                let instruction = match &call.arguments {
                    CheckedCallArguments::Spread { arguments, .. } => {
                        Instruction::CallPackageSpread(*function, arguments.len())
                    }
                    _ => Instruction::CallPackage(*function, call.argument_count()),
                };
                self.instructions.push(instruction);
            }
            CallTarget::UserDefined { function_index, .. } => {
                let instruction = match &call.arguments {
                    CheckedCallArguments::Spread { arguments, .. } => {
                        Instruction::CallFunctionSpread(*function_index, arguments.len())
                    }
                    _ => Instruction::CallFunction(*function_index, call.argument_count()),
                };
                self.instructions.push(instruction);
            }
        }
        Ok(())
    }

    fn compile_defer_call(&mut self, call: &CheckedCall) -> Result<(), CompileError> {
        if matches!(call.target, CallTarget::Builtin(BuiltinFunction::Panic)) {
            return self.compile_panic_call(call, "defer statement", true);
        }
        self.compile_call_arguments(call, "defer statement")?;

        match &call.target {
            CallTarget::Builtin(builtin) => {
                if matches!(call.arguments, CheckedCallArguments::Spread { .. }) {
                    return Err(CompileError::new(format!(
                        "builtin `{}` cannot be lowered with explicit `...` in defer",
                        builtin.render()
                    )));
                }
                self.instructions
                    .push(Instruction::DeferBuiltin(*builtin, call.argument_count()));
            }
            CallTarget::PackageFunction(function) => {
                let instruction = match &call.arguments {
                    CheckedCallArguments::Spread { arguments, .. } => {
                        Instruction::DeferPackageSpread(*function, arguments.len())
                    }
                    _ => Instruction::DeferPackage(*function, call.argument_count()),
                };
                self.instructions.push(instruction);
            }
            CallTarget::UserDefined { function_index, .. } => {
                let instruction = match &call.arguments {
                    CheckedCallArguments::Spread { arguments, .. } => {
                        Instruction::DeferFunctionSpread(*function_index, arguments.len())
                    }
                    _ => Instruction::DeferFunction(*function_index, call.argument_count()),
                };
                self.instructions.push(instruction);
            }
        }
        Ok(())
    }

    fn compile_call_arguments(
        &mut self,
        call: &CheckedCall,
        context: &str,
    ) -> Result<(), CompileError> {
        match &call.arguments {
            CheckedCallArguments::Expressions(arguments) => {
                for argument in arguments {
                    self.compile_expression(argument)?;
                    self.expect_value(&argument.ty, context)?;
                }
            }
            CheckedCallArguments::ExpandedCall(expanded_call) => {
                self.compile_call(expanded_call, context)?;
            }
            CheckedCallArguments::Spread { arguments, spread } => {
                for argument in arguments {
                    self.compile_expression(argument)?;
                    self.expect_value(&argument.ty, context)?;
                }
                self.compile_expression(spread)?;
                self.expect_value(&spread.ty, context)?;
            }
        }
        Ok(())
    }

    fn compile_value_source(
        &mut self,
        values: &CheckedValueSource,
        context: &str,
    ) -> Result<(), CompileError> {
        match values {
            CheckedValueSource::Expressions(expressions) => {
                for expression in expressions {
                    self.compile_expression(expression)?;
                    self.expect_value(&expression.ty, context)?;
                }
            }
            CheckedValueSource::Call(call) => self.compile_call(call, context)?,
        }
        Ok(())
    }

    fn expect_value(&self, ty: &Type, context: &str) -> Result<(), CompileError> {
        if ty.produces_value() {
            Ok(())
        } else {
            Err(CompileError::new(format!(
                "{context} requires a value-producing expression"
            )))
        }
    }

    fn push_instruction(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    fn patch_jump(&mut self, index: usize, instruction: Instruction) {
        self.instructions[index] = instruction;
    }

    fn allocate_hidden_local(&mut self, label: &str) -> usize {
        let slot = self.local_names.len();
        self.local_names.push(format!("{label}{slot}"));
        slot
    }

    fn patch_loop_continue_jumps(&mut self, target: usize) -> Result<(), CompileError> {
        let Some(ControlFlowContext::Loop { continue_jumps, .. }) = self.control_flow_stack.last()
        else {
            return Err(CompileError::new(
                "loop continue patching requires an active loop context",
            ));
        };
        let jumps = continue_jumps.clone();
        for jump in jumps {
            self.patch_jump(jump, Instruction::Jump(target));
        }
        if let Some(ControlFlowContext::Loop { continue_jumps, .. }) =
            self.control_flow_stack.last_mut()
        {
            continue_jumps.clear();
        }
        Ok(())
    }

    fn patch_loop_break_jumps(&mut self, target: usize) -> Result<(), CompileError> {
        let Some(ControlFlowContext::Loop { break_jumps, .. }) = self.control_flow_stack.last()
        else {
            return Err(CompileError::new(
                "loop break patching requires an active loop context",
            ));
        };
        let jumps = break_jumps.clone();
        for jump in jumps {
            self.patch_jump(jump, Instruction::Jump(target));
        }
        if let Some(ControlFlowContext::Loop { break_jumps, .. }) =
            self.control_flow_stack.last_mut()
        {
            break_jumps.clear();
        }
        Ok(())
    }

    fn patch_switch_break_jumps(&mut self, target: usize) -> Result<(), CompileError> {
        let Some(ControlFlowContext::Switch { break_jumps }) = self.control_flow_stack.last()
        else {
            return Err(CompileError::new(
                "switch break patching requires an active switch context",
            ));
        };
        let jumps = break_jumps.clone();
        for jump in jumps {
            self.patch_jump(jump, Instruction::Jump(target));
        }
        if let Some(ControlFlowContext::Switch { break_jumps }) = self.control_flow_stack.last_mut()
        {
            break_jumps.clear();
        }
        Ok(())
    }

    fn compile_range_statement(
        &mut self,
        source: &CheckedExpression,
        key_binding: Option<&CheckedBinding>,
        value_binding: Option<&CheckedBinding>,
        body: &CheckedBlock,
    ) -> Result<(), CompileError> {
        self.compile_expression(source)?;
        self.expect_value(&source.ty, "range loop")?;
        let source_slot = self.allocate_hidden_local("range$source");
        self.instructions.push(Instruction::StoreLocal(source_slot));
        self.control_flow_stack.push(ControlFlowContext::Loop {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });

        match &source.ty {
            Type::Slice(_) => {
                let index_slot = self.allocate_hidden_local("range$index");
                self.instructions.push(Instruction::PushInt(0));
                self.instructions.push(Instruction::StoreLocal(index_slot));

                let loop_start = self.instructions.len();
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(source_slot));
                self.instructions
                    .push(Instruction::CallBuiltin(BuiltinFunction::Len, 1));
                self.instructions.push(Instruction::Less);
                let jump_to_end = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));

                self.compile_range_binding_value(key_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    Ok(())
                })?;
                self.compile_range_binding_value(value_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(source_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    Ok(())
                })?;

                self.compile_block(body)?;
                let continue_target = self.instructions.len();
                self.patch_loop_continue_jumps(continue_target)?;
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::PushInt(1));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(index_slot));
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
                self.patch_loop_break_jumps(loop_end)?;
            }
            Type::Map { key, .. } => {
                let keys_slot = self.allocate_hidden_local("range$keys");
                let index_slot = self.allocate_hidden_local("range$index");
                self.instructions.push(Instruction::LoadLocal(source_slot));
                self.instructions
                    .push(Instruction::MapKeys(lower_value_type(key.as_ref())?));
                self.instructions.push(Instruction::StoreLocal(keys_slot));
                self.instructions.push(Instruction::PushInt(0));
                self.instructions.push(Instruction::StoreLocal(index_slot));

                let loop_start = self.instructions.len();
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::LoadLocal(keys_slot));
                self.instructions
                    .push(Instruction::CallBuiltin(BuiltinFunction::Len, 1));
                self.instructions.push(Instruction::Less);
                let jump_to_end = self.push_instruction(Instruction::JumpIfFalse(usize::MAX));

                self.compile_range_binding_value(key_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(keys_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    Ok(())
                })?;
                self.compile_range_binding_value(value_binding, |compiler| {
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(source_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(keys_slot));
                    compiler
                        .instructions
                        .push(Instruction::LoadLocal(index_slot));
                    compiler
                        .instructions
                        .push(Instruction::Index(SequenceKind::Slice));
                    compiler
                        .instructions
                        .push(Instruction::IndexMap(lower_value_type(&source.ty)?));
                    Ok(())
                })?;

                self.compile_block(body)?;
                let continue_target = self.instructions.len();
                self.patch_loop_continue_jumps(continue_target)?;
                self.instructions.push(Instruction::LoadLocal(index_slot));
                self.instructions.push(Instruction::PushInt(1));
                self.instructions.push(Instruction::Add);
                self.instructions.push(Instruction::StoreLocal(index_slot));
                self.instructions.push(Instruction::Jump(loop_start));
                let loop_end = self.instructions.len();
                self.patch_jump(jump_to_end, Instruction::JumpIfFalse(loop_end));
                self.patch_loop_break_jumps(loop_end)?;
            }
            _ => {
                return Err(CompileError::new(format!(
                    "range lowering does not support `{}`",
                    source.ty.render()
                )));
            }
        }

        self.control_flow_stack.pop();

        Ok(())
    }

    fn compile_range_binding_value(
        &mut self,
        binding: Option<&CheckedBinding>,
        emit_value: impl FnOnce(&mut Self) -> Result<(), CompileError>,
    ) -> Result<(), CompileError> {
        let Some(binding) = binding else {
            return Ok(());
        };
        emit_value(self)?;
        self.consume_binding_value(binding);
        Ok(())
    }

    fn consume_binding_value(&mut self, binding: &CheckedBinding) {
        match binding {
            CheckedBinding::Local { slot, .. } => {
                self.instructions.push(Instruction::StoreLocal(*slot));
            }
            CheckedBinding::Discard => self.instructions.push(Instruction::Pop),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ControlFlowContext {
    Loop {
        break_jumps: Vec<usize>,
        continue_jumps: Vec<usize>,
    },
    Switch {
        break_jumps: Vec<usize>,
    },
}

impl ControlFlowContext {
    fn supports_break(&self) -> bool {
        true
    }

    fn supports_continue(&self) -> bool {
        matches!(self, ControlFlowContext::Loop { .. })
    }

    fn record_break_jump(&mut self, jump: usize) {
        match self {
            ControlFlowContext::Loop { break_jumps, .. }
            | ControlFlowContext::Switch { break_jumps } => break_jumps.push(jump),
        }
    }

    fn record_continue_jump(&mut self, jump: usize) -> Result<(), CompileError> {
        match self {
            ControlFlowContext::Loop { continue_jumps, .. } => {
                continue_jumps.push(jump);
                Ok(())
            }
            ControlFlowContext::Switch { .. } => Err(CompileError::new(
                "`continue` lowering requires a loop context",
            )),
        }
    }
}
