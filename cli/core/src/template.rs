use crate::error::TemplateError;
use handlebars::Handlebars;
use handlebars::{RenderError, RenderErrorReason};
use serde::Serialize;
use std::sync::OnceLock;

fn get_engine() -> &'static Handlebars<'static> {
    static ENGINE: OnceLock<Handlebars> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut hb = Handlebars::new();
        hb.register_helper("default", Box::new(default_helper));
        hb.register_helper("format", Box::new(format_helper));
        hb
    })
}

pub fn render(tmpl: &str, data: &impl Serialize) -> Result<String, TemplateError> {
    let hb = get_engine();
    Ok(hb.render_template(tmpl, data)?)
}

fn default_helper(
    h: &handlebars::Helper<'_>,
    _: &handlebars::Handlebars<'_>,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| {
        RenderError::from(RenderErrorReason::Other(
            "default helper requires 1 argument".into(),
        ))
    })?;
    let value = param.value().as_str().unwrap_or("");
    if value.is_empty() {
        out.write("unknown")?;
    } else {
        out.write(value)?;
    }
    Ok(())
}

fn format_helper(
    h: &handlebars::Helper<'_>,
    _: &handlebars::Handlebars<'_>,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let value = h
        .param(0)
        .ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(
                "format helper requires at least 1 argument".into(),
            ))
        })?
        .value();
    let width_spec = h
        .param(1)
        .map(|p| p.value().as_str().unwrap_or(""))
        .unwrap_or("");
    let rendered = match value {
        handlebars::JsonValue::Number(n) => {
            let num: u64 = n.as_u64().unwrap_or(0);
            let stripped = width_spec.trim_start_matches('0');
            let width: usize = if stripped.is_empty() {
                width_spec.len()
            } else {
                stripped.parse().unwrap_or(0)
            };
            if width > 0 {
                format!("{num:0>width$}")
            } else {
                num.to_string()
            }
        }
        _ => value.to_string(),
    };
    out.write(&rendered)?;
    Ok(())
}
