pub mod archive;

use crate::context::Context;
use crate::manifest::Manifest;
use crate::result::Result;

#[allow(dead_code)]
pub fn build(ctx: &Context, manifest: &Manifest) -> Result<()> {
    archive::create_tar_gz(ctx, manifest)
}
