use strum::{EnumString, VariantNames};

#[derive(Clone, Eq, PartialEq, Hash, Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum AnsiMode {
    #[strum(serialize = "8bit")]
    Ansi256,
    Rgb,
}
