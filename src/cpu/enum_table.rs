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
