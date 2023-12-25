use regex::Regex;

use crate::gui::prompts::prompt_input;
use crate::core::HoardCmd;

pub trait Parameterized {
    /// Check if parameter pointers are present
    /// For example, given subject with parameter token '#1':
    /// 'This is a #1 with one parameter token'
    /// `is_parameterized("#")` returns `true`
    fn is_parameterized(&self, token: &str) -> bool;
    /// Count number of parameter pointers
    /// For example, given subject with parameter token '#1':
    /// 'This is a #1 with one parameter token'
    /// `get_parameter_count("#")` returns `1`
    fn get_parameter_count(&self, token: &str) -> usize;
    /// Split subject into vector of Strings
    /// For example, given subject with parameter token '#1':
    /// 'This is a #1 with one parameter token'
    /// `split("#")` returns
    /// Vec["This is a ", " with one parameter token"]
    fn split(&self, token: &str) -> Vec<String>;
    /// Get parameterized String like subject including parameter token
    /// For example, given subject with parameter token '#1':
    /// 'This is a #1 with one parameter token'
    /// `get_split_subject("#")` returns
    /// Vec["This is a ", "#", " with one parameter token"]
    fn split_inclusive_token(&self, token: &str) -> Vec<String>;
    /// Replaces parameter tokens with content from `parameters`,
    /// consuming entries one by one until `parameters` is empty.
    fn replace_parameter(&self, token: &str, ending_token: &str, parameter: &str) -> HoardCmd;
    /// Prompts user for input parameters
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

    fn replace_parameter(&self, start_token: &str, end_token: &str, replacement: &str) -> Self {
        let pattern = format!("{}.*?{}", regex::escape(start_token), regex::escape(end_token));
        let re = Regex::new(&pattern).unwrap();
        let replaced = re.replace_all(&self.command, replacement);
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
            self.command = self.replace_parameter(token, ending_token, &parameter).command;
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