
/// Converts a string of tags into a vector of tags
/// Tags are separated by commas
/// # Example
/// ```
/// use hoardlib::utils::string_to_tags;
/// 
/// let tags = string_to_tags("tag1,tag2,tag3");
/// assert_eq!(tags.len(), 3);
/// assert_eq!(tags[0], "tag1");
/// assert_eq!(tags[1], "tag2");
/// assert_eq!(tags[2], "tag3");
/// ```
pub fn string_to_tags(tags: &str) -> Vec<String> {
    tags.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .split(',')
        .map(std::string::ToString::to_string)
        .collect()
}

/// Find substring of a string `s` from first occurrence of `from` to first occurrence of `to` after `from` was encountered
/// # Example
/// ```
/// use hoardlib::utils::string_find_next;
///    
/// let s = "Hello #my test";
/// let from = "#";
/// let to = " ";
/// let expected = "#my";
/// assert_eq!(expected, string_find_next(&s, &from, &to));
/// ```
pub fn string_find_next(s: &str, from: &str, to: &str) -> String {

    if s.contains(from) {
        let split = s.split(from).nth(1).unwrap().split(to).next().unwrap();
        return format!("{from}{split}");
    }
    String::new()
}

/// Translates a number to its ordinal representation
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

/// Removes first and last character of a string
/// # Example
/// ```
/// use hoardlib::utils::rem_first_and_last;
/// 
/// let s = "Hello";
/// let expected = "ell";
/// assert_eq!(expected, rem_first_and_last(&s));
/// ```
pub fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

/// Splits a string without removing the delimiter
/// # Example
/// ```
/// use hoardlib::utils::split_with_delim;
/// 
/// let s = "Hello,World";
/// let expected = vec!["Hello", ",", "World"];
/// assert_eq!(expected, split_with_delim(&s, ","));
/// ```
pub fn split_with_delim(s: &str, delim: &str) -> Vec<String> {
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
        let s = String::from("#param test #param end");
        let expected = vec!["#param", " test ", "#param", " end"];
        assert_eq!(expected, split_with_delim(&s, "#param"));
    }

    #[test]
    fn test_numbers_to_nth() {
        assert_eq!("first", translate_number_to_nth(0));
        assert_eq!("second", translate_number_to_nth(1));
        assert_eq!("third", translate_number_to_nth(2));
        assert_eq!("fourth", translate_number_to_nth(3));
        assert_eq!("fifth", translate_number_to_nth(4));
        assert_eq!("sixth", translate_number_to_nth(5));
        assert_eq!("seventh", translate_number_to_nth(6));
        assert_eq!("eighth", translate_number_to_nth(7));
        assert_eq!("nth", translate_number_to_nth(8));
    }
}
