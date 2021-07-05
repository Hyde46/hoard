pub trait Command {
    fn new(matches: &&clap::ArgMatches) -> Self;
    fn default() -> Self;
    fn is_complete(&self) -> bool;
}
