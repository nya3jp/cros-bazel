use anyhow::{bail, Result};

pub fn split_key_value(spec: &str) -> Result<(&str, &str)> {
    let v: Vec<_> = spec.split("=").collect();
    if v.len() != 2 {
        bail!("invalid spec: {:?}", spec);
    }
    return Ok((v[0], v[1]));
}
