// Copyright (C) 2024 Mathew Robinson <chasinglogic@gmail.com>
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.
//
use serde::Deserialize;

use crate::comments::BlockComment;
use crate::comments::Comment;
use crate::comments::LineComment;

use super::RegexList;

fn def_trailing_lines() -> usize {
    0
}

pub fn get_filetype(filename: &str) -> &str {
    let iter = filename.split('.');
    iter.last().unwrap_or_default()
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Commenter {
    #[serde(alias = "block")]
    Block {
        start_block_char: String,
        end_block_char: String,
        per_line_char: Option<String>,
        #[serde(default = "def_trailing_lines")]
        trailing_lines: usize,
    },
    #[serde(alias = "line")]
    Line {
        comment_char: String,
        #[serde(default = "def_trailing_lines")]
        trailing_lines: usize,
    },
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
enum FileType {
    Single(String),
    List(Vec<String>),
}

impl FileType {
    fn matches(&self, ft: &str) -> bool {
        match self {
            FileType::Single(ext) => ext == "any" || ext == ft,
            FileType::List(ref extensions) => extensions.iter().any(|ext| ext == ft),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(alias = "extensions")]
    extension: FileType,
    #[serde(default)]
    files: Option<RegexList>,
    columns: Option<usize>,
    commenter: Commenter,
}

impl Config {
    pub fn default() -> Config {
        Config {
            extension: FileType::Single("any".to_string()),
            files: None,
            columns: None,
            commenter: Commenter::Line {
                comment_char: "#".to_string(),
                trailing_lines: 0,
            },
        }
    }

    pub fn matches(&self, file_type: &str, filename: &str) -> bool {
        if self.extension.matches(file_type) {
            if let Some(files) = &self.files {
                files.is_match(filename)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn commenter(&self) -> Box<dyn Comment> {
        match &self.commenter {
            Commenter::Line {
                comment_char,
                trailing_lines,
            } => Box::new(
                LineComment::new(comment_char.as_str(), self.get_columns())
                    .set_trailing_lines(*trailing_lines),
            ),
            Commenter::Block {
                start_block_char,
                end_block_char,
                per_line_char,
                trailing_lines,
            } => {
                let mut bc = BlockComment::new(
                    start_block_char.as_str(),
                    end_block_char.as_str(),
                    self.get_columns(),
                )
                .set_trailing_lines(*trailing_lines);

                if let Some(ch) = per_line_char {
                    bc = bc.with_per_line(ch.as_str());
                }

                Box::new(bc)
            }
        }
    }

    pub fn get_columns(&self) -> Option<usize> {
        self.columns
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_get_filetype() {
        assert_eq!("py", get_filetype("test.py"))
    }
}
