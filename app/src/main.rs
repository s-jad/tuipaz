mod db;
mod tui;

use std::path::PathBuf;

use log::LevelFilter;
use log4rs::config::{Config, Root, Appender};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::append::file::FileAppender;
use color_eyre::Result;
use db::{db_mac::DbMac, init_db};
use dotenv::dotenv;
use tui::app::{run, App};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Initialize logging
    let path = PathBuf::from("./logs/application.log");
    match FileAppender::builder()
       .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L}{n}— {m}{n}")))
       .build(path) {
        Ok(file_appender) => {
            let config = Config::builder()
               .appender(Appender::builder().build("file", Box::new(file_appender))) // Now correctly passing a Box<dyn Append>
               .build(Root::builder().appender("file").build(LevelFilter::Info))?;

            log4rs::init_config(config).expect("Failed to initialize logger");
        },
        Err(e) => eprintln!("Failed to create file appender: {}", e),
    }

    tui::errors::install_hooks()?;
    let db = init_db::create_db().await?;
    let mut term = tui::utils::init()?;
    let note_titles = DbMac::load_note_identifiers(&db).await?;
    let mut app = App::new(db, note_titles);
    run(&mut app, &mut term).await?;
    tui::utils::restore()?;

    println!("app final state:\n{:#?}", app.note_list);

    Ok(())
}
