use dialoguer::{theme::ColorfulTheme, Input};

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub value: Option<String>,
}

impl Argument {
    pub fn new(name: String) -> Self {
        Self { name, value: None }
    }

    // validate should use panic ATM, change in the future
    pub fn validate(&self) {}

    pub fn interactive(&mut self) {
        let command_string: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Command to hoard")
            .interact_text()
            .unwrap();
        self.value = Some(command_string.to_string());
    }

    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Set the argument's value.
    pub fn set_value(&mut self, value: Option<String>) {
        self.value = value;
    }

    /// Get a reference to the argument's value.
    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }
}
