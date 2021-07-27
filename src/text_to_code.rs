use std::{collections::HashMap, num::ParseIntError};

use thiserror::Error;

use crate::CodeTableEntryFile;

#[derive(Debug, Error)]
pub enum TextToCodeError {
    #[error("The character '{0:?}' has been escaped, but it doesn't need to. If you want to display \\, you need to write \\\\.")]
    UselessEscape(char),
    #[error("The final character of the string is an unescaped \\. If you want to display \\, add another \\ at the end of that string.")]
    UnfinishedEscape,
    #[error("The string end with an unfinished placeholder. [ start an escape sequence if they aren't preceded with \\ (this \\ isn't displayed), and a ] close it.")]
    UnclosedPlaceholder,
    #[error("The string contain an empty placeholder []. If you want to display [], write \\[] instead (to escape the [)")]
    EmptyPlaceholder,
    #[error("The string contain a placeholder containing too much part (a part in a placeholder is separated with :). The various part of the placeholder are : {0:?}")]
    PlaceholderTooMuchPart(Vec<String>),
    #[error("The placeholder {0:?} is unrecognized")]
    UnknownPlaceholder(String),
    #[error("The value \"{1:?}\" for the placeholder \"{2:?}\" is neither an hardcoded one, nor a base 10 string (or it may be a base 10 number, but superior to 2^32 - 1)")]
    InvalidValue(#[source] ParseIntError, String, String),
    #[error("The placeholder {1:?} has the associated value {0:?}, thought it should less or equal to 255 (hard value for this placeholder type")]
    CantEncodedParameterEmbeddedData(u32, String)
}

pub struct TextToCode<'a> {
    pub(crate) text_to_code: HashMap<&'a String, &'a CodeTableEntryFile>,
}

impl<'a> TextToCode<'a> {
    pub fn encode(&self, text: &str) -> Result<Vec<u16>, TextToCodeError> {
        let mut buffer = [0; 2];
        let mut result: Vec<u16> = Vec::new();

        let mut iterator = text.chars();
        while let Some(chara) = iterator.next() {
            if chara == '[' {
                let mut placeholder = Vec::new();
                let mut placeholder_current_string = String::with_capacity(10);
                loop {
                    if let Some(placeholder_char) = iterator.next() {
                        if placeholder_char == ']' {
                            placeholder.push(placeholder_current_string.clone());
                            break;
                        } else if placeholder_char == ':' {
                            placeholder_current_string.push(':');
                            placeholder.push(placeholder_current_string.clone());
                            placeholder_current_string = String::with_capacity(10);
                        } else {
                            placeholder_current_string.push(placeholder_char)
                        }
                    } else {
                        return Err(TextToCodeError::UnclosedPlaceholder);
                    }
                }
                let (directive, value) = match placeholder.len() {
                    0 => return Err(TextToCodeError::EmptyPlaceholder),
                    1 => (placeholder[0].clone(), None),
                    2 => (placeholder[0].clone(), Some(placeholder[1].clone())),
                    _ => return Err(TextToCodeError::PlaceholderTooMuchPart(placeholder)),
                };

                let complete_placeholder = if let Some(ref value) = value {
                    format!("{}{}", directive, value)
                } else {
                    directive.clone()
                };

                if let Some(placeholder_char) = self.text_to_code.get(&complete_placeholder) {
                    result.push(placeholder_char.value);
                } else if let Some(value) = value.clone() {
                    let entry = *self.text_to_code.get(&directive).map_or_else(|| Err(TextToCodeError::UnknownPlaceholder(directive.clone())), Ok)?;
                    let value_number = u32::from_str_radix(&value, 10).map_err(|err| TextToCodeError::InvalidValue(err, value.clone(), directive.clone()))?;
                    if entry.lenght == 0 {
                        if value_number > 255 {
                            return Err(TextToCodeError::CantEncodedParameterEmbeddedData(value_number, entry.string.clone()))
                        };
                        result.push(entry.value + value_number as u16);
                    } else {
                        result.push(entry.value + (value_number % 256).saturating_sub(1) as u16);
                        let mut remaining = value_number;
                        loop {
                            if remaining == 0 {
                                break
                            }
                            let this_part = remaining & 0x0000FFFF;
                            remaining = remaining >> 16;
                            result.push(this_part as u16);
                        }
                    }
                } else {
                    return Err(TextToCodeError::UnknownPlaceholder(complete_placeholder))
                }

            } else if chara == '\\' {
                if let Some(next_chara) = iterator.next() {
                    if next_chara == '[' || next_chara == '\\' {
                        let slice = next_chara.encode_utf16(&mut buffer);
                        result.extend(slice.iter());
                    } else {
                        return Err(TextToCodeError::UselessEscape(next_chara));
                    }
                } else {
                    return Err(TextToCodeError::UnfinishedEscape);
                }
            } else {
                let slice = chara.encode_utf16(&mut buffer);
                result.extend(slice.iter());
            }
        }

        Ok(result)
    }
}
