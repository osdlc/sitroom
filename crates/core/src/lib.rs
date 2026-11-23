//! Sitroom's core: the config file (`config`), the host that runs modules and
//! sends their views to renderers (`host`), the osdlc command-line conventions
//! (`cli_kit`), and markdown as styled lines for the renderers
//! (`markdown`).

pub mod cli_kit;
pub mod config;
pub mod host;
pub mod markdown;
