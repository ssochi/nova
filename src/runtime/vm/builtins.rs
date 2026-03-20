use super::support::{expect_exact_builtin_arguments, render_builtin_arguments};
use super::{RuntimeError, VirtualMachine};
use crate::builtin::BuiltinFunction;
use crate::runtime::value::{ChannelCloseError, MapKey, Value};

impl VirtualMachine {
    pub(super) fn call_builtin(
        &mut self,
        builtin: BuiltinFunction,
        arity: usize,
    ) -> Result<(), RuntimeError> {
        let arguments = self.pop_arguments(arity)?;
        for value in self.execute_builtin_call(builtin, arguments)? {
            self.stack.push(value);
        }
        Ok(())
    }

    pub(super) fn execute_builtin_call(
        &mut self,
        builtin: BuiltinFunction,
        arguments: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        match builtin {
            BuiltinFunction::Print => {
                self.output.push_str(&render_builtin_arguments(&arguments));
                Ok(Vec::new())
            }
            BuiltinFunction::Println => {
                self.output.push_str(&render_builtin_arguments(&arguments));
                self.output.push('\n');
                Ok(Vec::new())
            }
            BuiltinFunction::Len => {
                let [argument] = expect_exact_builtin_arguments(arguments, 1, "len")?;
                let value = match argument {
                    Value::String(value) => value.len() as i64,
                    Value::Slice(slice) => slice.len() as i64,
                    Value::Chan(channel) => channel.len() as i64,
                    Value::Map(map) => map.len() as i64,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `len` expected a string, slice, chan, or map argument",
                        ));
                    }
                };
                Ok(vec![Value::Integer(value)])
            }
            BuiltinFunction::Cap => {
                let [argument] = expect_exact_builtin_arguments(arguments, 1, "cap")?;
                let value = match argument {
                    Value::Slice(slice) => slice.capacity() as i64,
                    Value::Chan(channel) => channel.capacity() as i64,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `cap` expected a slice or chan argument",
                        ));
                    }
                };
                Ok(vec![Value::Integer(value)])
            }
            BuiltinFunction::Copy => {
                let [destination, source] = expect_exact_builtin_arguments(arguments, 2, "copy")?;
                let destination = match destination {
                    Value::Slice(slice) => slice,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `copy` expected a slice as argument 1",
                        ));
                    }
                };
                let copied = match source {
                    Value::Slice(slice) => destination.copy_from(&slice),
                    Value::String(value) => destination.copy_from_string(&value),
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `copy` expected a slice or string as argument 2",
                        ));
                    }
                };
                Ok(vec![Value::Integer(copied as i64)])
            }
            BuiltinFunction::Append => {
                let Some((first, rest)) = arguments.split_first() else {
                    return Err(RuntimeError::new(
                        "builtin `append` expected at least 1 argument",
                    ));
                };
                let slice = match first {
                    Value::Slice(slice) => slice.clone(),
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `append` expected a slice as the first argument",
                        ));
                    }
                };
                Ok(vec![Value::Slice(slice.append(rest))])
            }
            BuiltinFunction::Make => Err(RuntimeError::new(
                "builtin `make` is lowered into dedicated allocation bytecode",
            )),
            BuiltinFunction::Delete => {
                let [target, key] = expect_exact_builtin_arguments(arguments, 2, "delete")?;
                let map = match target {
                    Value::Map(map) => map,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `delete` expected a map as argument 1",
                        ));
                    }
                };
                let key = match key {
                    Value::Integer(value) => MapKey::Integer(value),
                    Value::Byte(value) => MapKey::Byte(value),
                    Value::Boolean(value) => MapKey::Boolean(value),
                    Value::String(value) => MapKey::String(value),
                    Value::Interface(_) | Value::Slice(_) | Value::Chan(_) | Value::Map(_) => {
                        return Err(RuntimeError::new(
                            "builtin `delete` expected a comparable scalar key as argument 2",
                        ));
                    }
                };
                map.remove(&key);
                Ok(Vec::new())
            }
            BuiltinFunction::Close => {
                let [target] = expect_exact_builtin_arguments(arguments, 1, "close")?;
                let channel = match target {
                    Value::Chan(channel) => channel,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `close` expected a chan as argument 1",
                        ));
                    }
                };
                channel.close().map_err(|error| match error {
                    ChannelCloseError::Nil => {
                        RuntimeError::user_panic_message("close of nil channel")
                    }
                    ChannelCloseError::Closed => {
                        RuntimeError::user_panic_message("close of closed channel")
                    }
                })?;
                Ok(Vec::new())
            }
            BuiltinFunction::Clear => {
                let [target] = expect_exact_builtin_arguments(arguments, 1, "clear")?;
                match target {
                    Value::Slice(slice) => slice.clear(),
                    Value::Map(map) => map.clear(),
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `clear` expected a slice or map as argument 1",
                        ));
                    }
                }
                Ok(Vec::new())
            }
            BuiltinFunction::Panic => Err(RuntimeError::new(
                "builtin `panic` is lowered into explicit panic bytecode",
            )),
        }
    }

    pub(super) fn call_builtin_spread(
        &mut self,
        builtin: BuiltinFunction,
        prefix_arity: usize,
    ) -> Result<(), RuntimeError> {
        let spread = self.pop_value()?;
        let arguments = self.pop_arguments(prefix_arity)?;
        for value in self.execute_builtin_spread_call(builtin, arguments, spread)? {
            self.stack.push(value);
        }
        Ok(())
    }

    pub(super) fn execute_builtin_spread_call(
        &mut self,
        builtin: BuiltinFunction,
        arguments: Vec<Value>,
        spread: Value,
    ) -> Result<Vec<Value>, RuntimeError> {
        match builtin {
            BuiltinFunction::Append => {
                let [target] = expect_exact_builtin_arguments(arguments, 1, "append")?;
                let slice = match target {
                    Value::Slice(slice) => slice,
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `append` expected a slice as argument 1",
                        ));
                    }
                };
                let appended = match spread {
                    Value::Slice(spread) => spread.visible_elements(),
                    Value::String(value) => {
                        value.as_bytes().iter().copied().map(Value::Byte).collect()
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            "builtin `append` with `...` expected a slice or string spread argument",
                        ));
                    }
                };
                Ok(vec![Value::Slice(slice.append(&appended))])
            }
            _ => Err(RuntimeError::new(format!(
                "builtin `{}` does not support explicit `...` arguments",
                builtin.render()
            ))),
        }
    }
}
