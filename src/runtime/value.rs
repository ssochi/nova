use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Byte(u8),
    Boolean(bool),
    String(StringValue),
    Slice(SliceValue),
    Chan(ChannelValue),
    Map(MapValue),
}

impl Default for Value {
    fn default() -> Self {
        Self::Integer(0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(value) => write!(f, "{value}"),
            Value::Byte(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
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
            (Value::Slice(left), Value::Slice(right)) => left == right,
            (Value::Chan(left), Value::Chan(right)) => left == right,
            (Value::Map(left), Value::Map(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

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
        if needle.bytes.is_empty() {
            return true;
        }

        self.bytes
            .windows(needle.bytes.len())
            .any(|window| window == needle.as_bytes())
    }

    pub fn has_prefix(&self, prefix: &Self) -> bool {
        self.bytes.starts_with(prefix.as_bytes())
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

    pub fn from_string(value: &StringValue) -> Self {
        Self::new(value.as_bytes().iter().copied().map(Value::Byte).collect())
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
mod tests {
    use super::{
        ChannelCloseError, ChannelReceiveError, ChannelReceiveResult, ChannelSendError,
        ChannelValue, MapKey, MapValue, SliceValue, StringValue, Value,
    };

    #[test]
    fn byte_oriented_strings_support_byte_access_and_slicing() {
        let value = StringValue::new(vec![0xE4, 0xB8, 0xAD, b'!']);

        assert_eq!(value.len(), 4);
        assert_eq!(value.byte_at(0), Some(0xE4));
        assert_eq!(
            value.slice(0, 3).expect("slice should succeed"),
            StringValue::new(vec![0xE4, 0xB8, 0xAD])
        );
    }

    #[test]
    fn byte_oriented_strings_join_and_search() {
        let joined = StringValue::join(
            &[StringValue::from("nova"), StringValue::from("go")],
            &StringValue::from("-"),
        );

        assert!(joined.contains(&StringValue::from("va-g")));
        assert!(joined.has_prefix(&StringValue::from("nova")));
        assert_eq!(joined, StringValue::from("nova-go"));
    }

    #[test]
    fn append_within_capacity_reuses_backing_storage() {
        let base = SliceValue::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let window = base.slice(0, 2).expect("slice window should succeed");

        let grown = window.append(&[Value::Integer(9)]);

        assert_eq!(
            grown.visible_elements(),
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(9)]
        );
        assert_eq!(grown.capacity(), 3);
        assert_eq!(
            base.visible_elements(),
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(9)]
        );
    }

    #[test]
    fn copy_handles_overlapping_ranges() {
        let base = SliceValue::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let destination = base.slice(0, 3).expect("destination slice should succeed");
        let source = base.slice(1, 4).expect("source slice should succeed");

        let copied = destination.copy_from(&source);

        assert_eq!(copied, 3);
        assert_eq!(
            base.visible_elements(),
            vec![
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4),
                Value::Integer(4),
            ]
        );
    }

    #[test]
    fn nil_slice_reports_zero_lengths_and_appends_into_real_storage() {
        let nil_slice = SliceValue::nil();

        assert_eq!(nil_slice.len(), 0);
        assert_eq!(nil_slice.capacity(), 0);
        assert_eq!(nil_slice.visible_elements(), Vec::<Value>::new());

        let grown = nil_slice.append(&[Value::Integer(7), Value::Integer(8)]);

        assert_eq!(
            grown.visible_elements(),
            vec![Value::Integer(7), Value::Integer(8)]
        );
        assert_eq!(grown.capacity(), 2);
    }

    #[test]
    fn make_allocates_hidden_capacity_with_zero_values() {
        let slice = SliceValue::with_len_and_capacity(Value::Integer(0), 2, 4);
        let values = slice.visible_elements();

        assert_eq!(values, vec![Value::Integer(0), Value::Integer(0)]);
        assert_eq!(slice.len(), 2);
        assert_eq!(slice.capacity(), 4);
        assert_eq!(slice.get(2), None);

        let expanded = slice.slice(0, 4).expect("reslice should expose capacity");
        assert_eq!(
            expanded.visible_elements(),
            vec![
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(0),
            ]
        );
    }

    #[test]
    fn copy_from_string_writes_byte_values() {
        let destination = SliceValue::with_len_and_capacity(Value::Byte(0), 3, 3);
        let copied = destination.copy_from_string(&StringValue::from("nova"));

        assert_eq!(copied, 3);
        assert_eq!(
            destination.visible_elements(),
            vec![Value::Byte(b'n'), Value::Byte(b'o'), Value::Byte(b'v')]
        );
    }

    #[test]
    fn string_and_byte_slice_conversions_round_trip() {
        let bytes = SliceValue::from_string(&StringValue::from("nova"));
        let string = StringValue::from_byte_slice(&bytes).expect("byte slice should convert");

        assert_eq!(
            bytes.visible_elements(),
            vec![
                Value::Byte(b'n'),
                Value::Byte(b'o'),
                Value::Byte(b'v'),
                Value::Byte(b'a'),
            ]
        );
        assert_eq!(string, StringValue::from("nova"));
    }

    #[test]
    fn maps_preserve_nil_state_and_support_updates() {
        let nil_map = MapValue::nil();
        assert_eq!(nil_map.len(), 0);
        assert!(
            nil_map
                .get(&MapKey::String(StringValue::from("nova")))
                .is_none()
        );
        assert!(
            nil_map
                .insert(MapKey::String(StringValue::from("nova")), Value::Integer(1))
                .is_err()
        );

        let ready = MapValue::with_hint(2);
        ready
            .insert(MapKey::String(StringValue::from("nova")), Value::Integer(3))
            .expect("map should accept writes");
        ready
            .insert(
                MapKey::Boolean(true),
                Value::String(StringValue::from("go")),
            )
            .expect("map should accept mixed supported keys");

        assert_eq!(ready.len(), 2);
        assert_eq!(
            ready.get(&MapKey::String(StringValue::from("nova"))),
            Some(Value::Integer(3))
        );
        assert_eq!(
            ready.visible_entries(),
            vec![
                (
                    MapKey::Boolean(true),
                    Value::String(StringValue::from("go"))
                ),
                (MapKey::String(StringValue::from("nova")), Value::Integer(3)),
            ]
        );
    }

    #[test]
    fn channels_track_capacity_close_and_identity() {
        let nil_channel = ChannelValue::nil();
        assert_eq!(nil_channel.len(), 0);
        assert_eq!(nil_channel.capacity(), 0);
        assert_eq!(
            nil_channel.send(Value::Integer(1)),
            Err(ChannelSendError::Nil)
        );
        assert_eq!(nil_channel.receive(), Err(ChannelReceiveError::Nil));
        assert_eq!(nil_channel.close(), Err(ChannelCloseError::Nil));

        let channel = ChannelValue::with_capacity(2);
        let alias = channel.clone();
        assert_eq!(channel, alias);
        assert!(channel.send(Value::Integer(3)).is_ok());
        assert!(channel.send(Value::Integer(5)).is_ok());
        assert_eq!(
            channel.send(Value::Integer(7)),
            Err(ChannelSendError::WouldBlock)
        );
        assert_eq!(channel.len(), 2);
        assert_eq!(
            alias.receive(),
            Ok(ChannelReceiveResult::Value(Value::Integer(3)))
        );
        assert!(alias.close().is_ok());
        assert_eq!(channel.close(), Err(ChannelCloseError::Closed));
        assert_eq!(
            channel.receive(),
            Ok(ChannelReceiveResult::Value(Value::Integer(5)))
        );
        assert_eq!(channel.receive(), Ok(ChannelReceiveResult::ClosedEmpty));
        assert_eq!(
            channel.send(Value::Integer(9)),
            Err(ChannelSendError::Closed)
        );
    }

    #[test]
    fn deleting_from_maps_handles_nil_and_missing_entries() {
        let nil_map = MapValue::nil();
        nil_map.remove(&MapKey::String(StringValue::from("ghost")));
        assert_eq!(nil_map.len(), 0);

        let ready = MapValue::with_hint(1);
        ready
            .insert(MapKey::String(StringValue::from("nova")), Value::Integer(3))
            .expect("map should accept writes");
        ready.remove(&MapKey::String(StringValue::from("missing")));
        assert_eq!(ready.len(), 1);
        ready.remove(&MapKey::String(StringValue::from("nova")));
        assert_eq!(ready.len(), 0);
    }
}
