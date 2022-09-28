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

pub fn string_find_next(s: &str, from: &str, to: &str) -> String {
    // Find substring of a string `s` from first occurence of `from` to first occurence of `to` after `from` was encountered
    // For example:
    // `s` = "Hello #my test" | `from` = "#" | `to` = " "
    // Returns "#my"
    if s.contains(from) {
        let split = s.split(from).nth(1).unwrap().split(to).next().unwrap();
        return format!("{}{}", from, split);
    }
    String::from("")
}

pub fn split_with_delim(s: &str, delim: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let s: String = s.to_string();

    let matched_indices: Vec<(usize, usize)> = s
        .match_indices(delim)
        .map(|(i, _)| (i, i + delim.len() - 1))
        .collect();
    let mut split_indices: Vec<(usize, usize)> = Vec::new();

    // string starts with delimiter
    if matched_indices.first().unwrap().0 != 0 {
        split_indices.push((0, matched_indices.first().unwrap().0 - 1));
    }

    for (i, indices) in matched_indices.iter().enumerate() {
        split_indices.push((indices.0, indices.1));
        if let Some(peeked_index) = matched_indices.get(i + 1) {
            split_indices.push((indices.1 + 1, peeked_index.0 - 1));
        }
    }

    // Delimiter is at the end
    if matched_indices.last().unwrap().1 != s.len() - 1 {
        split_indices.push((matched_indices.last().unwrap().1 + 1, s.len() - 1));
    }

    // What the hell was I thinking
    for (i, k) in &split_indices {
        let slice = &s[(*i)..=(*k)];
        result.push(slice.to_string());
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
