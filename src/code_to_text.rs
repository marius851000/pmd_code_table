use std::{collections::BTreeMap, string::FromUtf16Error};
use thiserror::Error;
use crate::CodeTableEntryFile;

/// An error that may occur while decoding a binary with [`CodeToText::decode`]
#[derive(Debug, Error)]
pub enum CodeToTextError {
    #[error("The final character of this line is not a valid UTF-16 character. It end with the 16bit code point {1} (partially decoded string: {0:?}). The encoding of the input is likely invalid.")]
    FinalInvalid(String, u16),
    #[error("Can't decode the pair of the two number {2} and {3} as a valid UTF-16 character. The input file is likely corrupted (partially decoded string: {1:?})")]
    CantDecodeUtf16(#[source] FromUtf16Error, String, u16, u16),
    #[error("There are missing character at the end of the string to decode the value of a placeholder. The input file is likely corrupted")]
    FinalNotLongEnoughtForData
}

pub struct CodeToText<'a> {
    pub(crate) code_to_text: BTreeMap<u16, &'a CodeTableEntryFile>
}

impl<'a> CodeToText<'a> {
    pub fn decode(&'a self, text: &[u16]) -> Result<String, CodeToTextError> {
        let mut iterator = text.iter().map(|x| *x);

        let mut result = String::new();

        while let Some(point) = iterator.next() {
            struct EncodedData<'a> {
                entry: &'a CodeTableEntryFile,
                encoded_value: u32,
            }

            let data: Option<EncodedData> = if let Some(entry) = self.code_to_text.get(&point) {
                Some(EncodedData {
                    entry: *entry,
                    encoded_value: 0
                })
            } else if let Some(entry) = self.code_to_text.get(&(point & 0xFF00)) {
                Some(EncodedData {
                    entry: *entry,
                    encoded_value: (point & 0x00FF) as u32 //TODO: maybe i need a +1 here
                })
            } else {
                None
            };

            if let Some(mut data) = data {
                let encoded_value_string = if data.entry.flags != 0 {
                    if data.entry.lenght > 0 {
                        data.encoded_value = 0;
                        for j in 0..data.entry.lenght {
                            let this_encoded_char = iterator.next().map_or_else(|| Err(CodeToTextError::FinalNotLongEnoughtForData), Ok)?;
                            data.encoded_value |= (this_encoded_char as u32) << j*16;
                        }
                    };

                    data.encoded_value.to_string()
                } else {
                    String::new()
                };
                
                result.push('[');
                result.push_str(&data.entry.string);
                result.push_str(&encoded_value_string);
                result.push(']');
            } else {
                if point == '[' as u16 {
                    result.push_str("\\[");
                } else if point == '\\' as u16 {
                    result.push_str("\\\\");
                } else {
                    if let Some(point_as_char) = char::from_u32(point as u32) {
                        result.push(point_as_char);
                    } else if let Some(next_point) = iterator.next() {
                        let decoded_string = String::from_utf16(&[point, next_point]).map_err(|err| CodeToTextError::CantDecodeUtf16(err, result.clone(), point, next_point))?;
                        result.push_str(&decoded_string);
                    } else {
                        return Err(CodeToTextError::FinalInvalid(result, point))
                    }
                }
            }
        };

        Ok(result)
    }
}
