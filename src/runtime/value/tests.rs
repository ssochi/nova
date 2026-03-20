use super::{
    ChannelCloseError, ChannelReceiveError, ChannelReceiveResult, ChannelSendError, ChannelValue,
    MapKey, MapValue, SliceValue, StringValue, Value,
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
    assert!(joined.has_suffix(&StringValue::from("-go")));
    assert_eq!(joined.index_of(&StringValue::from("go")), Some(5));
    assert_eq!(
        joined
            .trim_prefix(&StringValue::from("nova-"))
            .expect("trimmed prefix should succeed"),
        StringValue::from("go")
    );
    assert_eq!(
        joined
            .trim_suffix(&StringValue::from("-go"))
            .expect("trimmed suffix should succeed"),
        StringValue::from("nova")
    );
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

    assert!(nil_slice.is_nil());
    assert_eq!(nil_slice.len(), 0);
    assert_eq!(nil_slice.capacity(), 0);
    assert_eq!(nil_slice.visible_elements(), Vec::<Value>::new());

    let grown = nil_slice.append(&[Value::Integer(7), Value::Integer(8)]);

    assert_eq!(
        grown.visible_elements(),
        vec![Value::Integer(7), Value::Integer(8)]
    );
    assert!(!grown.is_nil());
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
fn byte_slices_support_index_suffix_and_trim_views() {
    let value = SliceValue::from_bytes(b"nova-go");
    let nil_value = SliceValue::nil();

    assert!(!value.is_nil());
    assert_eq!(value.byte_index_of(b"go"), Ok(Some(5)));
    assert_eq!(value.byte_index_of(b""), Ok(Some(0)));
    assert_eq!(value.has_byte_suffix(b"go"), Ok(true));
    assert_eq!(
        value
            .trim_byte_prefix(b"nova-")
            .expect("trimmed prefix should succeed")
            .byte_elements(),
        Ok(b"go".to_vec())
    );
    assert_eq!(
        value
            .trim_byte_suffix(b"-go")
            .expect("trimmed suffix should succeed")
            .byte_elements(),
        Ok(b"nova".to_vec())
    );
    let trimmed_nil = nil_value
        .trim_byte_prefix(b"")
        .expect("nil empty trim should succeed");
    assert_eq!(trimmed_nil, SliceValue::nil());
}

#[test]
fn clearing_slice_zeroes_visible_range_and_preserves_shape() {
    let base = SliceValue::new(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
    ]);
    let window = base.slice(1, 3).expect("slice window should succeed");

    window.clear();

    assert_eq!(
        base.visible_elements(),
        vec![
            Value::Integer(1),
            Value::Integer(0),
            Value::Integer(0),
            Value::Integer(4),
        ]
    );
    assert_eq!(window.len(), 2);
    assert_eq!(window.capacity(), 3);
}

#[test]
fn clearing_slice_of_composites_uses_nil_zero_values() {
    let nested = SliceValue::new(vec![
        Value::Slice(SliceValue::from_bytes(b"no")),
        Value::Slice(SliceValue::from_bytes(b"va")),
    ]);

    nested.clear();

    assert_eq!(
        nested.visible_elements(),
        vec![
            Value::Slice(SliceValue::nil()),
            Value::Slice(SliceValue::nil())
        ]
    );
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
fn clearing_maps_handles_nil_and_aliases() {
    let nil_map = MapValue::nil();
    nil_map.clear();
    assert_eq!(nil_map.len(), 0);
    assert_eq!(nil_map, MapValue::nil());

    let ready = MapValue::with_hint(1);
    ready
        .insert(MapKey::String(StringValue::from("nova")), Value::Integer(3))
        .expect("map should accept writes");
    let alias = ready.clone();

    alias.clear();

    assert_eq!(ready.len(), 0);
    assert_eq!(alias.len(), 0);
    assert_ne!(ready, MapValue::nil());
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
