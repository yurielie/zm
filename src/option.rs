
pub struct OptionDefinition {
    pub long: &'static str,
    pub short: &'static str,
    args: &'static str,
    help: &'static str,
}
impl OptionDefinition {
    const fn new(long: &'static str, short: &'static str, args: &'static str, help: &'static str) -> Self {
        Self { long, short, args, help, }
    }

    pub const fn header_len(&self) -> usize {
        let mut len = 0;
        len += self.long.len();
        if !self.short.is_empty() {
            len += ", ".len();
            len += self.short.len();
        }
        if !self.args.is_empty() {
            len += " ".len();
            len += self.args.len();
        }
        len += "  ".len();
        len
    }

    pub fn to_string_with_spaces(&self, spaces: usize) -> String {
        let mut s = String::new();
        if !self.short.is_empty() {
            s.push_str(self.short);
            s.push_str(", ");
        }
        s.push_str(self.long);
        if !self.args.is_empty() {
            s.push(' ');
            s.push_str(self.args);
        }
        format!("{}{}{}", s, " ".repeat(spaces), self.help)
    }
}


pub const OPT_FILE: OptionDefinition = OptionDefinition::new("--file", "-f", "<FILE>", "path of JSON configuration file");
pub const OPT_SHOW_KW_WITH: OptionDefinition = OptionDefinition::new("--show_keyword_with", "", "<DELIMITOR>", "show keyword name with given delimitor like 'keyword=value'");
pub const OPT_HELP: OptionDefinition = OptionDefinition::new("--help", "-h", "", "print help");

pub const OPTIONS: [OptionDefinition; 3] = [
    OPT_FILE,
    OPT_SHOW_KW_WITH,
    OPT_HELP,
];
