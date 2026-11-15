use crate::error::UpdateError;

pub fn parse(s: &str) -> Result<semver::Version, UpdateError> {
    semver::Version::parse(s).map_err(|_| UpdateError::BadVersion(s.to_string()))
}

/// `true`, wenn `candidate` neuer ist als `installed`.
pub fn is_newer(candidate: &str, installed: &str) -> Result<bool, UpdateError> {
    Ok(parse(candidate)? > parse(installed)?)
}
