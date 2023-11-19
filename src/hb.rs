use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};

pub fn flag(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    write!(out, "-flagssss")?;
    Ok(())
}
