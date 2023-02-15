use std::ffi::OsStr;

use uutils_args::{Arguments, Initial, Options, Value, ValueResult};

#[test]
fn string_option() {
    #[derive(Arguments)]
    enum Arg {
        #[option("--message=MSG")]
        Message(String),
    }

    #[derive(Initial)]
    struct Settings {
        message: String,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Message(s): Arg) {
            self.message = s
        }
    }

    assert_eq!(
        Settings::parse(["test", "--message=hello"]).message,
        "hello"
    );
}

#[test]
fn enum_option() {
    #[derive(Value, Default, Debug, PartialEq, Eq, Clone)]
    enum Format {
        #[default]
        #[value]
        Foo,
        #[value]
        Bar,
        #[value]
        Baz,
    }

    #[derive(Arguments)]
    enum Arg {
        #[option("--format=FORMAT")]
        Format(Format),
    }

    #[derive(Initial)]
    struct Settings {
        format: Format,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Format(f): Arg) {
            self.format = f;
        }
    }

    assert_eq!(
        Settings::parse(["test", "--format=bar"]).format,
        Format::Bar
    );

    assert_eq!(
        Settings::parse(["test", "--format", "baz"]).format,
        Format::Baz
    );
}

#[test]
fn enum_option_with_fields() {
    #[derive(Value, Default, Debug, PartialEq, Eq)]
    enum Indent {
        #[default]
        Tabs,
        #[value("thin", value = Self::Spaces(4))]
        #[value("wide", value = Self::Spaces(8))]
        Spaces(u8),
    }

    #[derive(Arguments)]
    enum Arg {
        #[option("-i INDENT")]
        Indent(Indent),
    }

    #[derive(Initial)]
    struct Settings {
        indent: Indent,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Indent(i): Arg) {
            self.indent = i;
        }
    }

    assert_eq!(
        Settings::parse(["test", "-i=thin"]).indent,
        Indent::Spaces(4)
    );
    assert_eq!(
        Settings::parse(["test", "-i=wide"]).indent,
        Indent::Spaces(8)
    );
}

#[test]
fn enum_with_complex_from_value() {
    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    enum Indent {
        #[default]
        Tabs,
        Spaces(u8),
    }

    impl Value for Indent {
        fn from_value(value: &std::ffi::OsStr) -> ValueResult<Self> {
            let value = String::from_value(value)?;
            if value == "tabs" {
                Ok(Self::Tabs)
            } else if let Ok(n) = value.parse() {
                Ok(Self::Spaces(n))
            } else {
                Err("Failure!".into())
            }
        }
    }

    #[derive(Arguments)]
    enum Arg {
        #[option("-i INDENT")]
        Indent(Indent),
    }

    #[derive(Initial)]
    struct Settings {
        indent: Indent,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Indent(i): Arg) {
            self.indent = i;
        }
    }

    assert_eq!(Settings::parse(["test", "-i=tabs"]).indent, Indent::Tabs);
    assert_eq!(Settings::parse(["test", "-i=4"]).indent, Indent::Spaces(4));
}

#[test]
fn color() {
    #[derive(Default, Value, Debug, PartialEq, Eq)]
    enum Color {
        #[value("yes", "always")]
        Always,
        #[default]
        #[value("auto")]
        Auto,
        #[value("no", "never")]
        Never,
    }

    #[derive(Arguments)]
    enum Arg {
        #[option("--color[=WHEN]")]
        Color(Option<Color>),
    }

    #[derive(Initial)]
    struct Settings {
        #[field(default = Color::Auto)]
        color: Color,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Color(c): Arg) {
            self.color = c.unwrap_or(Color::Always);
        }
    }

    assert_eq!(
        Settings::parse(["test", "--color=yes"]).color,
        Color::Always
    );
    assert_eq!(
        Settings::parse(["test", "--color=always"]).color,
        Color::Always
    );
    assert_eq!(Settings::parse(["test", "--color=no"]).color, Color::Never);
    assert_eq!(
        Settings::parse(["test", "--color=never"]).color,
        Color::Never
    );
    assert_eq!(Settings::parse(["test", "--color=auto"]).color, Color::Auto);
    assert_eq!(Settings::parse(["test", "--color"]).color, Color::Always)
}

#[test]
fn actions() {
    #[derive(Arguments)]
    enum Arg {
        #[option("-m MESSAGE")]
        Message(String),
        #[option("--send")]
        Send,
        #[option("--receive")]
        Receive,
    }

    #[derive(Initial)]
    struct Settings {
        last_message: String,
        send: bool,
        messages: Vec<String>,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, arg: Arg) {
            match arg {
                Arg::Message(m) => {
                    self.last_message = m.clone();
                    self.messages.push(m);
                }
                Arg::Send => self.send = true,
                Arg::Receive => self.send = false,
            }
        }
    }

    let settings = Settings::parse(["test", "-m=Hello", "-m=World", "--send"]);
    assert_eq!(settings.messages, vec!["Hello", "World"]);
    assert_eq!(settings.last_message, "World");
    assert!(settings.send);
}

