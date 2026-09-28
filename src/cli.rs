use crate::config::Config;
use crate::error::Result;
use crate::store;

pub fn list(config: &Config) -> Result<()> {
    store::list_slugs(&config.dir)?
        .iter()
        .for_each(|slug| println!("{slug}"));
    Ok(())
}

pub fn show(config: &Config, slug: &str) -> Result<()> {
    println!("{}", store::read(&config.dir, slug)?);
    Ok(())
}

pub fn path(config: &Config) -> Result<()> {
    println!("{}", config.dir.display());
    Ok(())
}
