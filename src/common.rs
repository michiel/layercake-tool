use handlebars::{handlebars_helper, Handlebars};
use serde_json::Value;

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn write_string_to_file(filename: &str, content: &str) -> std::io::Result<()> {
    let path = Path::new(filename);
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn get_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();

    handlebars_helper!(exists: |v: Value| !v.is_null());
    handlebars.register_helper("exists", Box::new(exists));

    handlebars_helper!(isnull: |v: Value| v.is_null());
    handlebars.register_helper("isnull", Box::new(isnull));

    handlebars_helper!(stringeq: |s1: String, s2: String| s1.eq(&s2));
    handlebars.register_helper("stringeq", Box::new(stringeq));

    handlebars
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn handlebars_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template("Hello {{name}}", &json!({"name": "foo"}))
            .expect("This to render");
        assert_eq!(res, "Hello foo");
    }

    #[test]
    fn handlebars_can_iterate() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#each names as |name|}}
Hello {{name}}
{{/each}}"#,
                &json!({"names": ["foo", "bar", "baz"]}),
            )
            .expect("This to render");
        assert_eq!(res, "Hello foo\nHello bar\nHello baz\n");
    }

    #[test]
    fn handlebars_can_iterate_objects() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#each people as |person|}}
Hello {{person.name}}
{{/each}}"#,
                &json!({"people": [
                {
                    "name": "foo"
                },
                {
                    "name": "bar"
                },
                {
                    "name": "baz"
                }
                ]}),
            )
            .expect("This to render");
        assert_eq!(res, "Hello foo\nHello bar\nHello baz\n");
    }

    #[test]
    fn handlebars_helper_stringeq_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#if (stringeq "A label" node.label) }}
  {{node.label}};
{{/if}}"#,
                &json!({
                    "node": {
                        "label": "A label",
                    }
                }),
            )
            .expect("This to render");
        assert_eq!(res, "  A label;\n");
    }

    #[test]
    fn handlebars_helper_isnull_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#if (isnull node.id) }}
  {{node.label}};
{{/if}}"#,
                &json!({
                    "node": {
                        "label": "A label"
                    }
                }),
            )
            .expect("This to render");
        assert_eq!(res, "  A label;\n");
    }
}
