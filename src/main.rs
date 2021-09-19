use clap::{AppSettings, Clap};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

mod overcast;
mod sqlite;
use overcast::OvercastClient;

#[derive(Clap)]
#[clap(version = "1.0", author = "Ben Congdon <ben@congdon.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Overcast username.
    #[clap(short, long)]
    username: Option<String>,
    /// Overcast password.
    #[clap(short, long)]
    password: Option<String>,
    /// Storage location for Overcast credentials.
    #[clap(short, long, default_value = "auth.json")]
    auth_file: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Authenticate with Overcast")]
    Auth(Auth),
    #[clap(about = "Save Overcast feeds/episodes to sqlite")]
    Archive(Archive),
}

#[derive(Clap)]
struct Auth {}

#[derive(Clap)]
struct Archive {
    /// The sqlite database path to store to.
    db_path: String,
}

#[derive(Serialize, Deserialize)]
struct AuthFile {
    #[serde(rename = "overcast_username")]
    username: String,
    #[serde(rename = "overcast_password")]
    password: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let client = OvercastClient::new();

    match opts.subcmd {
        SubCommand::Auth(_) => auth_cmd(&client, &opts),
        SubCommand::Archive(Archive { ref db_path }) => archive_cmd(client, &opts, db_path.clone()),
    }
}

fn archive_cmd(
    client: OvercastClient,
    opts: &Opts,
    db_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[1/3] Authenticating with Overcast...");
    if let (Some(username), Some(password)) = (opts.username.clone(), opts.password.clone()) {
        client.authenticate(&username, &password)?
    } else if std::path::Path::new(&opts.auth_file).exists() {
        let auth_file = std::fs::File::open(opts.auth_file.clone())?;
        let auth: AuthFile = serde_json::from_reader(auth_file)?;
        client.authenticate(&auth.username, &auth.password)?;
    } else {
        return Err("No credentials provided. Run the `auth` subcommand first, or provide credentials with --username and --password.".into());
    }
    eprintln!("[2/3] Fetching podcasts...");
    let podcasts = client.get_podcasts()?;
    eprintln!(
        "Fetched {} feeds with a total of {} episodes.",
        podcasts.len(),
        podcasts.iter().map(|p| p.episodes.len()).sum::<usize>()
    );
    eprintln!("[3/3] Writing podcasts to sqlite db...");
    let conn = Connection::open(&db_path)?;
    sqlite::create_tables(&conn)?;
    sqlite::upsert_feeds(&conn, &podcasts)?;
    Ok(())
}

fn auth_cmd(client: &OvercastClient, opts: &Opts) -> Result<(), Box<dyn std::error::Error>> {
    let credentials =
        // Use credentials from CLI flags
        if let (Some(username), Some(password)) = (opts.username.clone(), opts.password.clone()) {
            AuthFile { username, password }
        }
        // Prompt for credentials
        else {
            let username = rpassword::prompt_password_stdout("Overcast username: ")?;
            let password = rpassword::prompt_password_stdout("Overcast password: ")?;
            AuthFile { username, password }
        };
    // TODO: Patch with existing file if one already exists.
    let mut file = std::fs::File::create(&opts.auth_file)?;
    serde_json::to_writer_pretty(&mut file, &credentials)?;
    client.authenticate(&credentials.username, &credentials.password)?;
    eprintln!("Authenticated successfully.");
    Ok(())
}
