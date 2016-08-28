macro_rules! forward_to_deserialize {
    ($($func:ident),*) => {
        $(forward_to_deserialize!{func: $func})*
    };
    (func: deserialize_unit_struct) => {
        forward_to_deserialize!{named: deserialize_unit_struct}
    };
    (func: deserialize_newtype_struct) => {
        forward_to_deserialize!{named: deserialize_newtype_struct}
    };
    (func: deserialize_tuple) => {
        forward_to_deserialize!{tup_fn: deserialize_tuple}
    };
    (func: deserialize_seq_fixed_size) => {
        forward_to_deserialize!{tup_fn: deserialize_seq_fixed_size}
    };
    (func: deserialize_tuple_struct) => {
        #[inline]
        fn deserialize_tuple_struct<__V>(&mut self, _: &str, _: usize, visitor: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::Visitor {
            self.deserialize(visitor)
        }
    };
    (func: deserialize_struct) => {
        #[inline]
        fn deserialize_struct<__V>(&mut self, _: &str, _: &[&str], visitor: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::Visitor {
            self.deserialize(visitor)
        }
    };
    (func: deserialize_enum) => {
        #[inline]
        fn deserialize_enum<__V>(&mut self, _: &str, _: &[&str], _: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::EnumVisitor {
            Err(::serde::de::Error::invalid_type(::serde::de::Type::Enum))
        }
    };
    (named: $func:ident) => {
        #[inline]
        fn $func<__V>(&mut self, _: &str, visitor: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::Visitor {
            self.deserialize(visitor)
        }
    };
    (tup_fn: $func: ident) => {
        #[inline]
        fn $func<__V>(&mut self, _: usize, visitor: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::Visitor {
            self.deserialize(visitor)
        }
    };
    (func: $func:ident) => {
        #[inline]
        fn $func<__V>(&mut self, visitor: __V) -> Result<__V::Value, Self::Error>
            where __V: ::serde::de::Visitor {
            self.deserialize(visitor)
        }
    };
}
