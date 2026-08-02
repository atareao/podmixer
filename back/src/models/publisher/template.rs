use minijinja::Environment;

use super::super::Error;

pub struct TemplateContext {
    pub title: String,
    pub description: String,
    pub url: String,
}

pub struct TemplateRenderer;

impl TemplateRenderer {
    pub fn render(template_str: &str, ctx: &TemplateContext) -> Result<String, Error> {
        let mut env = Environment::new();
        env.add_filter("truncate", |value: String, length: usize| -> String {
            match value.char_indices().nth(length) {
                Some((idx, _)) => value[..idx].to_string(),
                None => value,
            }
        });
        env.add_filter("word_limit", |value: String, count: usize| -> String {
            value
                .split_whitespace()
                .take(count)
                .collect::<Vec<&str>>()
                .join(" ")
        });
        env.add_filter("strip_html", |value: String| -> String {
            let mut result = String::new();
            let mut in_tag = false;
            for c in value.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ => {
                        if !in_tag {
                            result.push(c);
                        }
                    }
                }
            }
            result
        });
        env.add_template("tpl", template_str)?;
        let tmpl = env.get_template("tpl")?;
        let result = tmpl.render(minijinja::context!(
            title => ctx.title.as_str(),
            description => ctx.description.as_str(),
            url => ctx.url.as_str(),
        ))?;
        Ok(result)
    }
}