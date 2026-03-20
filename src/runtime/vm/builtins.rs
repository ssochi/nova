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

        match builtin {
            BuiltinFunction::Print => {
                self.output.push_str(&render_builtin_arguments(&arguments));
            }
            BuiltinFunction::Println => {
                self.output.push_str(&render_builtin_arguments(&arguments));
                self.output.push('\n');
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
                self.stack.push(Value::Integer(value));
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
                self.stack.push(Value::Integer(value));
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
                self.stack.push(Value::Integer(copied as i64));
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
                self.stack.push(Value::Slice(slice.append(rest)));
            }
            BuiltinFunction::Make => {
                return Err(RuntimeError::new(
                    "builtin `make` is lowered into dedicated allocation bytecode",
                ));
            }
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
                    Value::Slice(_) | Value::Chan(_) | Value::Map(_) => {
                        return Err(RuntimeError::new(
                            "builtin `delete` expected a comparable scalar key as argument 2",
                        ));
                    }
                };
                map.remove(&key);
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
                    ChannelCloseError::Nil => RuntimeError::new("close of nil channel"),
                    ChannelCloseError::Closed => RuntimeError::new("close of closed channel"),
                })?;
            }
        }

        Ok(())
    }
}
