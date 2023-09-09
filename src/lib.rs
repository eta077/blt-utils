#![deny(missing_docs)]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use std::string::FromUtf8Error;

use thiserror::Error;

/// An enumeration of errors that can occur during custom deserialization.
#[derive(Debug, Error, PartialEq)]
pub enum DeserializationError {
    /// Indicates there are less than the necessary number of bytes to deserialize the value.
    #[error("Expected {0} bytes, found {1}")]
    UnexpectedByteCount(usize, usize),
    /// Indicates a String value contains invalid UTF8 bytes.
    #[error("Could not deserialize to string, invalid UTF8")]
    InvalidString {
        #[from]
        /// The source of the error.
        source: FromUtf8Error,
    },
    /// Indicates a custom type could not be converted from raw parts.
    #[error("{0}")]
    InvalidValue(String),
}

/// Appends the string representation of the given value to the buffer.
///
/// # Examples
///
/// ```
/// let mut buffer = Vec::new();
/// blt_utils::serialize_string("Hello World!", &mut buffer);
/// assert_eq!(buffer.as_slice(), [12, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33]);
/// ```
pub fn serialize_string<T: Into<String>>(value: T, buffer: &mut Vec<u8>) {
    let mut value = value.into().into_bytes();
    for b in value.len().to_le_bytes() {
        buffer.push(b);
    }
    buffer.append(&mut value);
}

/// Removes the next string value from the buffer.
///
/// # Examples
///
/// ```
/// let mut buffer = [12, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33].to_vec();
/// let value = blt_utils::deserialize_string::<String>(&mut buffer)?;
/// assert_eq!(value, String::from("Hello World!"));
/// # Ok::<(), blt_utils::DeserializationError>(())
/// ```
pub fn deserialize_string<T: TryFrom<String>>(
    buffer: &mut Vec<u8>,
) -> Result<T, DeserializationError>
where
    <T as TryFrom<String>>::Error: ToString,
{
    let value_size = deserialize_usize(buffer)?;
    if value_size > buffer.len() {
        return Err(DeserializationError::UnexpectedByteCount(
            value_size,
            buffer.len(),
        ));
    }
    let tmp = buffer.split_off(value_size);
    let result = String::from_utf8(buffer.to_owned()).map_err(|ex| ex.into());
    *buffer = tmp;
    result.and_then(|value| {
        T::try_from(value).map_err(|ex| DeserializationError::InvalidValue(ex.to_string()))
    })
}

/// Appends the given collection to the buffer.
///
/// # Examples
///
/// ```
/// let v = ["Hello", "World"].to_vec();
/// let mut buffer = Vec::new();
/// blt_utils::serialize_vec(v, &mut buffer);
///
/// assert_eq!(buffer.as_slice(), [2, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 5, 0, 0, 0, 0, 0, 0, 0, 87, 111, 114, 108, 100]);
/// ```
pub fn serialize_vec<T: Into<String>>(value: Vec<T>, buffer: &mut Vec<u8>) {
    for b in value.len().to_le_bytes() {
        buffer.push(b);
    }
    for item in value {
        serialize_string(item.into(), buffer);
    }
}

/// Removes the next collection of strings from the buffer.
/// If an error occurs for an element after the first, the buffer is left in an indeterminate state.
///
/// # Examples
///
/// ```
/// let mut buffer = [2, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 5, 0, 0, 0, 0, 0, 0, 0, 87, 111, 114, 108, 100].to_vec();
/// let value = blt_utils::deserialize_vec::<String>(&mut buffer)?;
/// assert_eq!(value.as_slice(), [String::from("Hello"), String::from("World")]);
/// # Ok::<(), blt_utils::DeserializationError>(())
/// ```
pub fn deserialize_vec<T: TryFrom<String>>(
    buffer: &mut Vec<u8>,
) -> Result<Vec<T>, DeserializationError>
where
    <T as TryFrom<String>>::Error: ToString,
{
    let num_items = deserialize_usize(buffer)?;
    let mut result = Vec::with_capacity(num_items);
    for _ in 0..num_items {
        result.push(deserialize_string(buffer)?);
    }
    Ok(result)
}

