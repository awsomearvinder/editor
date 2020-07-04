///This represents a singular Character given by the neovim API.
#[derive(Debug, PartialEq)]
pub struct Character {
    utf8_char: String, //string not char becuase it's actually a "grapheme" instead of
    //a unicode scalar.
    //TODO: when hl_attr_define is implemented,
    //switch this out fro a highlight directly (via some hashmap?)
    pub highlight_id: i64,
    repeat: i64,
}

impl std::convert::TryFrom<&rmpv::Value> for Character {
    type Error = super::errors::BridgeErrors;
    ///Convert a rmpv value to a Character.
    fn try_from(value: &rmpv::Value) -> Result<Self, Self::Error> {
        let arr = value
            .as_array()
            .ok_or(super::errors::BridgeErrors::ConversionError)?;
        let highlight_id = arr
            .get(1)
            .and_then(|i| i.as_i64())
            .ok_or(Self::Error::ConversionError)?;
        Self::from_val_highlight(value, highlight_id)
    }
}

impl Character {
    pub fn new(utf8_char: String, highlight_id: i64, repeat: i64) -> Self {
        Self {
            utf8_char,
            highlight_id,
            repeat,
        }
    }

    ///This takes in the value representing the charecter and the highlight id.
    ///It then either takes out the highlight ID if present in the value, or it
    ///uses the highlight_id you gave if not present.
    pub fn from_val_highlight(
        value: &rmpv::Value,
        highlight_id: i64,
    ) -> Result<Self, super::errors::BridgeErrors> {
        let arr = value
            .as_array()
            .ok_or(super::errors::BridgeErrors::ConversionError)?;
        let utf8_char = arr
            .get(0)
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .as_str()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .to_owned();
        let highlight_id = arr.get(1).and_then(|i| i.as_i64()).unwrap_or(highlight_id);
        let repeat = arr.get(2).and_then(|i| i.as_i64()).unwrap_or(1);
        Ok(Self {
            utf8_char,
            highlight_id,
            repeat,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn char_check_empty() {
        assert_eq!(
            Character::try_from(&rmpv::Value::Array(vec![
                rmpv::Value::String(rmpv::Utf8String::from("")),
                rmpv::Value::Integer(rmpv::Integer::from(0)),
                rmpv::Value::Integer(rmpv::Integer::from(0))
            ]))
            .unwrap(),
            Character::new("".into(), 0, 0)
        )
    }
    #[test]
    fn char_check_no_repeat() {
        assert_eq!(
            Character::try_from(&rmpv::Value::Array(vec![
                rmpv::Value::String(rmpv::Utf8String::from("T")),
                rmpv::Value::Integer(rmpv::Integer::from(0))
            ]))
            .unwrap(),
            Character::new("T".into(), 0, 1)
        )
    }
    #[test]
    fn char_check_no_information() {
        assert_eq!(
            Character::try_from(&rmpv::Value::Array(vec![rmpv::Value::String(
                rmpv::Utf8String::from("T")
            ),])),
            Result::Err(super::super::errors::BridgeErrors::ConversionError)
        )
    }
}
