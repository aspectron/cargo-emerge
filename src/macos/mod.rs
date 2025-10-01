pub mod dmg;

use crate::context::Context;
use crate::manifest::Manifest;
use crate::result::Result;

#[allow(dead_code)]
pub fn build(ctx: &Context, manifest: &Manifest) -> Result<()> {
    dmg::create(ctx, manifest)
}
