use crate::command::trove::CommandTrove;
use crate::gui::prompts::{prompt_input, prompt_input_validate};
use crate::util::string_find_next;
use serde::{Deserialize, Serialize};

pub trait Parsable {
    fn parse_arguments(matches: &clap::ArgMatches) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoardCommand {
    pub name: String,
    pub namespace: String,
    pub tags: Option<Vec<String>>,
    pub command: String,
    pub description: Option<String>,
}

impl HoardCommand {
    pub fn default() -> Self {
        Self {
            name: "".to_string(),
            namespace: "".to_string(),
            tags: None,
            command: "".to_string(),
            description: None,
        }
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        if self.name.is_empty()
            || self.namespace.is_empty()
            || self.tags.is_none()
            || self.command.is_empty()
            || self.description.is_none()
        {
            return false;
        }
        true
    }

    pub fn tags_as_string(&self) -> String {
        self.tags
            .as_ref()
            .unwrap_or(&vec!["".to_string()])
            .join(",")
    }

    #[allow(dead_code)]
    pub fn with_command_raw(self, command_string: &str) -> Self {
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: command_string.to_string(),
            description: self.description,
        }
    }

    pub fn with_command_string_input(
        self,
        default_value: Option<String>,
        parameter_token: &str,
    ) -> Self {
        let base_prompt = format!(
            "Command to hoard ( Mark unknown parameters with {} )\n",
            parameter_token
        );
        let command_string: String = prompt_input(&base_prompt, false, default_value);
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: command_string,
            description: self.description,
        }
    }

    pub fn with_tags_raw(self, tags: &str) -> Self {
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: Some(
                tags.chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            ),
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_tags_input(self, default_value: Option<String>) -> Self {
        let tag_validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("Tags cant contain whitespaces".to_string())
            } else {
                Ok(())
            }
        };
        let tags: String = prompt_input_validate(
            "Give your command some optional tags ( comma seperated )",
            true,
            default_value,
            Some(tag_validator),
        );
        self.with_tags_raw(&tags)
    }

    pub fn with_namespace_input(self, default_namespace: Option<String>) -> Self {
        let namespace: String = prompt_input("Namespace of the command", false, default_namespace);
        Self {
            name: self.name,
            namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    fn with_name_input_prompt(
        self,
        default_value: Option<String>,
        trove: &CommandTrove,
        prompt_string: &str,
    ) -> Self {
        let namespace = self.namespace.clone();
        let command_names = trove.commands.clone();
        let validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("The name cant contain whitespaces".to_string())
            } else if command_names
                .iter()
                .filter(|x| x.namespace == namespace)
                .any(|x| x.name == *input)
            {
                Err(
                    "A command with same name exists in the this namespace. Input a different name"
                        .to_string(),
                )
            } else {
                Ok(())
            }
        };
        let name = prompt_input_validate(prompt_string, false, default_value, Some(validator));
        Self {
            name,
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_name_input(self, default_value: Option<String>, trove: &CommandTrove) -> Self {
        self.with_name_input_prompt(default_value, trove, "Name your command")
    }

    pub fn with_alt_name_input(self, default_value: Option<String>, trove: &CommandTrove) -> Self {
        let name = self.name.clone();
        let command = self.command.clone();
        let namespace = self.namespace.clone();
        self.with_name_input_prompt(
            default_value,
            trove,
            &format!(
                "A command with same name already exists in the namespace '{}'. Enter an alternate name for '{}' with command `{}`",
                namespace,
                name,
                command
            ),
        )
    }

    pub fn with_description_input(self, default_value: Option<String>) -> Self {
        let description_string: String =
            prompt_input("Describe what the command does", false, default_value);
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: Some(description_string),
        }
    }
}

impl Parsable for HoardCommand {
    fn parse_arguments(matches: &clap::ArgMatches) -> Self {
        let mut new_command = Self::default();

        if let Some(n) = matches.value_of("name") {
            new_command.name = n.to_string();
        }
        // Defaults to 'default' namespace
        if let Some(ns) = matches.value_of("namespace") {
            new_command.namespace = ns.to_string();
        }
        // "$ hoard test -t" was run
        // Expects comma seperated tags
        if let Some(tags) = matches.value_of("tags") {
            new_command.tags = Some(
                tags.split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            );
        }
        if let Some(c) = matches.value_of("command") {
            new_command.command = c.to_string();
        }
        new_command
    }
}

