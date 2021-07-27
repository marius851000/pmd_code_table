//! A library permitting to decode the code_table.bin file, used in Pokémon Super Mystery Dungeon and Pokémon Rescue Team DX (and maybe Gates to Infinity).
//!
//! This file contain a list of unicode character that are used to represent placeholder in the game.

use binread::{BinRead, BinReaderExt, FilePtr32, NullWideString};
use pmd_sir0::{Sir0, Sir0Error};
use std::{
    collections::{BTreeMap, HashMap},
    io::{self, Read, Seek, SeekFrom},
};
use thiserror::Error;

mod code_to_text;
pub use code_to_text::{CodeToText, CodeToTextError};

mod text_to_code;
pub use text_to_code::{TextToCode, TextToCodeError};

/// Represent a single entry of a Unicode character <-> placeholder text pair.
#[derive(BinRead, Debug)]
#[br(little)]
pub struct CodeTableEntryFile {
    #[br(parse_with = FilePtr32::parse, try_map = |x: NullWideString| x.into_string_lossless())]
    pub string: String,
    pub value: u16,
    pub flags: u16,
    pub lenght: u16,
    pub unk: u16,
}

/// Represent a complete [`code_table.bin`] file.
#[derive(Default)]
pub struct CodeTable {
    entries: Vec<CodeTableEntryFile>,
}

/// An error that may happen while decoding a [`code_table.bin`] file with [`CodeTable`].
#[derive(Error, Debug)]
pub enum CodeTableDecodeError {
    #[error("can't decode the Sir0 file")]
    CantDecodeSir0(#[from] Sir0Error),
    #[error("the sir0 container only have {0} pointer, but it should have at least 5 pointer")]
    NotEnoughtPointer(usize),
    #[error("The offset of the pointer n°{0} can't be obtained")]
    CantGetOffsetForPointer(usize),
    #[error("Can't read (maybe a part) of the Sir0 file. This may be caused by an invalid file")]
    IOError(#[from] io::Error),
    #[error("Can't decode/read an entry of the code_table.bin file")]
    CantReadEntry(#[source] binread::Error),
}

impl CodeTable {
    pub fn new_from_file<F: Read + Seek>(file: F) -> Result<Self, CodeTableDecodeError> {
        let mut result = Self::default();
        let mut sir0 = Sir0::new(file)?;
        if sir0.offsets_len() < 5 {
            return Err(CodeTableDecodeError::NotEnoughtPointer(sir0.offsets_len()));
        };

        for pointer_id in 3..sir0.offsets_len() - 2 {
            let pointer = *sir0.offsets_get(pointer_id).map_or_else(
                || Err(CodeTableDecodeError::CantGetOffsetForPointer(pointer_id)),
                Ok,
            )?;
            let sir0_file = sir0.get_file();
            sir0_file.seek(SeekFrom::Start(pointer))?;
            let entry: CodeTableEntryFile = sir0_file
                .read_le()
                .map_or_else(|err| Err(CodeTableDecodeError::CantReadEntry(err)), Ok)?;
            result.entries.push(entry);
        }

        Ok(result)
    }

    pub fn add_missing(&mut self) {
        todo!();
    }

    pub fn generate_code_to_text(&self) -> CodeToText {
        let mut code_to_text = BTreeMap::new();
        for entry in self.entries.iter() {
            code_to_text.insert(entry.value, entry);
        };

        CodeToText {
            code_to_text
        }
    }

    pub fn generate_text_to_code(
        &self,
    ) -> TextToCode {
        let mut text_to_code = HashMap::new();
        for entry in self.entries.iter() {
            text_to_code.insert(&entry.string, entry);
        }

        TextToCode {
            text_to_code
        }
    }

    pub fn entries(&self) -> &Vec<CodeTableEntryFile> {
        &self.entries
    }
}