mod db;
mod tui;

use color_eyre::Result;
use db::{db_mac::DbMac, init_db};
use dotenv::dotenv;
use tui::app::{run, App};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tui::errors::install_hooks()?;
    let db = init_db::create_db().await?;
    let mut term = tui::utils::init()?;
    let note_titles = DbMac::load_note_identifiers(&db).await?;
    let mut app = App::new(db, note_titles);
    let res = run(&mut app, &mut term).await?;
    tui::utils::restore()?;

    println!("app final state:\n{:#?}", app.note_list);

    return Ok(res);
}
