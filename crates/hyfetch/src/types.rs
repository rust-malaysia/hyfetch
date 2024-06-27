use strum::{EnumString, VariantNames};

#[derive(Clone, Eq, PartialEq, Hash, Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum AnsiMode {
    #[strum(serialize = "8bit")]
    Ansi256,
    Rgb,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Backend {
    Qwqfetch,
    Neofetch,
    Fastfetch,
    FastfetchOld,
}
