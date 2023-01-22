fn main() {}

#[cfg(test)]
mod tests {
    use i18n::I18n;

    #[test]
    fn test() {
        let i18n = I18n::new("ja");
        assert_eq!(i18n.t.general.hello.world(), "こんにちは");
        assert_eq!(i18n.t.general.this.is.a("ペン"), "これはペンです");

        let i18n_en = I18n::new("en");
        assert_eq!(i18n_en.t.general.hello.world(), "Hello World");
    }
}
