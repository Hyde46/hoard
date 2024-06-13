use regex::Regex;

use crate::core::HoardCmd;
use crate::control::prompts::prompt_input;

pub trait Parameterized {
    /// Checks if the command string contains a specific token.
    ///
    /// This function takes a token and checks if the command string contains this token.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice that holds the token to be checked.
    ///
    /// # Returns
    ///
    /// This function returns a boolean. It returns true if the command string contains the token,
    /// and false otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default()::with_command("echo $");
    /// assert!(command.is_parameterized("$"));
    /// ```

    fn is_parameterized(&self, token: &str) -> bool;
    /// Counts the number of occurrences of a specific token in the command string.
    ///
    /// This function takes a token and counts how many times this token appears in the command string.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice that holds the token to be counted.
    ///
    /// # Returns
    ///
    /// This function returns a usize representing the number of times the token appears in the command string.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default()::with_command("echo $ $");
    /// assert_eq!(command.get_parameter_count("$"), 2);
    /// ```

    fn get_parameter_count(&self, token: &str) -> usize;
    /// Splits the command string into a vector of substrings at each occurrence of a specific token.
    ///
    /// This function takes a token and splits the command string into a vector of substrings
    /// where each split is made at the token. The token itself is not included in the resulting substrings.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice that holds the token at which to split the command string.
    ///
    /// # Returns
    ///
    /// This function returns a Vec<String> where each element is a substring of the original command string.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default()::with_command("This is a #1 with one parameter token");
    /// assert_eq!(command.split("#"), vec!["This is a ", " with one parameter token"]);
    /// ```
    fn split(&self, token: &str) -> Vec<String>;

    /// Splits the command string into a vector of substrings at each occurrence of a specific token including the toke.
    ///
    /// This function takes a token and splits the command string into a vector of substrings
    /// where each split is made at the token. The token itself is included in the resulting substrings.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice that holds the token at which to split the command string.
    ///
    /// # Returns
    ///
    /// This function returns a Vec<String> where each element is a substring of the original command string.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default()::with_command("This is a #1 with one parameter token");
    /// assert_eq!(command.split("#"), vec!["This is a ", "#1", " with one parameter token"]);
    /// ```
    fn split_inclusive_token(&self, token: &str) -> Vec<String>;

    /// Replaces a parameter, identified by start and end tokens, in the command string with a given value.
    ///
    /// This function takes start and end tokens, and a value. It constructs a regex pattern from the tokens,
    /// and replaces all occurrences of the pattern in the command string with the given value.
    ///
    /// # Arguments
    ///
    /// * `start_token` - A string slice that holds the start token of the parameter.
    /// * `end_token` - A string slice that holds the end token of the parameter.
    /// * `value` - A string slice that holds the value to replace the parameter with.
    ///
    /// # Returns
    ///
    /// This function returns a new instance of the command with the replaced parameter.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default()::with_command("echo #param1$");
    /// let replaced_command = command.replace_parameter("#", "$", "Hello, world!");
    /// assert_eq!(replaced_command.get_command(), "echo Hello, world!");
    /// ```
    fn replace_parameter(&self, token: &str, ending_token: &str, parameter: &str) -> HoardCmd;

    /// Replaces all occurrences of a parameter, identified by a token and an ending token, in the command string with user input.
    ///
    /// This function takes a token and an ending token. It prompts the user for input for each occurrence of the parameter
    /// in the command string and replaces the parameter with the user's input.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice that holds the token of the parameter.
    /// * `ending_token` - A string slice that holds the ending token of the parameter.
    ///
    /// # Returns
    ///
    /// This function returns a new instance of the command with the replaced parameters.
    ///
    /// # Example
    ///
    /// ```
    /// let mut command = HoardCmd::default()::with_command("echo #param1$");
    /// command = command.with_input_parameters("#", "$");
    /// // The user is prompted for input for each occurrence of the parameter.
    /// // The command string is updated with the user's input.
    /// ```
    fn with_input_parameters(&mut self, token: &str, ending_token: &str) -> HoardCmd;
}

