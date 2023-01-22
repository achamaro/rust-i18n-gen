mod template;

use convert_case::{Case, Casing};
use glob::glob;
use regex::{escape, Regex};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};

use template::generate_i18n;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Value {
    Singular(Singular),
    Plural(Plural),
    Map(Map),
}

impl Value {
    pub fn as_map(&self) -> &Map {
        match self {
            Value::Map(map) => map,
            _ => panic!("Not a map"),
        }
    }

    pub fn as_map_mut(&mut self) -> &mut Map {
        match self {
            Value::Map(map) => map,
            _ => panic!("Not a map"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Singular(pub String);

impl Singular {
    pub fn generate_struct(&self, name: &str) -> String {
        let (args, reps) = parse_args(&self.0);
        return format!(
            "
            pub fn {name}(&self{args}) -> String {{
                self.{name}{reps}
            }}
            "
        );
    }

    pub fn generate_code(&self) -> String {
        format!("\"{}\"", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Plural(pub Vec<String>);

impl Plural {
    pub fn generate_struct(&self, name: &str) -> String {
        let max_args_index = max_args_index(&self.0);
        let (args, reps) = parse_args(&self.0[max_args_index]);
        return format!(
            "
            pub fn {name}(&self, num: &usize{args}) -> String {{
                let i = message_index(&self.{name}.len(), num);
                self.{name}[i]{reps}
            }}
            "
        );
    }

    pub fn generate_code(&self) -> String {
        format!("vec!{:?}", self.0)
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Map(pub HashMap<String, Value>);

impl Map {
    pub fn insert(&mut self, namespaces: Vec<&str>, value: Value) {
        let mut current = self;

        let mut peekable = namespaces.iter().peekable();
        while let Some(&ns) = peekable.next() {
            if peekable.peek().is_none() {
                current.0.insert(ns.to_string(), value);
                break;
            }

            if !current.0.contains_key(ns) {
                current
                    .0
                    .insert(ns.to_string(), Value::Map(Map(HashMap::new())));
            }

            current = current.0.get_mut(ns).unwrap().as_map_mut();
        }
    }
}

impl Map {
    pub fn generate_struct(&self, name: &str) -> String {
        let p_name = name.to_case(Case::Pascal);
        let mut struct_str = "".to_string();
        let mut impl_str = "".to_string();
        let mut map_str = "".to_string();
        for (key, value) in &self.0 {
            match value {
                Value::Map(gen) => {
                    let p_key = key.to_case(Case::Pascal);
                    struct_str.push_str(&format!("pub {key}: {p_key},\n"));
                    map_str.push_str(&gen.generate_struct(key));
                }
                Value::Singular(gen) => {
                    struct_str.push_str(&format!("pub {key}: &'static str,\n"));
                    impl_str.push_str(&gen.generate_struct(key));
                }
                Value::Plural(gen) => {
                    struct_str.push_str(&format!("pub {key}: Vec<&'static str>,\n"));
                    impl_str.push_str(&gen.generate_struct(key));
                }
            }
        }

        format!(
            "
            pub struct {p_name} {{
                {struct_str}
            }}

            impl {p_name} {{
                {impl_str}
            }}

            {map_str}
            "
        )
    }

    pub fn generate_code(&self, name: &str, source: Option<&Self>) -> String {
        let p_name = name.to_case(Case::Pascal);
        let mut code_str = "".to_string();
        code_str.push_str(&format!("{p_name} {{\n"));

        // ソース言語を基準にする
        let map = match source {
            Some(map) => &map.0,
            None => &self.0,
        };

        for (key, source_value) in map {
            // 対象言語に存在する場合はそちらを使用する
            let value = match self.0.get(key) {
                Some(value) => value,
                None => source_value,
            };

            code_str.push_str(&format!(
                "{key}: {},\n",
                match value {
                    Value::Map(gen) => gen.generate_code(key, Some(&source_value.as_map())),
                    Value::Singular(gen) => gen.generate_code(),
                    Value::Plural(gen) => gen.generate_code(),
                }
            ));
        }
        code_str.push_str("}");

        code_str
    }
}

pub fn load_resources(dirs: &Vec<String>) -> HashMap<String, Value> {
    let mut root_map = Map(HashMap::new());

    for dir in dirs {
        let dir = dir.trim_end_matches("/").to_string() + "/";
        let ext = "yaml".to_string();
        let trim_regex = Regex::new(&format!(r"^{}|\.{}$", escape(&dir), escape(&ext))).unwrap();

        for entry in glob(&(dir.to_string() + "**/*" + &ext)).unwrap() {
            if let Ok(path) = entry {
                let path_str = path.to_str().unwrap();
                let namespace_str = trim_regex.replace_all(path_str, "").replace("/", ".");

                let mut namespaces: Vec<&str> = namespace_str.split(".").collect();
                let last_index = namespaces.len() - 1;

                // 言語コードを先頭に移動
                namespaces.swap(0, last_index);

                let value: Value = serde_yaml::from_reader(File::open(&path).unwrap()).unwrap();

                // 目的の階層のメッセージを取得
                root_map.insert(namespaces, value);
            }
        }
    }

    root_map.0
}

pub fn generate(source_language: &str, root_map: &mut HashMap<String, Value>) -> String {
    let source = root_map.remove(source_language).unwrap();
    let source_map = source.as_map();

    let mut codes = HashMap::new();

    codes.insert(
        source_language.to_string(),
        source_map.generate_code("Root", None),
    );

    for (locale, value) in root_map {
        codes.insert(
            locale.to_string(),
            value.as_map().generate_code("Root", Some(&source_map)),
        );
    }

    generate_i18n(source_map.generate_struct("Root"), codes)
}

fn parse_args(message: &String) -> (String, String) {
    let args_reg = Regex::new(r"\{(.*?)\}").unwrap();

    let args = args_reg
        .captures_iter(message)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<String>>();

    if args.is_empty() {
        return (String::from(""), String::from(".to_string()"));
    }

    (
        String::from(", ")
            + &args
                .iter()
                .enumerate()
                .map(|(i, arg)| format!("{arg}_{i}: &str"))
                .collect::<Vec<String>>()
                .join(", "),
        args.iter()
            .enumerate()
            .map(|(i, arg)| format!(".replacen(\"{{{arg}}}\", {arg}_{i}, 1)"))
            .collect::<Vec<String>>()
            .join(""),
    )
}

fn max_args_index(messages: &Vec<String>) -> usize {
    let args_reg = Regex::new(r"\{(.*?)\}").unwrap();

    let mut match_counts = messages
        .iter()
        .enumerate()
        .map(|(i, message)| (i, args_reg.find_iter(message).count()))
        .collect::<Vec<(usize, usize)>>();

    match_counts.sort_by_key(|(_, count)| *count);

    match_counts.last().unwrap().0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_resources() {
        let source: Map = serde_yaml::from_str(
            r#"
            general:
                key1: キー1
                key23:
                    - キー2
                    - キー3
                key4: キー4 {}
            "#,
        )
        .unwrap();
        let value: Map = serde_yaml::from_str(
            r#"
            general:
                key23:
                    - key2
                    - key3
                key4: key4 {}
            "#,
        )
        .unwrap();

        let mut root_message = HashMap::new();
        root_message.insert("ja".to_string(), Value::Map(source));
        root_message.insert("en".to_string(), Value::Map(value));

        let generated = generate("ja", &mut root_message);
        println!("{}", generated);
    }
}
