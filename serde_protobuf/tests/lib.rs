extern crate protobuf;
extern crate serde;
extern crate serde_value;

extern crate serde_protobuf;

use std::collections;
use std::fs;

use serde_protobuf::descriptor;
use serde_protobuf::de;

mod protobuf_unittest;

macro_rules! value {
    (bool: $v:expr) => {
        serde_value::Value::Bool($v)
    };

    (usize: $v:expr) => {
        serde_value::Value::USize($v)
    };
    (u8: $v:expr) => {
        serde_value::Value::U8($v)
    };
    (u16: $v:expr) => {
        serde_value::Value::U16($v)
    };
    (u32: $v:expr) => {
        serde_value::Value::U32($v)
    };
    (u64: $v:expr) => {
        serde_value::Value::U64($v)
    };

    (isize: $v:expr) => {
        serde_value::Value::ISize($v)
    };
    (i8: $v:expr) => {
        serde_value::Value::I8($v)
    };
    (i16: $v:expr) => {
        serde_value::Value::I16($v)
    };
    (i32: $v:expr) => {
        serde_value::Value::I32($v)
    };
    (i64: $v:expr) => {
        serde_value::Value::I64($v)
    };

    (f32: $v:expr) => {
        serde_value::Value::F32($v)
    };
    (f64: $v:expr) => {
        serde_value::Value::F64($v)
    };

    (char: $v:expr) => {
        serde_value::Value::Char($v)
    };
    (str: $v:expr) => {
        serde_value::Value::String($v.to_owned())
    };
    (string: $v:expr) => {
        serde_value::Value::String($v)
    };
    (unit) => {
        serde_value::Value::Unit
    };
    (unit_struct $v:expr) => {
        serde_value::Value::UnitStruct($v)
    };
    (some $($t:tt)+) => {
        serde_value::Value::Option(Some(Box::new(value!($($t)+))))
    };
    (none) => {
        serde_value::Value::Option(None)
    };
    (newtype $($t:tt)+) => {
        serde_value::Value::Newtype(Box::new(value!($($t)+)))
    };
    (seq [$(($($t:tt)+)),*]) => {
        {
            let mut values = Vec::new();
            $(
                values.push(value!($($t)+));
            )*
             serde_value::Value::Seq(values)
        }
    };
    (map {$(($($k:tt)+) => ($($v:tt)+)),*}) => {
        {
            let mut map = collections::BTreeMap::new();
            $(
                map.insert(value!($($k)+), value!($($v)+));
            )*
            serde_value::Value::Map(map)
        }
    };
    (bytes: $v:expr) => {
        serde_value::Value::Bytes($v.to_vec())
    };
    (byte_buf: $v:expr) => {
        serde_value::Value::Bytes($v)
    };
}

trait Subset {
    fn subset_of(&self, other: &Self) -> bool;
}

impl Subset for serde_value::Value {
    fn subset_of(&self, other: &Self) -> bool {
        use serde_value::Value::*;
        match (self, other) {
            (&Map(ref ma), &Map(ref mb)) => {
                for (ka, va) in ma {
                    if let Some(vb) = mb.get(ka) {
                        if !va.subset_of(vb) {
                            return false
                        }
                    } else {
                        return false
                    }
                }
                true
            },
            (&Option(Some(ref sa)), &Option(Some(ref sb))) => {
                sa.subset_of(&*sb)
            },
            _ => self == other
        }
    }
}

macro_rules! assert_subset {
    ($left:expr , $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(left_val.subset_of(right_val)) {
                    panic!("assertion failed: `(left.subset_of(right))` \
                            (left: `{:?}`, right: `{:?}`)", left_val, right_val)
                }
            }
        }
    })
}

macro_rules! roundtrip {
    ($t:ty, $v:ident, $s:stmt) => {
        {
            use serde::de::Deserialize;

            let mut file = fs::File::open("testdata/descriptors.pb").unwrap();
            let proto = protobuf::parse_from_reader(&mut file).unwrap();
            let descriptors = descriptor::Descriptors::from_proto(&proto);

            let mut $v = <$t>::new();
            $s;
            let bytes = protobuf::Message::write_to_bytes(&mut $v).unwrap();
            let input = protobuf::CodedInputStream::from_bytes(&bytes);

            let message_name = format!(".{}", protobuf::Message::descriptor(&$v).full_name());

            let mut deserializer = de::Deserializer::for_named_message(&descriptors, &message_name, input).unwrap();
            serde_value::Value::deserialize(&mut deserializer).unwrap()
        }
    }
}

#[test]
fn roundtrip_optional_message() {
    let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
        v.mut_optional_nested_message().set_bb(1);
    });

    assert_subset!(value!(map {
        (str: "optional_nested_message") => (some map {
            (str: "bb") => (some i32: 1)
        })
    }), v)
}

