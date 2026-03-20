use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use crate::bytecode::instruction::ValueType;

#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Byte(u8),
    Boolean(bool),
    String(StringValue),
    Interface(InterfaceValue),
    Slice(SliceValue),
    Chan(ChannelValue),
    Map(MapValue),
}

impl Default for Value {
    fn default() -> Self {
        Self::Integer(0)
    }
}

impl Value {
    fn clear_zero_value(&self) -> Self {
        match self {
            Value::Integer(_) => Value::Integer(0),
            Value::Byte(_) => Value::Byte(0),
            Value::Boolean(_) => Value::Boolean(false),
            Value::String(_) => Value::String(StringValue::empty()),
            Value::Interface(_) => Value::Interface(InterfaceValue::nil()),
            Value::Slice(_) => Value::Slice(SliceValue::nil()),
            Value::Chan(_) => Value::Chan(ChannelValue::nil()),
            Value::Map(_) => Value::Map(MapValue::nil()),
        }
    }

    pub fn boxed_any(value_type: ValueType, value: Value) -> Self {
        Self::Interface(InterfaceValue::boxed(value_type, value))
    }

    pub fn nil_any() -> Self {
        Self::Interface(InterfaceValue::nil())
    }

    pub fn is_runtime_comparable(&self) -> bool {
        match self {
            Value::Integer(_)
            | Value::Byte(_)
            | Value::Boolean(_)
            | Value::String(_)
            | Value::Chan(_) => true,
            Value::Interface(value) => value
                .value()
                .map(Value::is_runtime_comparable)
                .unwrap_or(true),
            Value::Slice(_) | Value::Map(_) => false,
        }
    }

