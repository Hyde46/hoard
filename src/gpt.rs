use crate::command::hoard_command::{string_to_tags, HoardCommand};
use chatgpt::prelude::*;

pub fn from_gpt_string(gpt_string: &str) -> HoardCommand {
    let mut cmd = HoardCommand::default();
    let mut name: String = "Something_went_wrong".to_owned();
    let mut tags: String = String::new();
    // If something goes wrong, we'll just use this as the description. So the user sees what's going on
    let mut description: String =
        format!("Something went wrong when parsing the GPT response.\n{gpt_string}");
    let mut command: String = String::new();
    let lines = gpt_string.lines();
    for line in lines {
        // Trying to anticipate how the GPT response will be formatted. Cant be guaranteed.
        if line.starts_with("name: > ") {
            name = line.strip_prefix("name: > ").unwrap().to_owned()
        } else if line.starts_with("name: ") {
            name = line.strip_prefix("name: ").unwrap().to_owned()
        } else if line.starts_with("explanation: > ") {
            description = line.strip_prefix("explanation: > ").unwrap().to_owned()
        } else if line.starts_with("explanation: ") {
            description = line.strip_prefix("explanation: ").unwrap().to_owned()
        } else if line.starts_with("tags: > ") {
            tags = line.strip_prefix("tags: > ").unwrap().to_owned();
            tags = tags.replace(' ', "");
        } else if line.starts_with("tags: ") {
            tags = line.strip_prefix("tags: ").unwrap().to_owned();
            tags = tags.replace(' ', "");
        } else if line.starts_with("command: > ") {
            command = line.strip_prefix("command: > ").unwrap().to_owned()
        } else if line.starts_with("command: ") {
            command = line.strip_prefix("command: ").unwrap().to_owned()
        }
    }
    cmd.name = name;
    cmd.description = Some(description);
    cmd.command = command.clone();
    cmd.tags = Some(string_to_tags(&tags));
    cmd.namespace = String::from("gpt");
    if command.is_empty() {
        cmd.description = Some(format!(
            "{}\n\nSomething probably went wrong parsing the GPT response:\n{gpt_string}",
            cmd.description.unwrap()
        ));
    }
    cmd
}

pub fn prompt(input: &str, key: &str) -> HoardCommand {
    let query_term = input;

    let formatted_command = format!(
        "
Write a linux command that does the following:
{query_term}

Reply with a made up name for the command ( Example: '> find_and_replace' ). 
Also a very short explanation without any formatting. ( Example: '> Does this thing')
After the the explanation add the command. ( Example: '> mv #file! #target!')
Come up with up to 3 tags for the command ( Example: '> git,filesystem' )
If there are any parameters, enclose them with !parameter_name#
This is the format how to reply:

name:<command name>

explanation:<short explanation>

tags: <tags>

command: <command>
    "
    );

    let key: String = String::from(key);
    let client = ChatGPT::new(key).unwrap();
    let resp = client.send_message(formatted_command);
    from_gpt_string(resp.unwrap().message_choices[0].message.content.as_str())
}
