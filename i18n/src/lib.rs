
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

    pub struct Root {
        pub general: General,
    }

    impl Root {}

    pub struct General {
        pub this: This,
        pub hello: Hello,
    }

    impl General {}

    pub struct This {
        pub is: Is,
    }

    impl This {}

    pub struct Is {
        pub a: &'static str,
    }

    impl Is {
        pub fn a(&self, _0: &str) -> String {
            self.a.replacen("{}", _0, 1)
        }
    }

    pub struct Hello {
        pub world: &'static str,
    }

    impl Hello {
        pub fn world(&self) -> String {
            self.world.to_string()
        }
    }

    pub fn build_translators() -> HashMap<String, Root> {
        let mut translators = HashMap::new();

        translators.insert(
            "en".to_string(),
            Root {
                general: General {
                    this: This {
                        is: Is { a: "this is a {}" },
                    },
                    hello: Hello {
                        world: "Hello World",
                    },
                },
            },
        );

        translators.insert(
            "ja".to_string(),
            Root {
                general: General {
                    this: This {
                        is: Is {
                            a: "これは{}です"
                        },
                    },
                    hello: Hello {
                        world: "こんにちは",
                    },
                },
            },
        );

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
