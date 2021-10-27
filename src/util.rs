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
