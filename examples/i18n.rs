use rust_i18n_gen::i18n;

#[i18n("examples/lang", "ja")]
pub struct I18n {}

fn main() {
    println!("{}", I18n::new("ja").t("general.hello.world").get());
}
