/**
@module SPECIAL.CONFIG.VERSION
Defines supported `special.toml` version values and version-selection parsing for config-driven dialect behavior.
*/
// @fileimplements SPECIAL.CONFIG.VERSION
use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecialVersion {
    #[default]
    V0,
    V1,
}

impl SpecialVersion {
    pub const CURRENT: Self = Self::V1;

    pub fn as_str(self) -> &'static str {
        match self {
            Self::V0 => "0",
            Self::V1 => "1",
        }
    }

    pub fn parse(value: &str, line: Option<usize>) -> Result<Self> {
        match value {
            "0" => Ok(Self::V0),
            "1" => Ok(Self::V1),
            _ => {
                if let Some(line) = line {
                    bail!("line {line} uses unsupported `special.toml` version `{value}`");
                }
                bail!("unsupported `special.toml` version `{value}`");
            }
        }
    }
}
