use crate::command::HoardCommand;
use crate::utils::string_find_next;

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
    /// Vec['This is a ', ' with one parameter token']
    fn split(&self, token: &str) -> Vec<String>;
    /// Get parameterized String like subject including parameter token
    /// For example, given subject with parameter token '#1':
    /// 'This is a #1 with one parameter token'
    /// `get_split_subject("#")` returns
    /// Vec['This is a ', '#', ' with one parameter token']
    fn split_inclusive_token(&self, token: &str) -> Vec<String>;
    /// Replaces parameter tokens with content from `parameters`,
    /// consuming entries one by one until `parameters` is empty.
    fn replace_parameter(&self, token: &str, ending_token: &str, parameter: &str) -> HoardCommand;
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

    fn replace_parameter(&self, token: &str, ending_token: &str, parameter: &str) -> HoardCommand {
        let parameter_array = &[parameter.clone()];
        let mut parameter_iter = parameter_array.iter();
        // Named parameter ending with a space
        let named_token = string_find_next(&self.command, token, " ");
        // Named parameter ending with ending token. If ending token is not used, `full_named_token` is an empty string
        let mut full_named_token = string_find_next(&self.command, token, ending_token);
        full_named_token.push_str(ending_token);
        // Select the split based on whether the ending token is part of the command or not
        let split_token = if self.command.contains(ending_token) {
            full_named_token
        } else {
            named_token
        };
        let split = self.split(&split_token);
        let mut collected: Vec<String> = Vec::new();
        for s in split {
            collected.push(s.clone());

            // if token is not named replace following occurrences of the token in the command with the token again.
            // only replace all occurrences of a token if it is names
            // this is a convoluted way of achieving this, but doing it properly would need this method to be completely reworked
            let to_push = if split_token == token {
                token
            } else {
                parameter
            };
            collected.push(parameter_iter.next().unwrap_or(&to_push).to_string());
        }
        // Always places either a token or the parameter at the end, due to the bad loop design.
        // Just remove it at the end
        collected.pop();
        let mut self_clone = self.clone();
        self_clone.command = collected.concat();

        return self_clone;
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn test_get_parameter_count() {
        let command = HoardCommand::default().with_command("test test test");
        assert_eq!(3, command.get_parameter_count("test"));
    }

    #[test]
    fn test_split() {
        let command = HoardCommand::default().with_command("test1 test2 test3");
        let expected = vec!["test1", "test2", "test3"];
        assert_eq!(expected, command.split(" "));
    }

    #[test]
    fn test_split_inclusive_token() {
        let command = HoardCommand::default().with_command("test1 test2 test3");
        let expected = vec!["test1", " ", "test2", " ", "test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }
    #[test]
    fn test_split_inclusive_token_multiple_spaces() {
        let command = HoardCommand::default().with_command("test1   test2   test3");
        let expected = vec!["test1", "   ", "test2", "   ", "test3"];
        assert_eq!(expected, command.split_inclusive_token("   "));
    }

    #[test]
    fn test_split_inclusive_token_no_spaces() {
        let command = HoardCommand::default().with_command("test1test2test3");
        let expected = vec!["test1test2test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_split_inclusive_token_special_characters() {
        let command = HoardCommand::default().with_command("test1@test2@test3");
        let expected = vec!["test1", "@", "test2", "@", "test3"];
        assert_eq!(expected, command.split_inclusive_token("@"));
    }
    #[test]
    fn test_split_inclusive_token_start() {
        let command = HoardCommand::default().with_command(" test1 test2 test3");
        let expected = vec![" ", "test1", " ", "test2", " ", "test3"];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_split_inclusive_token_end() {
        let command = HoardCommand::default().with_command("test1 test2 test3 ");
        let expected = vec!["test1", " ", "test2", " ", "test3", " "];
        assert_eq!(expected, command.split_inclusive_token(" "));
    }

    #[test]
    fn test_replace_parameter() {
        let command = HoardCommand::default().with_command("test1 # test3");
        let expected = HoardCommand::default().with_command("test1 replacement test3");
        assert_eq!(expected, command.replace_parameter("#", "", "replacement"));
    }
}
