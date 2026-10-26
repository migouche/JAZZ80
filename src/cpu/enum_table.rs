//! Declarative Z80 encoding tables.
//!
//! The CPU decoder uses several small enums whose numeric encoding, display
//! text and parsed text are all defined by the Z80 instruction set. This
//! module provides a single macro that turns an explicit table into the enum
//! plus its `TryFrom<u8>`, `Display` and `FromStr` implementations.

/// Define an enum and its string/numeric conversions from one table.
///
/// The generated `TryFrom<u8>` maps each value to its variant, `Display`
/// writes the table text, and `FromStr` maps that exact text back to the
/// variant. Derives and documentation stay on the enum declaration:
///
/// ```ignore
/// table_enum! {
///     /// Optional doc comment.
///     #[derive(Debug, Clone, Copy)]
///     pub enum Example {
///         First = 0 => "FIRST",
///         Second = 1 => "SECOND",
///     }
/// }
/// ```
macro_rules! table_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $variant:ident = $value:literal => $text:literal,
            )*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant = $value,)*
        }

        impl TryFrom<u8> for $name {
            type Error = String;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok($name::$variant),)*
                    _ => Err(format!("Invalid {} value: {}", stringify!($name), value)),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($name::$variant => f.write_str($text),)*
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok($name::$variant),)*
                    _ => Err(format!("Invalid {}: {}", stringify!($name), s)),
                }
            }
        }
    };
}

pub(crate) use table_enum;
