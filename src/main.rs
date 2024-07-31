use std::io::Error;

fn main() -> Result<(), Error> {
    let (action, file_path) = tdt::config();

    tdt::run(action, file_path?)?;

    Ok(())
}
