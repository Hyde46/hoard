pub fn translate_number_to_nth(count: u16) -> String {
    match count {
        0 => "first".to_string(),
        1 => "second".to_string(),
        2 => "third".to_string(),
        3 => "fourth".to_string(),
        4 => "fifth".to_string(),
        5 => "sixth".to_string(),
        6 => "seventh".to_string(),
        7 => "eighth".to_string(),
        _ => "nth".to_string(),
    }
}

pub fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

pub fn string_find_next(s: &str, from: &str, to: &str) -> String {
    // Find substring of a string `s` from first occurence of `from` to first occurence of `to` after `from` was encountered
    // For example:
    // `s` = "Hello #my test" | `from` = "#" | `to` = " "
    // Returns "#my"
    if s.contains(from) {
        let split = s.split(from).nth(1).unwrap().split(to).next().unwrap();
        return format!("{from}{split}");
    }
    String::new()
}

pub fn split_with_delim(s: &str, delim: &str) -> Vec<String> {
    //Credits to chatGPT for simplifying this wtf
    let mut result = Vec::new();
    let mut start = 0;

    for (i, _) in s.match_indices(delim) {
        if i > start {
            result.push(s[start..i].to_string());
        }
        result.push(delim.to_string());
        start = i + delim.len();
    }

    if start < s.len() {
        result.push(s[start..].to_string());
    }

    while result.first().map_or(false, std::string::String::is_empty) {
        result.remove(0);
    }

    while result.last().map_or(false, std::string::String::is_empty) {
        result.pop();
    }

    result
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn extract_named_parameter() {
        let s = String::from("Hello #test command");
        let expected = "#test";
        assert_eq!(expected, string_find_next(&s, "#", " "));
    }

    #[test]
    fn extract_named_parameter_first() {
        let s = String::from("#test command");
        let expected = "#test";
        assert_eq!(expected, string_find_next(&s, "#", " "));
    }

    #[test]
    fn extract_named_parameter_last() {
        let s = String::from("command #test");
        let expected = "#test";
        assert_eq!(expected, string_find_next(&s, "#", " "));
    }

    #[test]
    fn extract_named_parameter_noname() {
        let s = String::from("test # bar");
        let expected = "#";
        assert_eq!(expected, string_find_next(&s, "#", " "));
    }

    #[test]
    fn extract_named_parameter_none() {
        let s = String::from("command test");
        let expected = "";
        assert_eq!(expected, string_find_next(&s, "#", " "));
    }

    #[test]
    fn test_split_with_delim() {
        let s = String::from("command #param test #param lol");
        let expected = vec!["command ", "#param", " test ", "#param", " lol"];
        assert_eq!(expected, split_with_delim(&s, "#param"));
    }

    #[test]
    fn test_split_with_delim_at_end() {
        let s = String::from("command #param test #param lol #param");
        let expected = vec!["command ", "#param", " test ", "#param", " lol ", "#param"];
        assert_eq!(expected, split_with_delim(&s, "#param"));
    }

    #[test]
    fn test_split_with_delim_at_start() {
        let s = String::from("#param test #param lol");
        let expected = vec!["#param", " test ", "#param", " lol"];
        assert_eq!(expected, split_with_delim(&s, "#param"));
    }
}
