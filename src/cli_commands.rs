use clap::{Parser, Subcommand, ValueEnum};

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
    /// Revert the last `hoard sync` command
    Revert,
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

        /// Return hoarded commands in structured format
        #[arg(short = 'j', long)]
        json: bool,

        /// Return hoarded commands in a simplified table view
        #[arg(short = 's', long)]
        simple: bool,
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

    /// Sync your trove file
    Sync {
        /// Synchronize your trove file on multiple clients. If no mode is selected, it will fetch your online trove file and synchronize it with your local trove file
        #[arg(value_enum)]
        command: Option<Mode>,
    },
}
