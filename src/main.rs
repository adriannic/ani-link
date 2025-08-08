use std::error::Error;

use ani_link::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::init()?;
    app.run()?;

    Ok(())
}
