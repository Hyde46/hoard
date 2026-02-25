use crate::core::HoardCmd;
use crate::gui::prompts::prompt_input;

pub trait Parameterized {
    fn escape_input(input: &str, start_token: &str, end_token: &str) -> String;
    fn cleanup_escapes(&self, start_token: &str, end_token: &str) -> HoardCmd;
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
    // Escapet einen String so, dass get_parameter_count ihn komplett ignoriert
    fn escape_input(input: &str, start_token: &str, end_token: &str) -> String {
        let mut escaped = String::with_capacity(input.len() * 2);
        let mut i = 0;

        while i < input.len() {
            if input.as_bytes()[i] == b'\\' {
                escaped.push_str("\\\\");
                i += 1;
                continue;
            }

            if input[i..].starts_with(start_token) {
                escaped.push('\\');
                escaped.push_str(start_token);
                i += start_token.len();
                continue;
            }

            if !end_token.is_empty() && input[i..].starts_with(end_token) {
                escaped.push('\\');
                escaped.push_str(end_token);
                i += end_token.len();
                continue;
            }

            let c = input[i..].chars().next().unwrap();
            escaped.push(c);
            i += c.len_utf8();
        }

        escaped
    }

    fn cleanup_escapes(&self, start_token: &str, end_token: &str) -> HoardCmd {
        let s = &self.command;
        let mut out = String::with_capacity(s.len());
        let mut i = 0;

        while i < s.len() {
            if s.as_bytes()[i] == b'\\' {
                i += 1;
                if i < s.len() {
                    if s[i..].starts_with(start_token) {
                        out.push_str(start_token);
                        i += start_token.len();
                    } else if s[i..].starts_with(end_token) && !end_token.is_empty() {
                        out.push_str(end_token);
                        i += end_token.len();
                    } else if s.as_bytes()[i] == b'\\' {
                        out.push('\\');
                        i += 1;
                    } else {
                        let c = s[i..].chars().next().unwrap();
                        out.push(c);
                        i += c.len_utf8();
                    }
                }
                continue;
            }

            let c = s[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }

        Self::default().with_command(&out)
    }

    fn get_parameter_count(&self, token: &str) -> usize {
        let s = &self.command;
        let mut count = 0;
        let mut i = 0;

        while i < s.len() {
            if s.as_bytes()[i] == b'\\' {
                i += 1;
                if i < s.len() {
                    if s[i..].starts_with(token) {
                        i += token.len();
                    } else {
                        let c = s[i..].chars().next().unwrap();
                        i += c.len_utf8();
                    }
                }
                continue;
            }

            if s[i..].starts_with(token) {
                count += 1;
                i += token.len();
                continue;
            }

            let c = s[i..].chars().next().unwrap();
            i += c.len_utf8();
        }
        count
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
        let s = &self.command;
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut replaced = false;

        while i < s.len() {
            if s.as_bytes()[i] == b'\\' {
                out.push('\\'); // Keep Backslash for final cleanup
                i += 1;
                if i < s.len() {
                    if s[i..].starts_with(start_token) {
                        out.push_str(start_token);
                        i += start_token.len();
                    } else if s[i..].starts_with(end_token) && !end_token.is_empty() {
                        out.push_str(end_token);
                        i += end_token.len();
                    } else {
                        let c = s[i..].chars().next().unwrap();
                        out.push(c);
                        i += c.len_utf8();
                    }
                }
                continue;
            }

            if !replaced && s[i..].starts_with(start_token) {
                let param_content_start = i + start_token.len();

                let mut search_idx = param_content_start;
                let mut found_end = None;

                while search_idx < s.len() {
                    if s.as_bytes()[search_idx] == b'\\' {
                        search_idx += 1;
                        if search_idx < s.len() {
                            let c = s[search_idx..].chars().next().unwrap();
                            search_idx += c.len_utf8();
                        }
                        continue;
                    }

                    if !end_token.is_empty() && s[search_idx..].starts_with(end_token) {
                        found_end = Some(search_idx);
                        break;
                    }

                    if s[search_idx..].starts_with(start_token) {
                        break;
                    }

                    let c = s[search_idx..].chars().next().unwrap();
                    search_idx += c.len_utf8();
                }

                if let Some(end_idx) = found_end {
                    out.push_str(value);
                    i = end_idx + end_token.len();
                    replaced = true;
                    continue;
                } else {
                    out.push_str(value);
                    i += start_token.len();
                    replaced = true;
                    continue;
                }
            }

            let c = s[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }

        Self::default().with_command(&out)
    }

    fn with_input_parameters(&mut self, token: &str, ending_token: &str) -> Self {
        let s = &self.command;
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut param_count = 0;

        while i < s.len() {
            if s.as_bytes()[i] == b'\\' {
                if i + 1 < s.len() {
                    let next_pos = i + 1;

                    if s[next_pos..].starts_with(token) {
                        out.push_str(token);
                        i = next_pos + token.len();
                        continue;
                    }

                    if s.as_bytes()[next_pos] == b'\\' {
                        out.push('\\');
                        i = next_pos + 1;
                        continue;
                    }
                }
                out.push('\\');
                i += 1;
                continue;
            }

            if s[i..].starts_with(token) {
                param_count += 1;
                let param_content_start = i + token.len();

                let current_preview = format!("{}{}[...]", out, &s[i..]);

                let prompt_dialog = format!(
                    "Enter parameter({}) nr {}\n~> {}\n",
                    token, param_count, current_preview
                );

                let user_input = prompt_input(&prompt_dialog, false, None);

                if let Some(end_offset) = s[param_content_start..].find(ending_token) {
                    out.push_str(&user_input);
                    i = param_content_start + end_offset + ending_token.len();
                    continue;
                } else {
                    out.push_str(&user_input);
                    i += token.len();
                    continue;
                }
            }

            let c = s[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }

        self.command = out;
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

    #[test]
    fn test_escape_double_backslash_before_token() {
        let command = HoardCmd::default().with_command("wewantto\\\\#escape");
        // Backslash-Cleanup happens later
        let expected = HoardCmd::default().with_command("wewantto\\\\replacementescape");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
    #[test]
    fn test_escape_single_backslash_before_token() {
        let command = HoardCmd::default().with_command("wewantto\\#escape");
        let expected = HoardCmd::default().with_command("wewantto\\#escape");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
    #[test]
    fn test_escape_no_backslash_before_token() {
        let command = HoardCmd::default().with_command("wewantto#!escape");
        let expected = HoardCmd::default().with_command("wewanttoreplacementescape");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
    #[test]
    fn test_escape_backslash_before_token_with_end() {
        let command = HoardCmd::default().with_command("wewantto\\##!escape");
        let expected = HoardCmd::default().with_command("wewantto\\#replacementescape");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
    #[test]
    fn test_escape_backslash_before_multiple_token_with_end() {
        let command = HoardCmd::default().with_command("wewantto\\##!escape##");
        // Only the first gets replaced in a single iteration
        let expected = HoardCmd::default().with_command("wewantto\\#replacementescape##");
        assert_eq!(expected, command.replace_parameter("#", "!", "replacement"));
    }
}
