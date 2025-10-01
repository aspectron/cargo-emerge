pub mod archive;

use crate::context::Context;
use crate::manifest::Manifest;
use crate::result::Result;

pub fn build(ctx: &Context, manifest: &Manifest) -> Result<()> {
    archive::create_zip(ctx, manifest)
}

