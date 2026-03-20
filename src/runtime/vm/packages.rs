use std::convert::TryFrom;

use super::support::{
    execute_bytes_package_function, expect_exact_package_arguments,
    expect_integer_package_argument, expect_string_package_argument,
    expect_string_slice_package_argument, render_package_arguments,
};
use super::{RuntimeError, VirtualMachine};
use crate::package::PackageFunction;
use crate::runtime::value::{StringValue, Value};

impl VirtualMachine {
    pub(super) fn call_package_function(
        &mut self,
        function: PackageFunction,
        arity: usize,
    ) -> Result<(), RuntimeError> {
        let arguments = self.pop_arguments(arity)?;

        match function {
            PackageFunction::FmtPrint => {
                self.output
                    .push_str(&render_package_arguments(&arguments, ""));
            }
            PackageFunction::FmtPrintln => {
                self.output
                    .push_str(&render_package_arguments(&arguments, " "));
                self.output.push('\n');
            }
            PackageFunction::FmtSprint => {
                self.stack
                    .push(Value::String(StringValue::from(render_package_arguments(
                        &arguments, "",
                    ))));
            }
            PackageFunction::StringsContains => {
                let [haystack, needle] = expect_exact_package_arguments(function, arguments, 2)?;
                let haystack = expect_string_package_argument(function, 1, haystack)?;
                let needle = expect_string_package_argument(function, 2, needle)?;
                self.stack.push(Value::Boolean(haystack.contains(&needle)));
            }
            PackageFunction::StringsHasPrefix => {
                let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let prefix = expect_string_package_argument(function, 2, prefix)?;
                self.stack.push(Value::Boolean(value.has_prefix(&prefix)));
            }
            PackageFunction::StringsHasSuffix => {
                let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let suffix = expect_string_package_argument(function, 2, suffix)?;
                self.stack.push(Value::Boolean(value.has_suffix(&suffix)));
            }
            PackageFunction::StringsIndex => {
                let [value, needle] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let needle = expect_string_package_argument(function, 2, needle)?;
                let index = value
                    .index_of(&needle)
                    .map(|offset| offset as i64)
                    .unwrap_or(-1);
                self.stack.push(Value::Integer(index));
            }
            PackageFunction::StringsCut => self.execute_strings_cut(function, arguments)?,
            PackageFunction::StringsCutPrefix => {
                self.execute_strings_cut_prefix(function, arguments)?
            }
            PackageFunction::StringsCutSuffix => {
                self.execute_strings_cut_suffix(function, arguments)?
            }
            PackageFunction::StringsTrimPrefix => {
                let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let prefix = expect_string_package_argument(function, 2, prefix)?;
                let trimmed = value.trim_prefix(&prefix).map_err(|_| {
                    RuntimeError::new("strings.TrimPrefix produced an invalid suffix")
                })?;
                self.stack.push(Value::String(trimmed));
            }
            PackageFunction::StringsTrimSuffix => {
                let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let suffix = expect_string_package_argument(function, 2, suffix)?;
                let trimmed = value.trim_suffix(&suffix).map_err(|_| {
                    RuntimeError::new("strings.TrimSuffix produced an invalid prefix")
                })?;
                self.stack.push(Value::String(trimmed));
            }
            PackageFunction::StringsJoin => {
                let [elements, separator] = expect_exact_package_arguments(function, arguments, 2)?;
                let elements = expect_string_slice_package_argument(function, 1, elements)?;
                let separator = expect_string_package_argument(function, 2, separator)?;
                self.stack
                    .push(Value::String(StringValue::join(&elements, &separator)));
            }
            PackageFunction::StringsRepeat => {
                let [value, count] = expect_exact_package_arguments(function, arguments, 2)?;
                let value = expect_string_package_argument(function, 1, value)?;
                let count = expect_integer_package_argument(function, 2, count)?;
                if count < 0 {
                    return Err(RuntimeError::new(format!(
                        "package function `{}` requires a non-negative repeat count",
                        function.render()
                    )));
                }
                let repeat_count = usize::try_from(count).map_err(|_| {
                    RuntimeError::new(format!(
                        "package function `{}` could not convert repeat count",
                        function.render()
                    ))
                })?;
                value.len().checked_mul(repeat_count).ok_or_else(|| {
                    RuntimeError::new(format!(
                        "package function `{}` overflowed the repeated string size",
                        function.render()
                    ))
                })?;
                self.stack.push(Value::String(value.repeat(repeat_count)));
            }
            PackageFunction::BytesEqual
            | PackageFunction::BytesContains
            | PackageFunction::BytesHasPrefix
            | PackageFunction::BytesHasSuffix
            | PackageFunction::BytesIndex
            | PackageFunction::BytesCut
            | PackageFunction::BytesCutPrefix
            | PackageFunction::BytesCutSuffix
            | PackageFunction::BytesTrimPrefix
            | PackageFunction::BytesTrimSuffix
            | PackageFunction::BytesJoin
            | PackageFunction::BytesRepeat => {
                for value in execute_bytes_package_function(function, arguments)? {
                    self.stack.push(value);
                }
            }
        }

        Ok(())
    }

    fn execute_strings_cut(
        &mut self,
        function: PackageFunction,
        arguments: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        let [value, separator] = expect_exact_package_arguments(function, arguments, 2)?;
        let value = expect_string_package_argument(function, 1, value)?;
        let separator = expect_string_package_argument(function, 2, separator)?;
        let found_index = value.index_of(&separator);
        if let Some(index) = found_index {
            let before = value
                .slice(0, index)
                .map_err(|_| RuntimeError::new("strings.Cut produced an invalid prefix"))?;
            let after = value
                .slice(index + separator.len(), value.len())
                .map_err(|_| RuntimeError::new("strings.Cut produced an invalid suffix"))?;
            self.stack.push(Value::String(before));
            self.stack.push(Value::String(after));
            self.stack.push(Value::Boolean(true));
        } else {
            self.stack.push(Value::String(value));
            self.stack.push(Value::String(StringValue::from("")));
            self.stack.push(Value::Boolean(false));
        }
        Ok(())
    }

    fn execute_strings_cut_prefix(
        &mut self,
        function: PackageFunction,
        arguments: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        let [value, prefix] = expect_exact_package_arguments(function, arguments, 2)?;
        let value = expect_string_package_argument(function, 1, value)?;
        let prefix = expect_string_package_argument(function, 2, prefix)?;
        if prefix.as_bytes().is_empty() || value.has_prefix(&prefix) {
            let after = value
                .trim_prefix(&prefix)
                .map_err(|_| RuntimeError::new("strings.CutPrefix produced an invalid suffix"))?;
            self.stack.push(Value::String(after));
            self.stack.push(Value::Boolean(true));
        } else {
            self.stack.push(Value::String(value));
            self.stack.push(Value::Boolean(false));
        }
        Ok(())
    }

    fn execute_strings_cut_suffix(
        &mut self,
        function: PackageFunction,
        arguments: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        let [value, suffix] = expect_exact_package_arguments(function, arguments, 2)?;
        let value = expect_string_package_argument(function, 1, value)?;
        let suffix = expect_string_package_argument(function, 2, suffix)?;
        if suffix.as_bytes().is_empty() || value.has_suffix(&suffix) {
            let before = value
                .trim_suffix(&suffix)
                .map_err(|_| RuntimeError::new("strings.CutSuffix produced an invalid prefix"))?;
            self.stack.push(Value::String(before));
            self.stack.push(Value::Boolean(true));
        } else {
            self.stack.push(Value::String(value));
            self.stack.push(Value::Boolean(false));
        }
        Ok(())
    }
}