#[test]
fn roundtrip_optional_enum() {
    let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
        v.set_optional_nested_enum(protobuf_unittest::unittest::TestAllTypes_NestedEnum::BAZ);
    });

    assert_subset!(value!(map {
        (str: "optional_nested_enum") => (some str: "BAZ")
    }), v)
}

#[test]
fn roundtrip_required() {
    let v = roundtrip!(protobuf_unittest::unittest::TestRequired, v, {
        v.set_a(1);
        v.set_b(2);
        v.set_c(3);
    });

    assert_subset!(value!(map {
        (str: "a") => (i32: 1),
        (str: "b") => (i32: 2),
        (str: "c") => (i32: 3)
    }), v)
}

#[test]
fn roundtrip_repeated_message() {
    let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
        v.mut_repeated_nested_message().push_default().set_bb(1);
        v.mut_repeated_nested_message().push_default().set_bb(2);
        v.mut_repeated_nested_message().push_default().set_bb(3);
    });

    assert_subset!(value!(map {
        (str: "repeated_nested_message") => (seq [
            (map {
                (str: "bb") => (some i32: 1)
            }),
            (map {
                (str: "bb") => (some i32: 2)
            }),
            (map {
                (str: "bb") => (some i32: 3)
            })
        ])
    }), v)
}

#[test]
fn roundtrip_repeated_enum() {
    let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
        v.mut_repeated_nested_enum().push(protobuf_unittest::unittest::TestAllTypes_NestedEnum::BAZ);
        v.mut_repeated_nested_enum().push(protobuf_unittest::unittest::TestAllTypes_NestedEnum::FOO);
        v.mut_repeated_nested_enum().push(protobuf_unittest::unittest::TestAllTypes_NestedEnum::BAR);
    });

    assert_subset!(value!(map {
        (str: "repeated_nested_enum") => (seq [(str: "BAZ"), (str: "FOO"), (str: "BAR")])
    }), v)
}


#[test]
fn roundtrip_recursive() {
    let v = roundtrip!(protobuf_unittest::unittest::TestRecursiveMessage, v, {
        v.mut_a().mut_a().set_i(3);
        v.mut_a().mut_a().mut_a().mut_a().set_i(4);
    });

    assert_subset!(value!(map {
        (str: "a") => (some map {
            (str: "a") => (some map {
                (str: "i") => (some i32: 3),
                (str: "a") => (some map {
                    (str: "a") => (some map {
                        (str: "i") => (some i32: 4)
                    })
                })
            })
        })
    }), v)
}

macro_rules! check_roundtrip_singular {
    ($id:ident, $field:ident, $setter:ident, $v:expr, $($p:tt)+) => {
        #[test]
        fn $id() {
            let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
                v.$setter($v);
            });
            assert_subset!(value!(map {
                (str: stringify!($field)) => ($($p)+: $v)
            }), v)
        }
    }
}

check_roundtrip_singular!(roundtrip_optional_int32, optional_int32, set_optional_int32, 42, some i32);
check_roundtrip_singular!(roundtrip_optional_int64, optional_int64, set_optional_int64, 42, some i64);
check_roundtrip_singular!(roundtrip_optional_uint32, optional_uint32, set_optional_uint32, 42, some u32);
check_roundtrip_singular!(roundtrip_optional_uint64, optional_uint64, set_optional_uint64, 42, some u64);
check_roundtrip_singular!(roundtrip_optional_sint32, optional_sint32, set_optional_sint32, 42, some i32);
check_roundtrip_singular!(roundtrip_optional_sint64, optional_sint64, set_optional_sint64, 42, some i64);
check_roundtrip_singular!(roundtrip_optional_fixed32, optional_fixed32, set_optional_fixed32, 42, some u32);
check_roundtrip_singular!(roundtrip_optional_fixed64, optional_fixed64, set_optional_fixed64, 42, some u64);
check_roundtrip_singular!(roundtrip_optional_sfixed32, optional_sfixed32, set_optional_sfixed32, 42, some i32);
check_roundtrip_singular!(roundtrip_optional_sfixed64, optional_sfixed64, set_optional_sfixed64, 42, some i64);
check_roundtrip_singular!(roundtrip_optional_float, optional_float, set_optional_float, 0.4, some f32);
check_roundtrip_singular!(roundtrip_optional_double, optional_double, set_optional_double, 0.4, some f64);
check_roundtrip_singular!(roundtrip_optional_bool, optional_bool, set_optional_bool, true, some bool);
check_roundtrip_singular!(roundtrip_optional_string, optional_string, set_optional_string, "hello".to_owned(), some string);
check_roundtrip_singular!(roundtrip_optional_bytes, optional_bytes, set_optional_bytes, vec![1, 2, 3], some byte_buf);

