use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ThemeCommands {
    #[command(alias = "c")]
    Check {
        #[arg(short = 'A', long)]
        all: bool,

        #[arg(short, long)]
        verbose: bool,
    },

    #[command(alias = "o")]
    Open,

    #[command(alias = "new")]
    Create {
        name: String,

        #[arg(short, long, default_value = "blank")]
        template: String,

        #[arg(short, long)]
        base: Option<String>,
    },

    #[command(alias = "ls")]
    List {
        #[arg(long)]
        builtin: bool,

        #[arg(long)]
        user: bool,

        #[arg(long)]
        iterm2: bool,

        #[arg(long)]
        search: Option<String>,
    },

    #[command(alias = "i")]
    Import {
        scheme: String,

        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(alias = "f")]
    Fetch {
        #[arg(short, long)]
        force: bool,
    },

    Clean {
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
