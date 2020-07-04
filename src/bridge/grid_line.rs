#[derive(Debug, PartialEq)]
pub struct GridLine {
    grid: i64,
    row: i64,
    col_start: i64,
    content: Vec<super::character::Character>,
}

impl std::convert::TryFrom<&rmpv::Value> for GridLine {
    type Error = super::errors::BridgeErrors;
    fn try_from(value: &rmpv::Value) -> Result<Self, Self::Error> {
        //All this does is check if first second and third value in array is
        //numeric, if they are, store them in the respective variables.
        let mut arr = value.as_array().unwrap().iter();
        let grid = arr
            .next()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .as_i64()
            .ok_or(super::errors::BridgeErrors::ConversionError)?;
        let row = arr
            .next()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .as_i64()
            .ok_or(super::errors::BridgeErrors::ConversionError)?;
        let col_start = arr
            .next()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .as_i64()
            .ok_or(super::errors::BridgeErrors::ConversionError)?;

        let mut content = Vec::new();
        let mut characters = arr
            .next()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .as_array()
            .ok_or(super::errors::BridgeErrors::ConversionError)?
            .iter();

        //handling the edge case of if there is no characters in a grid line update.
        if characters.len() == 0 {
            return Ok(Self {
                grid,
                row,
                col_start,
                content: vec![],
            });
        }

        //The first character *must* have a highlight ID.
        content.push(
            super::character::Character::try_from(
                characters
                    .next()
                    .ok_or(super::errors::BridgeErrors::ConversionError)?,
            )
            .map_err(|_| super::errors::BridgeErrors::ConversionError)?,
        );

        for (i, c) in characters.enumerate() {
            content.push(
                super::character::Character::from_val_highlight(c, content[i].highlight_id)
                    .map_err(|_| super::errors::BridgeErrors::ConversionError)?,
            )
        }
        Ok(Self {
            grid,
            row,
            col_start,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    #[test]
    fn grid_line_empty() {
        assert_eq!(
            GridLine::try_from(&rmpv::Value::Array(vec![
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Array(vec![])
            ]))
            .unwrap(),
            GridLine {
                grid: 0,
                col_start: 0,
                row: 0,
                content: vec![]
            }
        );
    }
    #[test]
    fn grid_line_col_row_val_check() {
        assert_eq!(
            GridLine::try_from(&rmpv::Value::Array(vec![
                rmpv::Value::Integer(rmpv::Integer::from(1i32)),
                rmpv::Value::Integer(rmpv::Integer::from(2i32)),
                rmpv::Value::Integer(rmpv::Integer::from(3i32)),
                rmpv::Value::Array(vec![])
            ]))
            .unwrap(),
            GridLine {
                grid: 1,
                row: 2,
                col_start: 3,
                content: vec![]
            }
        );
    }
    #[test]
    fn grid_line_character_check() {
        assert_eq!(
            GridLine::try_from(&rmpv::Value::Array(vec![
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Integer(rmpv::Integer::from(0i32)),
                rmpv::Value::Array(vec![
                    rmpv::Value::Array(vec![
                        rmpv::Value::String(rmpv::Utf8String::from("T")),
                        rmpv::Value::Integer(rmpv::Integer::from(1i32)),
                        rmpv::Value::Integer(rmpv::Integer::from(3i32)),
                    ]),
                    rmpv::Value::Array(vec![
                        rmpv::Value::String(rmpv::Utf8String::from("E")),
                        rmpv::Value::Integer(rmpv::Integer::from(2i32)),
                        rmpv::Value::Integer(rmpv::Integer::from(2i32)),
                    ]),
                    rmpv::Value::Array(vec![rmpv::Value::String(rmpv::Utf8String::from("S")),])
                ])
            ]))
            .unwrap(),
            GridLine {
                grid: 0,
                row: 0,
                col_start: 0,
                content: vec![
                    super::super::character::Character::new("T".into(), 1, 3),
                    super::super::character::Character::new("E".into(), 2, 2),
                    super::super::character::Character::new("S".into(), 2, 1),
                ]
            }
        );
    }
}
