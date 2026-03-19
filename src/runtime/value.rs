use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),
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
            Value::Boolean(value) => write!(f, "{value}"),
            Value::String(value) => f.write_str(value),
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
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Slice(left), Value::Slice(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

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
    use super::{SliceValue, Value};

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
        let values = SliceValue::with_len_and_capacity(Value::Integer(0), 2, 4);
        let grown = values
            .slice(0, 3)
            .expect("reslicing into spare capacity should succeed");

        assert_eq!(values.len(), 2);
        assert_eq!(values.capacity(), 4);
        assert_eq!(
            grown.visible_elements(),
            vec![Value::Integer(0), Value::Integer(0), Value::Integer(0)]
        );
    }
}
