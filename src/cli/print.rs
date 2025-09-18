use std::io::{self, Write, stderr, stdout};

pub fn print(s: &str) -> io::Result<()> {
    if let Err(err) = stdout().write_all(s.as_bytes()) {
        writeln!(stderr(), "{}", err)?;
    }

    if let Err(err) = stdout().flush() {
        writeln!(stderr(), "{}", err)?;
    }

    Ok(())
}

pub fn error(s: &str) -> io::Result<()> {
    if let Err(err) = stderr().write_all(s.as_bytes()) {
        writeln!(stderr(), "{}", err)?;
    }

    if let Err(err) = stderr().flush() {
        writeln!(stderr(), "{}", err)?;
    }

    Ok(())
}
