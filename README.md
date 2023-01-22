# rust-i18n-gen

[link-i18n-ally]: https://marketplace.visualstudio.com/items?itemName=lokalise.i18n-ally
[link-i18n-ally-config]: https://github.com/lokalise/i18n-ally/wiki/Configurations
[link-cargo-watch]: https://github.com/watchexec/cargo-watch

[i18n-ally][link-i18n-ally] 頼みの翻訳モジュール生成ライブラリ（自分用）

## Getting Started

### [i18n-ally][link-i18n-ally] の [ドキュメント][link-i18n-ally-config] に沿って VSCode の設定

```
example/
  ├─ .vscode/
  │   ├─ i18n-ally-custom-framework.yml
  │   └─ settings.json
  ├─ lang/
  │   ├─ en.yaml
  │   └─ ja.yaml
  └─ src/
      └─ main.rs
```

`.vscode/settings.json`

```json:.vscode/settings.json
{
  "i18n-ally.localesPaths": ["lang"],
  "i18n-ally.namespace": true,
  "i18n-ally.sourceLanguage": "ja",
  "i18n-ally.pathMatcher": "{locale}.{ext}",
  "i18n-ally.keystyle": "nested",
  "i18n-ally.sortKeys": true
}
```

`.vscode/i18n-ally-custom-framework.yml`

```yaml:.vscode/i18n-ally-custom-framework.yml
languageIds:
  - rust
usageMatchRegex:
  - "[^\\w\\d]t.({key})\\("
monopoly: true
```

### 翻訳モジュールの出力先の作成

```
cargo init {module_name}
cd {module_name}
```

`{module_name}/Cargo.toml` の `[dependencies]` にパッケージを追記  
（バージョンは適当なものを設定する）

```toml:{module_name}/Cargo.toml
once_cell = "1.17.0"
rust-i18n-gen = { git = "https://github.com/achamaro/rust-i18n-gen", tag = "v0.1.1" }
```

モジュール生成用の `{module_name}/src/main.rs` を作成

```rust:{module_name}/src/main.rs
use std::fs::write;

use rust_i18n_gen::{generate, load_resources};

fn main() {
    let code = generate(
        "ja",
        &mut load_resources(&vec!["../example/lang".to_string()]),
    );

    write("src/lib.rs", code).unwrap();
}
```

### 生成

[cargo watch][link-cargo-watch] をつかって言語ファイルを監視しつつ生成＆フォーマット

```
cargo watch -w ../example/lang -x run -x fmt
```