macro_rules! check_roundtrip_default {
    ($id:ident, $field:ident, $v:expr, $($p:tt)+) => {
        #[test]
        fn $id() {
            let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {});
            assert_subset!(value!(map {
                (str: stringify!($field)) => ($($p)+: $v)
            }), v)
        }
    }
}

check_roundtrip_default!(roundtrip_default_int32, default_int32, 41, some i32);
check_roundtrip_default!(roundtrip_default_int64, default_int64, 42, some i64);
check_roundtrip_default!(roundtrip_default_uint32, default_uint32, 43, some u32);
check_roundtrip_default!(roundtrip_default_uint64, default_uint64, 44, some u64);
check_roundtrip_default!(roundtrip_default_sint32, default_sint32, -45, some i32);
check_roundtrip_default!(roundtrip_default_sint64, default_sint64, 46, some i64);
check_roundtrip_default!(roundtrip_default_fixed32, default_fixed32, 47, some u32);
check_roundtrip_default!(roundtrip_default_fixed64, default_fixed64, 48, some u64);
check_roundtrip_default!(roundtrip_default_sfixed32, default_sfixed32, 49, some i32);
check_roundtrip_default!(roundtrip_default_sfixed64, default_sfixed64, -50, some i64);
check_roundtrip_default!(roundtrip_default_float, default_float, 51.5, some f32);
check_roundtrip_default!(roundtrip_default_double, default_double, 52e3, some f64);
check_roundtrip_default!(roundtrip_default_bool, default_bool, true, some bool);
check_roundtrip_default!(roundtrip_default_string, default_string, "hello".to_owned(), some string);
check_roundtrip_default!(roundtrip_default_bytes, default_bytes, "world".as_bytes().to_owned(), some byte_buf);

macro_rules! check_roundtrip_repeated {
    ($id:ident, $field:ident, $mut_getter:ident, [$($v:expr),+], $p:tt) => {
        #[test]
        fn $id() {
            let v = roundtrip!(protobuf_unittest::unittest::TestAllTypes, v, {
                $(
                    v.$mut_getter().push($v);
                )+
            });
            assert_subset!(value!(map {
                (str: stringify!($field)) => (seq [$(($p: $v)),+])
            }), v)
        }
    }
}

check_roundtrip_repeated!(roundtrip_repeated_int32, repeated_int32, mut_repeated_int32, [42, 21, 0], i32);
check_roundtrip_repeated!(roundtrip_repeated_int64, repeated_int64, mut_repeated_int64, [42, 21, 0], i64);
check_roundtrip_repeated!(roundtrip_repeated_uint32, repeated_uint32, mut_repeated_uint32, [42, 21, 0], u32);
check_roundtrip_repeated!(roundtrip_repeated_uint64, repeated_uint64, mut_repeated_uint64, [42, 21, 0], u64);
check_roundtrip_repeated!(roundtrip_repeated_sint32, repeated_sint32, mut_repeated_sint32, [42, 21, 0], i32);
check_roundtrip_repeated!(roundtrip_repeated_sint64, repeated_sint64, mut_repeated_sint64, [42, 21, 0], i64);
check_roundtrip_repeated!(roundtrip_repeated_fixed32, repeated_fixed32, mut_repeated_fixed32, [42, 21, 0], u32);
check_roundtrip_repeated!(roundtrip_repeated_fixed64, repeated_fixed64, mut_repeated_fixed64, [42, 21, 0], u64);
check_roundtrip_repeated!(roundtrip_repeated_sfixed32, repeated_sfixed32, mut_repeated_sfixed32, [42, 21, 0], i32);
check_roundtrip_repeated!(roundtrip_repeated_sfixed64, repeated_sfixed64, mut_repeated_sfixed64, [42, 21, 0], i64);
check_roundtrip_repeated!(roundtrip_repeated_float, repeated_float, mut_repeated_float, [0.4, 0.0, 1.0], f32);
check_roundtrip_repeated!(roundtrip_repeated_double, repeated_double, mut_repeated_double, [0.4, 0.0, 1.0], f64);
check_roundtrip_repeated!(roundtrip_repeated_bool, repeated_bool, mut_repeated_bool, [true, true, false], bool);
check_roundtrip_repeated!(roundtrip_repeated_string, repeated_string, mut_repeated_string, ["hello".to_owned(), "".to_owned()], string);
check_roundtrip_repeated!(roundtrip_repeated_bytes, repeated_bytes, mut_repeated_bytes, [vec![1, 2, 3], vec![2, 3, 4]], byte_buf);
