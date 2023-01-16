use std::collections::HashMap;

use once_cell::sync::Lazy;
use tera::{Context, Tera};

pub fn generate_i18n(structs: String, codes: HashMap<String, String>) -> String {
    let mut ctx = Context::new();
    ctx.insert("structs", &structs);
    ctx.insert("codes", &codes);

    TERA.render("i18n", &ctx).unwrap()
}

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "i18n",
        r#"
    {% raw %}
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    mod trans {
        use std::{cmp::min, collections::HashMap};

        #[allow(dead_code)]
        fn message_index(size: &usize, num: &usize) -> usize {
            if *size == 2 {
                if *num == 1 {
                    0
                } else {
                    1
                }
            } else {
                min(size - 1, *num)
            }
        }

        {% endraw %}{{ structs }}{% raw %}

        pub fn build_translators() -> HashMap<String, Root> {
            let mut translators = HashMap::new();

            {% endraw %}
            {% for key, value in codes %}
                translators.insert(
                    "{{key}}".to_string(),
                    {{value}},
                );
            {% endfor %}
            {% raw %}

            translators
        }
    }

    static TRANSLATORS: Lazy<HashMap<String, trans::Root>> = Lazy::new(trans::build_translators);

    pub struct I18n {
        pub locale: String,
        pub t: &'static trans::Root,
    }

    impl I18n {
        pub fn new(locale: impl Into<String>) -> Self {
            let locale = locale.into();
            let t = TRANSLATORS.get(&locale).unwrap();

            Self { locale, t }
        }
    }

    {% endraw %}
    "#,
    )
    .unwrap();

    tera
});
