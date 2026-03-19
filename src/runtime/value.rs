use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Byte(u8),
    Boolean(bool),
    String(StringValue),
    Slice(SliceValue),
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
            _ => false,
        }
    }
}

impl Eq for Value {}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::{SliceValue, StringValue, Value};

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
}