pub trait Parameterized {
    // Check if parameter pointers are present
    fn is_parameterized(&self, token: &str) -> bool;
    // Count number of parameter pointers
    fn get_parameter_count(&self, token: &str) -> usize;
    fn split(&self, token: &str) -> Vec<String>;
    // Get parameterized Stringlike subject including parameter token
    // For example, given subject with parameter token '#1':
    // 'This is a #1 with one parameter token'
    // `get_split_subject("#")` returns
    // Vec['This is a ', '#', ' with one parameter token']
    fn get_split_subject(&self, token: &str) -> Vec<String>;
    // Replaces parameter tokens with content from `parameters`,
    // consuming entries one by one until `parameters` is empty.
    fn replace_parameter(self, token: &str, parameter: String) -> HoardCommand;

    fn with_input_parameters(self, token: &str) -> HoardCommand;
}

impl Parameterized for HoardCommand {
    fn is_parameterized(&self, token: &str) -> bool {
        self.command.contains(token)
    }
    fn get_parameter_count(&self, token: &str) -> usize {
        self.command.matches(token).count()
    }
    fn split(&self, token: &str) -> Vec<String> {
        self.command.split(token).map(ToString::to_string).collect()
    }
    fn get_split_subject(&self, token: &str) -> Vec<String> {
        let split = self.split(token);
        let mut collected: Vec<String> = Vec::new();
        for s in split {
            collected.push(s.clone());
            collected.push(token.to_string());
        }
        collected
    }

    //fn get_first_named_parameters(&self, token: &str) -> Vec<String> {
//
    //}

    fn replace_parameter(self, token: &str, parameter: String) -> HoardCommand {
        let parameter_array = &[parameter.clone()];
        let mut parameter_iter = parameter_array.iter();

        let named_token = string_find_next(&self.command, token, " ");
        let split = self.split(&named_token);
        let mut collected: Vec<String> = Vec::new();
        for s in split {
            collected.push(s.clone());
            
            // if token is not named replace following occurences of the token in the command with the token again.
            // only replace all occurences of a token if it is names
            // this is a convoluted way of achieving this, but doing it properly would need this method to be completely reworked
            let to_push = if named_token == token {token.to_string()} else {parameter.clone()};
            collected.push(parameter_iter.next().unwrap_or(&to_push).clone());
        }
        // Always places either a token or the parmeter at the end, due to the bad loop design. 
        // Just remove it at the end
        collected.pop();
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: collected.concat(),
            description: self.description,
        }
    }

    fn with_input_parameters(self, token: &str) -> HoardCommand {
        let parameter_count = self.get_parameter_count(token);
        if parameter_count == 0 {
            return self;
        }
        let mut command_state = self.command.clone();
        for i in 0..parameter_count {
            let prompt_dialoge = format!(
                "Enter parameter({}) nr {} \n~> {}\n",
                token,
                (i + 1),
                command_state
            );
            let parameter = prompt_input(&prompt_dialoge, false, None);
            command_state = command_state.replacen(token, &parameter, 1);
        }
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: command_state,
            description: self.description,
        }
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn one_tag_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = "foo";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn no_tag_as_string() {
        let command = HoardCommand::default();
        let expected = "";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn multiple_tags_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = "foo,bar";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = Some(vec!["foo".to_string()]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_no_tag() {
        let command = HoardCommand::default();
        let expected = None;
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_tags() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = Some(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCommand::default().with_tags_raw("foo, bar");
        let expected = Some(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(expected, command.tags);
    }
}

#[cfg(test)]
mod test_parameterized {
    use super::*;

    fn command_struct(command: &str) -> HoardCommand {
        HoardCommand::default().with_command_raw(command)
    }

    #[test]
    fn test_split() {
        let token = "#".to_string();
        let c: HoardCommand = command_struct("test # test");
        let expected = vec!["test ".to_string(), " test".to_string()];
        assert_eq!(expected, c.split(&token));
    }

    #[test]
    fn test_split_empty() {
        let token = "#".to_string();
        let c: HoardCommand = command_struct("test  test");
        let expected = vec!["test  test".to_string()];
        assert_eq!(expected, c.split(&token));
    }

    #[test]
    fn test_split_multiple() {
        let token = "#".to_string();
        let c: HoardCommand = command_struct("test # test #");
        let expected = vec!["test ".to_string(), " test ".to_string(), "".to_string()];
        assert_eq!(expected, c.split(&token));
    }

    #[test]
    fn test_replace_parameter() {
        let token = "#".to_string();
        let c: HoardCommand = command_struct("test # bar");
        let to_replace = "foo".to_string();
        let expected = "test foo bar".to_string();
        assert_eq!(expected, c.replace_parameter(&token, to_replace).command);
    }

    #[test]
    fn test_replace_last_parameter() {
        let token = "#".to_string();
        let c: HoardCommand = command_struct("test foo #");
        let to_replace = "bar".to_string();
        let expected = "test foo bar".to_string();
        assert_eq!(expected, c.replace_parameter(&token, to_replace).command);
    }
}
