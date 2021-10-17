use dialoguer::theme::ColorfulTheme;
use dialoguer::{MultiSelect, Select};

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

    if let Confirmation::Yes = prompt_yes_or_no(question) {
        let selected_indices = MultiSelect::with_theme(&ColorfulTheme::default())
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

    let answer = Select::with_theme(&ColorfulTheme::default())
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
