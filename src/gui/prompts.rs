use crate::gui::theme::HoardTheme;
use dialoguer::{Input, MultiSelect, Password, Select};
pub enum Confirmation {
    Yes,
    No,
}

pub fn prompt_multiselect_options<F, T, S>(
    question: &str,
    selection_prompt: &str,
    options: &[T],
    text_extractor: F,
) -> Vec<T>
where
    F: FnMut(&T) -> S,
    T: Clone,
    S: ToString,
{
    let options_texts: Vec<S> = options.iter().map(text_extractor).collect();

    if matches!(prompt_yes_or_no(question), Confirmation::Yes) {
        let selected_indices = MultiSelect::with_theme(&HoardTheme::default())
            .with_prompt(selection_prompt)
            .items(&options_texts)
            .interact()
            .unwrap();

        take_elements_by_indices(options, &selected_indices)
    } else {
        options.to_vec()
    }
}

pub fn prompt_yes_or_no(text: &str) -> Confirmation {
    const YES_ANSWER: usize = 0;

    let answer = Select::with_theme(&HoardTheme::default())
        .with_prompt(text)
        .items(&["Yes", "No"])
        .default(YES_ANSWER)
        .interact()
        .unwrap();

    if answer == YES_ANSWER {
        Confirmation::Yes
    } else {
        Confirmation::No
    }
}

pub fn prompt_select_with_options(text_prompt: &str, options: &[&str]) -> usize {
    Select::with_theme(&HoardTheme::default())
        .with_prompt(text_prompt)
        .items(options)
        .default(0)
        .interact()
        .unwrap()
}

pub fn prompt_input(text: &str, allow_empty: bool, default_value: Option<String>) -> String {
    // Just calls `prompt_input_validate` to not keep on typing `None` for the validator
    prompt_input_validate(
        text,
        allow_empty,
        default_value,
        None::<Box<dyn FnMut(&String) -> Result<(), String>>>,
    )
}

pub fn prompt_input_validate<F>(
    text: &str,
    allow_empty: bool,
    default_value: Option<String>,
    validator: Option<F>,
) -> String
where
    F: FnMut(&String) -> Result<(), String>,
{
    let theme = HoardTheme::default();
    let mut input: Input<String> = Input::with_theme(&theme);
    // Add default value to input prompt
    if let Some(val) = default_value {
        input.default(val);
    }
    // Add validator if any
    if let Some(val) = validator {
        input.validate_with(val);
    }
    input.allow_empty(allow_empty);
    input.with_prompt(text).interact_text().unwrap()
}

pub fn prompt_password_repeat(text: &str) -> String {
    Password::with_theme(&HoardTheme::default())
        .with_prompt(text)
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .interact()
        .unwrap()
}

pub fn prompt_password(text: &str) -> String {
    Password::with_theme(&HoardTheme::default())
        .with_prompt(text)
        .interact()
        .unwrap()
}

fn take_elements_by_indices<T>(elements: &[T], indices: &[usize]) -> Vec<T>
where
    T: Clone,
{
    elements
        .iter()
        .enumerate()
        .filter_map(|(i, val)| {
            if indices.contains(&i) {
                Some(val.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod test_prompts {
    use super::*;

    #[test]
    fn elements_by_indices() {
        let items = vec!["item1", "item2", "item3", "item4"];
        let indices = vec![1, 3];
        let expected_items = vec![items[1], items[3]];

        assert_eq!(expected_items, take_elements_by_indices(&items, &indices));
    }
}
