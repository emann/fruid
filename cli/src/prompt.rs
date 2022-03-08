use std::fmt::{Debug, Display};
use std::str::FromStr;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

pub fn get_bool(prompt: &str, default: Option<bool>) -> bool {
    match default {
        Some(default_value) => Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default_value)
            .interact()
            .unwrap(),
        None => Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact()
            .unwrap(),
    }
}

pub fn get_string<T>(prompt: &str, default: Option<T>) -> String
where
    T: ToString,
{
    match default {
        Some(default_value) => Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default_value.to_string())
            .interact_text()
            .unwrap(),
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact_text()
            .unwrap(),
    }
}

pub fn get_parseable<T>(prompt: &str, default: Option<T>) -> T
where
    T: FromStr + Display,
    <T as FromStr>::Err: Debug + Display,
{
    let input_str = match default {
        Some(default_value) => Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .validate_with(|s: &String| s.parse::<T>().map(|_| ()))
            .default(default_value.to_string())
            .interact_text()
            .unwrap(),
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .validate_with(|s: &String| s.parse::<T>().map(|_| ()))
            .interact_text()
            .unwrap(),
    };

    input_str
        .parse::<T>()
        .expect("Parsability is checked by the input validator")
}

pub fn select<'a>(prompt: &str, mut choices: Vec<&'a str>) -> &'a str {
    let selection_index = Select::with_theme(&ColorfulTheme::default())
        .items(&choices)
        .with_prompt(prompt)
        .interact()
        .unwrap();

    choices.remove(selection_index)
}