#[test]
fn width() {
    #[derive(Arguments)]
    enum Arg {
        #[option("-w WIDTH")]
        Width(u64),
    }

    #[derive(Initial)]
    struct Settings {
        width: Option<u64>,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Width(w): Arg) {
            self.width = match w {
                0 => None,
                x => Some(x),
            }
        }
    }

    assert_eq!(Settings::parse(["test", "-w=0"]).width, None);
    assert_eq!(Settings::parse(["test", "-w=1"]).width, Some(1));
}

#[test]
fn integers() {
    #[derive(Arguments)]
    enum Arg {
        #[option("--u8=N")]
        U8(u8),
        #[option("--u16=N")]
        U16(u16),
        #[option("--u32=N")]
        U32(u32),
        #[option("--u64=N")]
        U64(u64),
        #[option("--u128=N")]
        U128(u128),
        #[option("--i8=N")]
        I8(i8),
        #[option("--i16=N")]
        I16(i16),
        #[option("--i32=N")]
        I32(i32),
        #[option("--i64=N")]
        I64(i64),
        #[option("--i128=N")]
        I128(i128),
    }

    #[derive(Initial)]
    struct Settings {
        n: i128,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, arg: Arg) {
            self.n = match arg {
                Arg::U8(x) => x as i128,
                Arg::U16(x) => x as i128,
                Arg::U32(x) => x as i128,
                Arg::U64(x) => x as i128,
                Arg::U128(x) => x as i128,
                Arg::I8(x) => x as i128,
                Arg::I16(x) => x as i128,
                Arg::I32(x) => x as i128,
                Arg::I64(x) => x as i128,
                Arg::I128(x) => x,
            }
        }
    }

    assert_eq!(Settings::parse(["test", "--u8=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u16=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u32=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u64=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u128=5"]).n, 5);

    assert_eq!(Settings::parse(["test", "--i8=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i16=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i32=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i64=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i128=5"]).n, 5);
}

#[test]
fn ls_classify() {
    #[derive(Value, Default, PartialEq, Eq, Debug)]
    enum When {
        #[value]
        Never,
        #[default]
        #[value]
        Auto,
        #[value]
        Always,
    }

    #[derive(Arguments)]
    enum Arg {
        #[option(
            "-F", "--classify[=WHEN]",
            default = When::Always,
        )]
        Classify(When),
    }

    #[derive(Initial)]
    struct Settings {
        classify: When,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::Classify(c): Arg) {
            self.classify = c;
        }
    }

    assert_eq!(Settings::parse(["test"]).classify, When::Auto);
    assert_eq!(
        Settings::parse(["test", "--classify=never"]).classify,
        When::Never,
    );
    assert_eq!(
        Settings::parse(["test", "--classify"]).classify,
        When::Always,
    );
    assert_eq!(Settings::parse(["test", "-F"]).classify, When::Always,);
    assert!(Settings::try_parse(["test", "-Falways"]).is_err());
}

#[test]
fn mktemp_tmpdir() {
    #[derive(Clone, Arguments)]
    enum Arg {
        #[option(
            "-p DIR", "--tmpdir[=DIR]",
            default = String::from("/tmp"),
        )]
        TmpDir(String),
    }

    #[derive(Initial)]
    struct Settings {
        tmpdir: Option<String>,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, Arg::TmpDir(dir): Arg) {
            self.tmpdir = Some(dir);
        }
    }

    let settings = Settings::parse(["test", "-p", "X"]);
    assert_eq!(settings.tmpdir.unwrap(), "X");

    let settings = Settings::parse(["test", "--tmpdir=X"]);
    assert_eq!(settings.tmpdir.unwrap(), "X");

    let settings = Settings::parse(["test", "--tmpdir"]);
    assert_eq!(settings.tmpdir.unwrap(), "/tmp");

    assert!(Settings::try_parse(["test", "-p"]).is_err());
}

#[test]
fn infer_value() {
    #[derive(Value, PartialEq, Eq, Debug)]
    enum Foo {
        #[value("long")]
        Long,
        #[value("link")]
        Link,
        #[value("deck")]
        Deck,
        #[value("desk")]
        Desk,
    }

    assert_eq!(Foo::from_value(OsStr::new("lo")).unwrap(), Foo::Long);
    assert_eq!(Foo::from_value(OsStr::new("dec")).unwrap(), Foo::Deck);

    Foo::from_value(OsStr::new("l")).unwrap_err();
    Foo::from_value(OsStr::new("de")).unwrap_err();
}

#[test]
fn deprecated() {
    #[derive(Arguments)]
    enum Arg {
        #[option("-{N}")]
        Min(usize),

        #[option("+{N}")]
        Plus(isize),
    }

    #[derive(Initial)]
    struct Settings {
        n1: usize,
        n2: isize,
    }

    impl Options<Arg> for Settings {
        fn apply(&mut self, arg: Arg) {
            match arg {
                Arg::Min(n) => self.n1 = n,
                Arg::Plus(n) => self.n2 = n,
            }
        }
    }

    assert_eq!(Settings::parse(["test", "-10"]).n1, 10usize);
    assert!(Settings::try_parse(["test", "--10"]).is_err());
    assert_eq!(Settings::parse(["test", "+10"]).n2, 10isize);
    assert_eq!(Settings::parse(["test", "+-10"]).n2, -10isize);
}
