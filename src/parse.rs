use crate::result::{I18nError, Result};
use glob::glob;
use indexmap::IndexMap;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::{fs::File, io::Read, ops::Range};

pub type Resource = (String, IndexMap<usize, Range<usize>>);
pub type ResourceMap = IndexMap<String, Vec<Resource>>;

pub fn parse_dir(dir: &str, source_locale: &str) -> Result<IndexMap<String, ResourceMap>> {
    let mut locale_resources = IndexMap::new();

    // 指定ディレクトリのファイルを全てパースする
    for entry in glob(&(dir.to_string() + "/*"))? {
        if let Ok(path_buf) = entry {
            let locale = path_buf.file_stem().unwrap().to_str().unwrap().to_string();
            let path = path_buf.to_str().unwrap();

            match parse_file(path) {
                Ok(resources) => {
                    locale_resources.insert(locale, resources);
                }
                Err(err) => match err.downcast_ref::<I18nError>() {
                    Some(I18nError::InvalidSourceFileFormat(_)) => {
                        println!("Skip invalid source file format: {:?}", path);
                    }
                    _ => {
                        return Err(err);
                    }
                },
            }
        }
    }

    // ソースロケールを基準に、他のロケールに存在しないメッセージがあれば追加する
    let source_resources = locale_resources
        .remove(source_locale)
        .ok_or(I18nError::SourceLocaleNotExists)?;

    for (_, resources) in &mut locale_resources {
        for (key, source_resource) in &source_resources {
            if !resources.contains_key(key) {
                resources.insert(key.to_string(), source_resource.clone());
            }
        }
    }

    locale_resources.insert(source_locale.to_string(), source_resources);

    Ok(locale_resources)
}

fn parse_file(path: &str) -> Result<ResourceMap> {
    if path.ends_with(".yaml") || path.ends_with(".yml") {
        parse_yaml_file(path)
    } else if path.ends_with(".json") {
        parse_json_file(path)
    } else {
        Err(I18nError::InvalidSourceFileFormat(path.to_string()).into())
    }
}

fn parse_yaml_file(path: &str) -> Result<ResourceMap> {
    let value = serde_yaml::from_reader(File::open(&path)?)?;
    parse(value)
}

fn parse_json_file(path: &str) -> Result<ResourceMap> {
    let mut json_str = String::new();
    File::open(&path)?.read_to_string(&mut json_str)?;
    let value = json5::from_str(&json_str)?;
    parse(value)
}

fn parse(value: Value) -> Result<ResourceMap> {
    let mut resource: ResourceMap = IndexMap::new();

    if let Value::Mapping(map) = value {
        parse_map(&map, &mut resource, "");
    }

    Ok(resource)
}

fn parse_map(map: &Mapping, resource: &mut ResourceMap, prefix: &str) {
    for (key, value) in map.iter() {
        let key = key.as_str().unwrap();
        let path: String;
        if prefix == "" {
            path = format!("{key}");
        } else {
            path = format!("{prefix}.{key}");
        }
        match value {
            Value::Mapping(map) => {
                parse_map(map, resource, &path);
            }
            Value::String(msg) => {
                resource.insert(
                    path,
                    msg.split(" | ")
                        .map(|msg| (msg.to_string(), parse_message(msg)))
                        .collect(),
                );
            }
            _ => panic!("Invalid value: {:?}", value),
        }
    }
}

fn parse_message(msg: &str) -> IndexMap<usize, Range<usize>> {
    let mut map = IndexMap::new();
    let re = Regex::new(r"\{\d+\}").unwrap();
    for mat in re.find_iter(&msg) {
        let range = mat.range();
        map.insert(msg[range.start + 1..range.end - 1].parse().unwrap(), range);
    }

    // 後ろから処理しないと置換位置がずれてしまうので順序を反転させる
    map.reverse();
    map
}