/// Prepends the length of the buffer to the buffer.
///
/// # Examples
///
/// ```
/// // let mut buffer = Vec::new();
/// // blt_utils::serialize_string("First", &mut buffer);
/// // blt_utils::serialize_string("Last", &mut buffer);
/// // blt_utils::serialize_u32(42, &mut buffer);
/// let mut buffer = [5, 0, 0, 0, 0, 0, 0, 0, 70, 105, 114, 115, 116, 4, 0, 0, 0, 0, 0, 0, 0, 76, 97, 115, 116, 42, 0, 0, 0, 0, 0, 0, 0].to_vec();
/// blt_utils::finalize_serialization(&mut buffer);
///
/// assert_eq!(buffer.as_slice(), [33, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 70, 105, 114, 115, 116, 4, 0, 0, 0, 0, 0, 0, 0, 76, 97, 115, 116, 42, 0, 0, 0, 0, 0, 0, 0]);
/// ```
pub fn finalize_serialization(buffer: &mut Vec<u8>) {
    let buffer_len = buffer.len();
    for (index, b) in buffer_len.to_le_bytes().iter().enumerate() {
        buffer.insert(index, *b);
    }
}

blt_macros::add_num!(u64, "u8");
blt_macros::add_num!(u64, "u16");
blt_macros::add_num!(u64, "u32");
blt_macros::add_num!(u64, "u64");
blt_macros::add_num!(u64, "u128");
blt_macros::add_num!(usize, "usize");

blt_macros::add_num!(i64, "i8");
blt_macros::add_num!(i64, "i16");
blt_macros::add_num!(i64, "i32");
blt_macros::add_num!(i64, "i64");
blt_macros::add_num!(i64, "i128");
blt_macros::add_num!(isize, "isize");

blt_macros::add_num!(f32, "f32");
blt_macros::add_num!(f64, "f64");

blt_macros::remove_num!(u64, "u8");
blt_macros::remove_num!(u64, "u16");
blt_macros::remove_num!(u64, "u32");
blt_macros::remove_num!(u64, "u64");
blt_macros::remove_num!(u64, "u128");
blt_macros::remove_num!(usize, "usize");

blt_macros::remove_num!(i64, "i8");
blt_macros::remove_num!(i64, "i16");
blt_macros::remove_num!(i64, "i32");
blt_macros::remove_num!(i64, "i64");
blt_macros::remove_num!(i64, "i128");
blt_macros::remove_num!(isize, "isize");

blt_macros::remove_num!(f32, "f32");
blt_macros::remove_num!(f64, "f64");

#[macro_use]
mod blt_macros {
    macro_rules! add_num {
        ($t: ty, $t_name: expr) => {
            paste::paste! {
                /// Adds the given numeric value to the buffer.
                pub fn [<serialize_ $t_name>](value: $t, buffer: &mut Vec<u8>) {
                    for b in value.to_le_bytes() {
                        buffer.push(b);
                    }
                }
            }
        };
    }

    macro_rules! remove_num {
        ($t: ty, $t_name: expr) => {
            paste::paste! {
                /// Removes the next numeric value from the buffer.
                /// If the buffer does not contain enough elements to create a numeric value, the buffer is unchanged and an error is returned.
                pub fn [<deserialize_ $t_name>](buffer: &mut Vec<u8>) -> Result<$t, DeserializationError> {
                    let t_len = std::mem::size_of::<$t>();
                    if t_len > buffer.len() {
                        return Err(DeserializationError::UnexpectedByteCount(
                            t_len,
                            buffer.len(),
                        ));
                    }
                    let remaining_bytes = buffer.split_off(t_len);
                    let result = $t::from_le_bytes(buffer.as_slice().try_into().unwrap());
                    *buffer = remaining_bytes;
                    Ok(result)
                }
            }
        };
    }

    pub(crate) use add_num;
    pub(crate) use remove_num;
}