    pub fn boxed_value(&self) -> Option<&Value> {
        match self {
            Value::Interface(value) => value.value(),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(value) => write!(f, "{value}"),
            Value::Byte(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::Interface(value) => write!(f, "{value}"),
            Value::Slice(slice) => write!(
                f,
                "[{}]",
                slice
                    .visible_elements()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Value::Chan(channel) => write!(f, "{channel}"),
            Value::Map(map) => write!(f, "{map}"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::Byte(left), Value::Byte(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Interface(left), Value::Interface(right)) => left == right,
            (Value::Slice(left), Value::Slice(right)) => left == right,
            (Value::Chan(left), Value::Chan(right)) => left == right,
            (Value::Map(left), Value::Map(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceValue {
    value_type: Option<ValueType>,
    value: Option<Box<Value>>,
}

impl InterfaceValue {
    pub fn nil() -> Self {
        Self {
            value_type: None,
            value: None,
        }
    }

    pub fn boxed(value_type: ValueType, value: Value) -> Self {
        Self {
            value_type: Some(value_type),
            value: Some(Box::new(value)),
        }
    }

    pub fn value(&self) -> Option<&Value> {
        self.value.as_deref()
    }

    pub fn value_type(&self) -> Option<&ValueType> {
        self.value_type.as_ref()
    }

    pub fn into_inner(self) -> Option<Value> {
        self.value.map(|value| *value)
    }

    pub fn is_nil(&self) -> bool {
        self.value.is_none()
    }
}

impl fmt::Display for InterfaceValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value() {
            Some(value) => write!(f, "{value}"),
            None => f.write_str("<nil>"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringValue {
    bytes: Vec<u8>,
}

impl StringValue {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn empty() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn byte_at(&self, index: usize) -> Option<u8> {
        self.bytes.get(index).copied()
    }

    pub fn slice(&self, low: usize, high: usize) -> Result<Self, ()> {
        if low > high || high > self.bytes.len() {
            return Err(());
        }

        Ok(Self::new(self.bytes[low..high].to_vec()))
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut bytes = Vec::with_capacity(self.len() + other.len());
        bytes.extend_from_slice(&self.bytes);
        bytes.extend_from_slice(&other.bytes);
        Self::new(bytes)
    }

    pub fn contains(&self, needle: &Self) -> bool {
        self.index_of(needle).is_some()
    }

    pub fn has_prefix(&self, prefix: &Self) -> bool {
        self.bytes.starts_with(prefix.as_bytes())
    }

    pub fn has_suffix(&self, suffix: &Self) -> bool {
        self.bytes.ends_with(suffix.as_bytes())
    }

    pub fn index_of(&self, needle: &Self) -> Option<usize> {
        if needle.bytes.is_empty() {
            return Some(0);
        }

        self.bytes
            .windows(needle.bytes.len())
            .position(|window| window == needle.as_bytes())
    }

    pub fn trim_prefix(&self, prefix: &Self) -> Result<Self, ()> {
        if prefix.as_bytes().is_empty() || self.has_prefix(prefix) {
            self.slice(prefix.len(), self.len())
        } else {
            Ok(self.clone())
        }
    }

    pub fn trim_suffix(&self, suffix: &Self) -> Result<Self, ()> {
        if suffix.as_bytes().is_empty() || self.has_suffix(suffix) {
            self.slice(0, self.len() - suffix.len())
        } else {
            Ok(self.clone())
        }
    }

    pub fn repeat(&self, count: usize) -> Self {
        let mut bytes = Vec::with_capacity(self.len() * count);
        for _ in 0..count {
            bytes.extend_from_slice(&self.bytes);
        }
        Self::new(bytes)
    }

    pub fn join(elements: &[Self], separator: &Self) -> Self {
        if elements.is_empty() {
            return Self::empty();
        }

        let separator_bytes = separator.as_bytes();
        let total_separator_bytes = separator_bytes.len() * elements.len().saturating_sub(1);
        let total_element_bytes = elements.iter().map(Self::len).sum::<usize>();
        let mut bytes = Vec::with_capacity(total_element_bytes + total_separator_bytes);

        for (index, element) in elements.iter().enumerate() {
            if index > 0 {
                bytes.extend_from_slice(separator_bytes);
            }
            bytes.extend_from_slice(element.as_bytes());
        }

        Self::new(bytes)
    }

    pub fn from_byte_slice(slice: &SliceValue) -> Result<Self, ()> {
        Ok(Self::new(slice.byte_elements()?))
    }

    fn render_lossy(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

impl From<&str> for StringValue {
    fn from(value: &str) -> Self {
        Self::new(value.as_bytes().to_vec())
    }
}

impl From<String> for StringValue {
    fn from(value: String) -> Self {
        Self::new(value.into_bytes())
    }
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_lossy())
    }
}

#[derive(Clone, Debug)]
pub struct SliceValue {
    storage: Rc<RefCell<Vec<Value>>>,
    start: usize,
    len: usize,
    capacity: usize,
    is_nil: bool,
}

impl SliceValue {
    pub fn new(elements: Vec<Value>) -> Self {
        let len = elements.len();
        Self {
            storage: Rc::new(RefCell::new(elements)),
            start: 0,
            len,
            capacity: len,
            is_nil: false,
        }
    }

    pub fn nil() -> Self {
        Self {
            storage: Rc::new(RefCell::new(Vec::new())),
            start: 0,
            len: 0,
            capacity: 0,
            is_nil: true,
        }
    }

    pub fn with_len_and_capacity(fill: Value, len: usize, capacity: usize) -> Self {
        let mut storage = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            storage.push(fill.clone());
        }
        Self {
            storage: Rc::new(RefCell::new(storage)),
            start: 0,
            len,
            capacity,
            is_nil: false,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_nil(&self) -> bool {
        self.is_nil
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.len {
            return None;
        }
        self.storage.borrow().get(self.start + index).cloned()
    }

    pub fn set(&self, index: usize, value: Value) -> Result<(), ()> {
        if index >= self.len {
            return Err(());
        }
        let mut storage = self.storage.borrow_mut();
        storage[self.start + index] = value;
        Ok(())
    }

    pub fn visible_elements(&self) -> Vec<Value> {
        self.storage.borrow()[self.start..self.start + self.len].to_vec()
    }

    pub fn slice(&self, low: usize, high: usize) -> Result<Self, ()> {
        if low > high || high > self.capacity {
            return Err(());
        }
        Ok(Self {
            storage: Rc::clone(&self.storage),
            start: self.start + low,
            len: high - low,
            capacity: self.capacity - low,
            is_nil: self.is_nil && low == 0 && high == 0,
        })
    }

    pub fn copy_from(&self, source: &Self) -> usize {
        let count = self.len.min(source.len);
        let snapshot = source
            .visible_elements()
            .into_iter()
            .take(count)
            .collect::<Vec<_>>();
        let mut storage = self.storage.borrow_mut();
        for (offset, value) in snapshot.into_iter().enumerate() {
            storage[self.start + offset] = value;
        }
        count
    }

    pub fn copy_from_string(&self, source: &StringValue) -> usize {
        let count = self.len.min(source.len());
        let mut storage = self.storage.borrow_mut();
        for (offset, value) in source.as_bytes().iter().take(count).enumerate() {
            storage[self.start + offset] = Value::Byte(*value);
        }
        count
    }

    pub fn byte_elements(&self) -> Result<Vec<u8>, ()> {
        self.visible_elements()
            .into_iter()
            .map(|element| match element {
                Value::Byte(value) => Ok(value),
                _ => Err(()),
            })
            .collect()
    }

    pub fn byte_index_of(&self, needle: &[u8]) -> Result<Option<usize>, ()> {
        if needle.is_empty() {
            return Ok(Some(0));
        }

        let bytes = self.byte_elements()?;
        Ok(bytes
            .windows(needle.len())
            .position(|window| window == needle))
    }

    pub fn has_byte_prefix(&self, prefix: &[u8]) -> Result<bool, ()> {
        Ok(self.byte_elements()?.starts_with(prefix))
    }

    pub fn has_byte_suffix(&self, suffix: &[u8]) -> Result<bool, ()> {
        Ok(self.byte_elements()?.ends_with(suffix))
    }

    pub fn trim_byte_prefix(&self, prefix: &[u8]) -> Result<Self, ()> {
        if prefix.is_empty() || self.has_byte_prefix(prefix)? {
            self.slice(prefix.len(), self.len())
        } else {
            Ok(self.clone())
        }
    }

    pub fn trim_byte_suffix(&self, suffix: &[u8]) -> Result<Self, ()> {
        if suffix.is_empty() || self.has_byte_suffix(suffix)? {
            self.slice(0, self.len() - suffix.len())
        } else {
            Ok(self.clone())
        }
    }

    pub fn from_string(value: &StringValue) -> Self {
        Self::new(value.as_bytes().iter().copied().map(Value::Byte).collect())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(bytes.iter().copied().map(Value::Byte).collect())
    }

    pub fn append(&self, rest: &[Value]) -> Self {
        if rest.is_empty() {
            return self.clone();
        }

        let new_len = self.len + rest.len();
        if new_len <= self.capacity {
            let mut storage = self.storage.borrow_mut();
            for (offset, value) in rest.iter().cloned().enumerate() {
                storage[self.start + self.len + offset] = value;
            }
            drop(storage);
            return Self {
                storage: Rc::clone(&self.storage),
                start: self.start,
                len: new_len,
                capacity: self.capacity,
                is_nil: false,
            };
        }

        let mut elements = self.visible_elements();
        elements.extend(rest.iter().cloned());
        Self::new(elements)
    }

    pub fn clear(&self) {
        if self.len == 0 {
            return;
        }

        let mut storage = self.storage.borrow_mut();
        for offset in 0..self.len {
            let index = self.start + offset;
            let zero_value = storage[index].clear_zero_value();
            storage[index] = zero_value;
        }
    }
}

impl PartialEq for SliceValue {
    fn eq(&self, other: &Self) -> bool {
        self.is_nil == other.is_nil
            && self.len == other.len
            && self.capacity == other.capacity
            && self.visible_elements() == other.visible_elements()
    }
}

impl Eq for SliceValue {}

#[derive(Clone, Debug)]
pub struct ChannelValue {
    state: Rc<RefCell<ChannelState>>,
    is_nil: bool,
}

#[derive(Clone, Debug)]
struct ChannelState {
    buffer: VecDeque<Value>,
    capacity: usize,
    closed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelSendError {
    Nil,
    Closed,
    WouldBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelReceiveError {
    Nil,
    WouldBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelReceiveResult {
    Value(Value),
    ClosedEmpty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelCloseError {
    Nil,
    Closed,
}

impl ChannelValue {
    pub fn nil() -> Self {
        Self {
            state: Rc::new(RefCell::new(ChannelState {
                buffer: VecDeque::new(),
                capacity: 0,
                closed: false,
            })),
            is_nil: true,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            state: Rc::new(RefCell::new(ChannelState {
                buffer: VecDeque::new(),
                capacity,
                closed: false,
            })),
            is_nil: false,
        }
    }

    pub fn len(&self) -> usize {
        self.state.borrow().buffer.len()
    }

    pub fn capacity(&self) -> usize {
        self.state.borrow().capacity
    }

    pub fn send(&self, value: Value) -> Result<(), ChannelSendError> {
        if self.is_nil {
            return Err(ChannelSendError::Nil);
        }
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(ChannelSendError::Closed);
        }
        if state.buffer.len() == state.capacity {
            return Err(ChannelSendError::WouldBlock);
        }
        state.buffer.push_back(value);
        Ok(())
    }

    pub fn receive(&self) -> Result<ChannelReceiveResult, ChannelReceiveError> {
        if self.is_nil {
            return Err(ChannelReceiveError::Nil);
        }
        let mut state = self.state.borrow_mut();
        if let Some(value) = state.buffer.pop_front() {
            return Ok(ChannelReceiveResult::Value(value));
        }
        if state.closed {
            Ok(ChannelReceiveResult::ClosedEmpty)
        } else {
            Err(ChannelReceiveError::WouldBlock)
        }
    }

    pub fn close(&self) -> Result<(), ChannelCloseError> {
        if self.is_nil {
            return Err(ChannelCloseError::Nil);
        }
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(ChannelCloseError::Closed);
        }
        state.closed = true;
        Ok(())
    }
}

impl fmt::Display for ChannelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.borrow();
        write!(
            f,
            "chan(len={} cap={} closed={})",
            state.buffer.len(),
            state.capacity,
            state.closed
        )
    }
}

impl PartialEq for ChannelValue {
    fn eq(&self, other: &Self) -> bool {
        (self.is_nil && other.is_nil)
            || (!self.is_nil && !other.is_nil && Rc::ptr_eq(&self.state, &other.state))
    }
}

impl Eq for ChannelValue {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapKey {
    Integer(i64),
    Byte(u8),
    Boolean(bool),
    String(StringValue),
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapKey::Integer(value) => write!(f, "{value}"),
            MapKey::Byte(value) => write!(f, "{value}"),
            MapKey::Boolean(value) => write!(f, "{value}"),
            MapKey::String(value) => write!(f, "{value}"),
        }
    }
}

impl From<MapKey> for Value {
    fn from(value: MapKey) -> Self {
        match value {
            MapKey::Integer(value) => Value::Integer(value),
            MapKey::Byte(value) => Value::Byte(value),
            MapKey::Boolean(value) => Value::Boolean(value),
            MapKey::String(value) => Value::String(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapValue {
    entries: Rc<RefCell<BTreeMap<MapKey, Value>>>,
    is_nil: bool,
}

impl MapValue {
    pub fn nil() -> Self {
        Self {
            entries: Rc::new(RefCell::new(BTreeMap::new())),
            is_nil: true,
        }
    }

    pub fn with_hint(_hint: usize) -> Self {
        Self {
            entries: Rc::new(RefCell::new(BTreeMap::new())),
            is_nil: false,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn get(&self, key: &MapKey) -> Option<Value> {
        self.entries.borrow().get(key).cloned()
    }

    pub fn insert(&self, key: MapKey, value: Value) -> Result<(), ()> {
        if self.is_nil {
            return Err(());
        }
        self.entries.borrow_mut().insert(key, value);
        Ok(())
    }

    pub fn remove(&self, key: &MapKey) {
        if self.is_nil {
            return;
        }
        self.entries.borrow_mut().remove(key);
    }

    pub fn clear(&self) {
        if self.is_nil {
            return;
        }
        self.entries.borrow_mut().clear();
    }

    pub fn visible_entries(&self) -> Vec<(MapKey, Value)> {
        self.entries
            .borrow()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    pub fn keys_as_values(&self) -> Vec<Value> {
        self.entries
            .borrow()
            .keys()
            .cloned()
            .map(Value::from)
            .collect()
    }
}

impl fmt::Display for MapValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self
            .visible_entries()
            .into_iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "map[{entries}]")
    }
}

impl PartialEq for MapValue {
    fn eq(&self, other: &Self) -> bool {
        self.is_nil == other.is_nil && self.visible_entries() == other.visible_entries()
    }
}

impl Eq for MapValue {}

#[cfg(test)]
mod tests;
