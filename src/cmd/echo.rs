pub fn echo(args: &[String]) -> Result<String, String> {
    Ok(args.join(""))
}
