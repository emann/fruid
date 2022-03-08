#![warn(clippy::all)]
#![warn(clippy::correctness)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
pub mod prompt;

use std::fmt::Debug;

pub trait FromCLIInput
where
    Self: Sized,
{
    fn from_cli_input(prompt: &str, default: Option<Self>) -> Self;
}

impl FromCLIInput for String {
    fn from_cli_input(prompt: &str, default: Option<Self>) -> Self {
        prompt::get_string(prompt, default)
    }
}

impl FromCLIInput for bool {
    fn from_cli_input(prompt: &str, default: Option<Self>) -> Self {
        prompt::get_bool(prompt, default)
    }
}

impl<T: FromCLIInput + Debug + Copy, const N: usize> FromCLIInput for [T; N] {
    fn from_cli_input(prompt: &str, default: Option<Self>) -> Self {
        let default_values: [Option<T>; N] = match default {
            Some(default_vals) => default_vals.map(Some),
            None => [None; N],
        };
        let mut values: Vec<T> = Vec::with_capacity(N);
        for (i, default_value) in default_values.into_iter().enumerate() {
            let extended_prompt = format!("{} ({})", prompt, i);
            let value: T = FromCLIInput::from_cli_input(&extended_prompt, default_value);
            values.push(value);
        }
        values
            .try_into()
            .expect("The number of values obtained should match what is expected")
    }
}

macro_rules! impl_from_cli_input_for_parseable_type {
    ($typ: ident) => {
        impl FromCLIInput for $typ {
            fn from_cli_input(prompt: &str, default: Option<Self>) -> Self {
                prompt::get_parseable(prompt, default)
            }
        }

        // impl_parser_for_int_arrays!($typ);
    };
}

// uints
impl_from_cli_input_for_parseable_type!(u8);
impl_from_cli_input_for_parseable_type!(u16);
impl_from_cli_input_for_parseable_type!(u32);
impl_from_cli_input_for_parseable_type!(u64);
impl_from_cli_input_for_parseable_type!(u128);
impl_from_cli_input_for_parseable_type!(usize);
// ints
impl_from_cli_input_for_parseable_type!(i8);
impl_from_cli_input_for_parseable_type!(i16);
impl_from_cli_input_for_parseable_type!(i32);
impl_from_cli_input_for_parseable_type!(i64);
impl_from_cli_input_for_parseable_type!(i128);
impl_from_cli_input_for_parseable_type!(isize);
// floats
impl_from_cli_input_for_parseable_type!(f32);
impl_from_cli_input_for_parseable_type!(f64);
