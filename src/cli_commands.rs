use clap::{
    Arg, ArgAction, ArgMatches, Args, Command, Error, FromArgMatches, Id, Parser, Subcommand,
    ValueEnum,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Only used when `hoard` is run as shell plugin
    #[arg(long)]
    pub autocomplete: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    /// Register new hoard account
    Register,
    /// Log into your hoard account
    Login,
    /// Log out from your hoard account
    Logout,
    /// Push your local trove file to the synchronization server
    Save,
    /// Get your online trove file and merge it with your local one
    Get,
    /// Revert the last `hoard sync get` command
    Revert,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ListFormat {
    /// Format commands as table
    simple,
    /// Format commands as JSON
    json,
    /// Format commands as YAML
    yaml,
}

impl FromArgMatches for ListFormat {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let mut matches = matches.clone();
        Self::from_arg_matches_mut(&mut matches)
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, Error> {
        match (
            matches.get_flag("simple"),
            matches.get_flag("json"),
            matches.get_flag("yaml"),
        ) {
            (_, false, false) => Ok(Self::simple),
            (false, true, false) => Ok(Self::json),
            (false, false, true) => Ok(Self::yaml),
            (_, _, _) => Err(Error::raw(
                clap::error::ErrorKind::ValueValidation,
                "cannot specify more than one of 'simple', 'json', or 'yaml' at once",
            )),
        }
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        let mut matches = matches.clone();
        self.update_from_arg_matches_mut(&mut matches)
    }

    fn update_from_arg_matches_mut(&mut self, matches: &mut ArgMatches) -> Result<(), Error> {
        match (
            matches.get_flag("simple"),
            matches.get_flag("json"),
            matches.get_flag("yaml"),
        ) {
            (_, false, false) => {
                *self = Self::simple;
                Ok(())
            }
            (false, true, false) => {
                *self = Self::json;
                Ok(())
            }
            (false, false, true) => {
                *self = Self::yaml;
                Ok(())
            }
            (_, _, _) => Err(Error::raw(
                clap::error::ErrorKind::ValueValidation,
                "cannot specify more than one of 'simple', 'json', or 'yaml' at once",
            )),
        }
    }
}

impl Args for ListFormat {
    fn augment_args(cmd: Command) -> Command {
        cmd.arg(
            Arg::new("simple")
                .short('s')
                .long("simple")
                .action(ArgAction::SetTrue)
                .groups(["list"]),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .action(ArgAction::SetTrue)
                .groups(["list"]),
        )
        .arg(
            Arg::new("yaml")
                .short('y')
                .long("yaml")
                .action(ArgAction::SetTrue)
                .groups(["list"]),
        )
    }
    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
    fn group_id() -> Option<clap::Id> {
        Some(Id::from("list"))
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Shows setting file paths
    Info {},

    /// Save a new command
    New {
        /// [Optional] Name of the new command
        #[arg(short = 'n', long, value_name = "NAME")]
        name: Option<String>,

        /// [Optional] Supply single word tags for the newly created command, comma separated
        #[arg(short = 't', long, value_name = "TAGS")]
        tags: Option<String>,

        /// [Optional] The command to save
        #[arg(short = 'c', long, value_name = "COMMAND")]
        command: Option<String>,

        /// [Optional] Description of what the command does
        #[arg(short = 'd', long, value_name = "DESCRIPTION")]
        description: Option<String>,
    },

    /// List commands saved in trove.
    List {
        /// Apply filter to listed commands
        #[arg(short = 'f', long)]
        filter: Option<String>,

        /// Select which format to output commands. Default is "simple", which formats commands as a table
        #[command(flatten)]
        format: Option<ListFormat>,
    },

    /// Pick a command of the trove and print it
    Pick {
        /// Name of the command to print
        #[arg(short = 'n', long)]
        name: String,
    },

    /// Set a custom parameter token
    SetParameterToken {
        /// Parameter token to replace
        #[arg(long)]
        name: String,
    },

    /// Removes a command in the trove by name
    Remove {
        /// command to remove
        #[arg(short = 'n', long)]
        name: String,
    },

    /// Remove all commands of a namespace
    RemoveNamespace {
        /// Namespace to remove
        #[arg(short = 'n', long)]
        namespace: String,
    },

    /// Import a trove file from a local file or URL
    Import {
        /// URL or path to .trove file to import
        #[arg(long)]
        uri: String,
    },

    /// Export a trove file
    Export {
        /// Path where the trove file should be saved to
        #[arg(long)]
        path: String,
    },

    /// Edit a saved command
    Edit {
        /// Name of the command to edit
        #[arg(short = 'n', long)]
        name: String,
    },

    /// Print shell config
    ShellConfig {
        /// shell type to print the config for
        #[arg(short = 's', long)]
        shell: String,
    },

    /// Synchronize your trove file on multiple clients. If no mode is selected, it will fetch your online trove file and synchronize it with your local trove file
    Sync {
        ///
        #[arg(value_enum)]
        command: Mode,
    },
}
