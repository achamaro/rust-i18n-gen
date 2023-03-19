use std::{cmp::min, ops::Range};

use indexmap::{indexmap, IndexMap};
use once_cell::sync::OnceCell;
use rust_i18n_gen::i18n;

struct I18nMessage {
    message: &'static str,
    ranges: IndexMap<usize, Range<usize>>,
}

type I18nResources = IndexMap<&'static str, I18nResource>;

impl I18nMessage {
    pub fn new(message: &'static str, ranges: IndexMap<usize, Range<usize>>) -> Self {
        Self { message, ranges }
    }

    pub fn get(&self) -> &'static str {
        self.message
    }

    pub fn replace(&self, reps: &Vec<&str>) -> String {
        let mut message = self.message.to_string();

        for (i, range) in self.ranges.iter() {
            message.replace_range(range.clone(), reps[*i]);
        }

        message
    }
}

struct I18nResource {
    messages: Vec<I18nMessage>,
    messages_len: usize,
}

impl I18nResource {
    pub fn new(messages: Vec<I18nMessage>) -> Self {
        let messages_len = messages.len();
        Self {
            messages,
            messages_len,
        }
    }

    pub fn get(&self) -> &'static str {
        self.messages[0].get()
    }

    pub fn replace(&self, reps: &Vec<&str>) -> String {
        self.messages[0].replace(reps)
    }

    pub fn plural(&self, num: usize) -> &I18nMessage {
        let i;
        if self.messages_len == 1 {
            i = 0;
        } else if self.messages_len == 2 {
            if num == 1 {
                i = 0;
            } else {
                i = 1;
            }
        } else {
            i = min(self.messages_len - 1, num)
        }

        &self.messages[i]
    }
}

struct I18n {
    resources: &'static I18nResources,
}

impl I18n {
    pub fn new(locale: &str) -> Self {
        Self {
            resources: Self::resources().get(locale).unwrap(),
        }
    }

    pub fn resources() -> &'static IndexMap<&'static str, I18nResources> {
        static R: OnceCell<IndexMap<&'static str, I18nResources>> = OnceCell::new();
        R.get_or_init(|| {
            let mut map: IndexMap<&'static str, I18nResources> = IndexMap::new();

            let mut map1: I18nResources = IndexMap::new();
            map1.insert(
                "a",
                I18nResource::new(vec![
                    I18nMessage::new("a", indexmap! {}),
                    I18nMessage::new("b", indexmap! {}),
                ]),
            );
            map1.insert(
                "b",
                I18nResource::new(vec![I18nMessage::new("aa{0}bb", indexmap! {0 => 2..5})]),
            );
            map.insert("en", map1);
            map
        })
    }

    pub fn t(&self, key: &str) -> &'static I18nResource {
        match self.resources.get(key) {
            Some(v) => v,
            None => {
                panic!("Key `{}` not found", key)
            }
        }
    }
}

#[derive(Debug)]
#[i18n("lang", "ja")]
pub struct I18nGen {}

#[test]
fn test_i18n() {
    let i18n = I18n::new("en");
    assert_eq!("a", i18n.t("a").get());
    assert_eq!("a", i18n.t("a").plural(1).get());
    assert_eq!("b", i18n.t("a").plural(0).get());
    assert_eq!("b", i18n.t("a").plural(5).get());
    assert_eq!("aacbb", i18n.t("b").replace(&vec!["c"]));
    assert_eq!("aacbb", i18n.t("b").plural(5).replace(&vec!["c"]));

    let i18n_gen = I18nGen::new("en");
    assert_eq!("Hello World", i18n_gen.t("general.hello.world").get());
}