impl Parameterized for HoardCmd {
    fn is_parameterized(&self, token: &str) -> bool {
        self.command.contains(token)
    }
    fn get_parameter_count(&self, token: &str) -> usize {
        self.command.matches(token).count()
    }
    fn split(&self, token: &str) -> Vec<String> {
        self.command.split(token).map(ToString::to_string).collect()
    }
    fn split_inclusive_token(&self, token: &str) -> Vec<String> {
        let split = self.split(token);
        let mut collected: Vec<String> = Vec::new();
        let len = split.len();
        for (i, s) in split.into_iter().enumerate() {
            if !s.is_empty() {
                collected.push(s);
            }
            if i != len - 1 {
                collected.push(token.to_string());
            }
        }
        collected
    }

    fn replace_parameter(&self, start_token: &str, end_token: &str, value: &str) -> Self {
        let pattern = format!(
            "{}.*?{}",
            regex::escape(start_token),
            regex::escape(end_token)
        );
        let re = Regex::new(&pattern).unwrap();
        let replaced = re.replace_all(&self.command, value);
        Self::default().with_command(&replaced)
    }

    fn with_input_parameters(&mut self, token: &str, ending_token: &str) -> Self {
        let mut param_count = 0;
        while self.get_parameter_count(token) != 0 {
            let prompt_dialog = format!(
                "Enter parameter({}) nr {} \n~> {}\n",
                token,
                (param_count + 1),
                self.command
            );
            let parameter = prompt_input(&prompt_dialog, false, None);
            self.command = self
                .replace_parameter(token, ending_token, &parameter)
                .command;
            param_count += 1;
        }
        self.clone()
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn test_get_parameter_count() {
        let command = HoardCmd::default().with_command("test test test");
        assert_eq!(3, command.get_parameter_count("test"));
    }

    #[test]
    fn test_split() {
        let command = HoardCmd::default().with_command("test1 test2 test3");
        let expected = vec!["test1", "test2", "test3"];
        assert_eq!(expected, command.split(" "));
    }

    #[test]
    fn test_split_inclusive_token() {
        let command = HoardCmd::default().with_command("test1 test2 test3");
        let expected = vec!["test1", " ", "test2", " ", "test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }
    #[test]
    fn test_split_inclusive_token_multiple_spaces() {
        let command = HoardCmd::default().with_command("test1   test2   test3");
        let expected = vec!["test1", "   ", "test2", "   ", "test3"];
        assert_eq!(expected, command.split_inclusive_token("   "));
    }

    #[test]
    fn test_split_inclusive_token_no_spaces() {
        let command = HoardCmd::default().with_command("test1test2test3");
        let expected = vec!["test1test2test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_split_inclusive_token_special_characters() {
        let command = HoardCmd::default().with_command("test1@test2@test3");
        let expected = vec!["test1", "@", "test2", "@", "test3"];
        assert_eq!(expected, command.split_inclusive_token("@"));
    }
    #[test]
    fn test_split_inclusive_token_start() {
        let command = HoardCmd::default().with_command(" test1 test2 test3");
        let expected = vec![" ", "test1", " ", "test2", " ", "test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_split_inclusive_token_end() {
        let command = HoardCmd::default().with_command("test1 test2 test3 ");
        let expected = vec!["test1", " ", "test2", " ", "test3", " "];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_replace_parameter() {
        let command = HoardCmd::default().with_command("test1 # test3");
        let expected = HoardCmd::default().with_command("test1 replacement test3");
        assert_eq!(expected, command.replace_parameter("#", "", "replacement"));
    }

    #[test]
    fn test_replace_parameter_with_endtoken() {
        let command = HoardCmd::default().with_command("test1 #thisisacommand! test3");
        let expected = HoardCmd::default().with_command("test1 replacement test3");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }

    #[test]
    fn test_replace_parameter_with_endtoken_no_spaces() {
        let command = HoardCmd::default().with_command("test1#thisisacommand!test3");
        let expected = HoardCmd::default().with_command("test1replacementtest3");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
}
