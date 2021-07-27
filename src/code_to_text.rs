use std::{collections::BTreeMap, string::FromUtf16Error};
use thiserror::Error;
use crate::CodeTableEntryFile;

/// An error that may occur while decoding a binary with [`CodeToText::decode`]
#[derive(Debug, Error)]
pub enum CodeToTextError {
    #[error("While replacing the characters didn't caused any apparent issue, the resulting UTF-16 string can't be decoded (result : {1:?})")]
    CantDecodeResult(#[source] FromUtf16Error, Vec<u16>)
}

pub struct CodeToText<'a> {
    pub(crate) code_to_text: BTreeMap<u16, &'a CodeTableEntryFile>
}

impl<'a> CodeToText<'a> {
    pub fn decode(&'a self, text: &[u16]) -> Result<String, CodeToTextError> {
        let mut iterator = text.iter().map(|x| *x);

        let mut result: Vec<u16> = Vec::new(); //TODO: result should be a String to start with

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
                            let this_encoded_char = iterator.next().unwrap(); //TODO:
                            data.encoded_value |= (this_encoded_char as u32) << j*16;
                        }
                    };

                    data.encoded_value.to_string()
                } else {
                    String::new()
                };
                
                println!("{:?}, {:?}", data.entry.string, encoded_value_string);
                result.push('[' as u16);
                result.extend(data.entry.string.encode_utf16());
                result.extend(encoded_value_string.encode_utf16());
                result.push(']' as u16);
            } else {
                if point == '[' as u16 {
                    result.extend(&['\\' as u16, '[' as u16]);
                } else if point == '\\' as u16 {
                    result.extend(&['\\' as u16, '\\' as u16]);
                } else {
                    result.push(point);
                }
            }
        };

        String::from_utf16(&result).map_err(|err| CodeToTextError::CantDecodeResult(err, result))
    }
}
