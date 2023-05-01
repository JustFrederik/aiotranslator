use crate::error::Error;

/// checks if limit exceeds query length and returns error if it does
pub fn input_limit_checker(query: &str, input_limit: u32) -> Result<(), Error> {
    if query.len() > input_limit as usize {
        return Err(Error::new_option(format!(
            "Input limit exceeded: {} by {}",
            input_limit,
            query.len() - input_limit as usize
        )));
    }
    Ok(())
}

/// moves error out of option
pub fn option_error<T>(value: Option<Result<T, Error>>) -> Result<Option<T>, Error> {
    Ok(match value {
        Some(v) => Some(v?),
        None => None,
    })
}
