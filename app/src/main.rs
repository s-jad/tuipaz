mod db;
mod tui;

use std::path::PathBuf;

use log::{LevelFilter, info};
use log4rs::config::{Config as LogConfig, Root, Appender};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::append::file::FileAppender;
use color_eyre::Result;
use db::{db_mac::DbMac, init_db};
use dotenv::dotenv;
use tui::app::{run, App};

use crate::tui::config::try_load_config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Initialize logging
    let path = PathBuf::from("./logs/application.log");
    match FileAppender::builder()
       .encoder(Box::new(PatternEncoder::new("{n}| {({l}):5.5}| {f}:{L}{n}{m}{n}")))
       .build(path) {
        Ok(file_appender) => {
            let config = LogConfig::builder()
               .appender(Appender::builder().build("file", Box::new(file_appender))) // Now correctly passing a Box<dyn Append>
               .build(Root::builder().appender("file").build(LevelFilter::Info))?;

            log4rs::init_config(config).expect("Failed to initialize logger");
        },
        Err(e) => eprintln!("Failed to create file appender: {}", e),
    }
    
    let seperator = "-".repeat(40);
    info!("{}NEW SESSION{}\n", seperator, seperator);
    tui::errors::install_hooks()?;
    let db = init_db::create_db().await?;
    let mut term = tui::utils::init()?;
    let note_titles = DbMac::load_note_identifiers(&db).await?;
    let term_size = term.size().expect("Terminal should have a size").width;
    let config: tui::config::Config = try_load_config("../config.toml")?;
    info!("config: {config:?}");

    let mut app = App::new(config, db, note_titles, term_size);
    run(&mut app, &mut term).await?;
    tui::utils::restore()?;
    info!("{}END SESSION{}\n", seperator, seperator);

    Ok(())
}
