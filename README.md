# rust-i18n-gen

[link-i18n-ally]: https://marketplace.visualstudio.com/items?itemName=lokalise.i18n-ally
[link-i18n-ally-config]: https://github.com/lokalise/i18n-ally/wiki/Configurations
[link-cargo-watch]: https://github.com/watchexec/cargo-watch

[i18n-ally][link-i18n-ally] 頼みの翻訳モジュール生成ライブラリ（自分用）

## Getting Started

`Cargo.toml` の `[dependencies]` にパッケージを追記  
（バージョンは適当なものを設定する）

```toml
[dependencies]
rust-i18n-gen = { git = "https://github.com/achamaro/rust-i18n-gen", tag = "v0.2.0" }
once_cell = "1.17.0"
indexmap = "1.9.2"
```

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

```json
{
  "i18n-ally.localesPaths": ["lang"],
  "i18n-ally.namespace": true,
  "i18n-ally.sourceLanguage": "ja",
  "i18n-ally.keystyle": "nested",
  "i18n-ally.sortKeys": true
}
```

`.vscode/i18n-ally-custom-framework.yml`

```yaml
languageIds:
  - rust
usageMatchRegex:
  - "[^\\w\\d]t\\(['\"`]({key})['\"`]"
monopoly: true
```

### 翻訳モジュール作成

```rust
use rust_i18n_gen::i18n;

// i18n(翻訳ファイルディレクトリ, ソース言語)
#[i18n("tests/lang", "ja")]
pub struct I18n {}
```

### 使い方

```rust
// { key1: "hello" }
// { key2: "hello {0}" }
// { key3: "apple | apples" }
// { key4: "no apples | 1 apple | 2 apples | {0} apples" }

// ロケールごとにインスタンス生成
let i18n = I18n::new("ja");

// 翻訳
i18n.t("key1").get(); // hello
// 置換
i18n.t("key2").replace(vec!["everyone"]); // hello everyone
// 複数形 (単数 | 複数)
i18n.t("key3").plural(0).get(); // apples
i18n.t("key3").plural(1).get(); // apple
i18n.t("key3").plural(3).get(); // apples
// 複数形+置換 (0 | 単数 | 複数)
i18n.t("key4").plural(0).replace(vec!["5"]); // no apples
i18n.t("key4").plural(1).replace(vec!["5"]); // 1 apple
i18n.t("key4").plural(2).replace(vec!["5"]); // 2 apple
i18n.t("key4").plural(5).replace(vec!["5"]); // 5 apples

```
