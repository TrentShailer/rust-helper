#![allow(clippy::while_let_on_iterator)]

use core::ops::{Add, AddAssign};

use jsonschema::paths::{Location, LocationSegment};

/// A position in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The line number (not index).
    pub line: usize,
    /// The column number (not index).
    pub column: usize,
}
impl Add<char> for Position {
    type Output = Self;

    fn add(mut self, rhs: char) -> Self::Output {
        if rhs == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self
    }
}
impl AddAssign<char> for Position {
    fn add_assign(&mut self, rhs: char) {
        *self = self.add(rhs);
    }
}
impl Default for Position {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

#[derive(Debug)]
pub struct Tag {
    pub position: Position,
    pub value: String,
}

/// A JSON node with it's position in the source file.
#[derive(Debug)]
pub enum PositionedJsonNode {
    /// An object.
    Object {
        /// The object's position.
        position: Position,
        /// The object's properties.
        properties: Vec<(Tag, PositionedJsonNode)>,
    },
    /// An array.
    Array {
        /// The array's position.
        position: Position,
        /// The array's items.
        items: Vec<PositionedJsonNode>,
    },
    /// A value.
    Value {
        /// The value's position.
        position: Position,
        /// The value.
        value: String,
    },
}

impl PositionedJsonNode {
    /// Try parse a source file into a JSON node while tracking node positions.
    pub fn try_parse(src: &str) -> Option<Self> {
        let mut position = Position::default();
        let mut iter = src.chars();
        Self::parse(&mut position, &mut iter).map(|(node, ..)| node)
    }

    fn parse<T: Iterator<Item = char>>(
        current_position: &mut Position,
        src: &mut T,
    ) -> Option<(Self, Option<char>)> {
        while let Some(ch) = src.next() {
            if ch.is_whitespace() {
                *current_position += ch;
                continue;
            }

            if ch == '{' {
                *current_position += ch;
                let object = Self::parse_object(current_position, src)?;
                return Some((object, None));
            } else if ch == '[' {
                *current_position += ch;
                let array = Self::parse_array(current_position, src)?;
                return Some((array, None));
            } else if ch == '\"' {
                let position = *current_position;
                *current_position += ch;
                let value = Self::parse_string(current_position, src)?;
                return Some((Self::Value { position, value }, None));
            } else {
                let value = Self::parse_value(current_position, src, ch)?;
                return Some(value);
            }
        }

        None
    }

    fn parse_object<T: Iterator<Item = char>>(
        current_position: &mut Position,
        src: &mut T,
    ) -> Option<Self> {
        let position = *current_position;
        let mut properties = vec![];

        while let Some(ch) = src.next() {
            if ch == '\"' {
                let tag_position = *current_position;
                *current_position += ch;
                let tag = Self::parse_string(current_position, src)?;
                let tag = Tag {
                    position: tag_position,
                    value: tag,
                };

                let mut has_consumed_colon = false;
                while !has_consumed_colon && let Some(ch) = src.next() {
                    *current_position += ch;

                    if ch == ':' {
                        has_consumed_colon = true;
                    }
                }

                let (property, overeaten) = Self::parse(current_position, src)?;
                properties.push((tag, property));

                if let Some(overeaten) = overeaten
                    && overeaten == '}'
                {
                    break;
                }
            } else if ch == '}' {
                *current_position += ch;
                break;
            } else {
                *current_position += ch;
            }
        }

        Some(Self::Object {
            position,
            properties,
        })
    }

    fn parse_array<T: Iterator<Item = char>>(
        current_position: &mut Position,
        src: &mut T,
    ) -> Option<Self> {
        let position = *current_position;
        let mut items = vec![];

        while let Some(ch) = src.next() {
            if ch.is_whitespace() || ch == ',' {
                *current_position += ch;
                continue;
            } else if ch == ']' {
                *current_position += ch;
                break;
            } else if ch == '\"' {
                let position = *current_position;
                *current_position += ch;
                let value = Self::parse_string(current_position, src)?;
                items.push(Self::Value { position, value });
            } else if ch == '{' {
                *current_position += ch;
                let object = Self::parse_object(current_position, src)?;
                items.push(object);
            } else if ch == '[' {
                *current_position += ch;
                let array = Self::parse_array(current_position, src)?;
                items.push(array);
            } else {
                let (value, overeaten) = Self::parse_value(current_position, src, ch)?;
                items.push(value);

                if let Some(overeaten) = overeaten
                    && overeaten == ']'
                {
                    break;
                }
            }
        }

        Some(Self::Array { position, items })
    }

    fn parse_string<T: Iterator<Item = char>>(
        current_position: &mut Position,
        src: &mut T,
    ) -> Option<String> {
        let mut value = String::new();

        let mut is_escaped = false;
        while let Some(ch) = src.next() {
            *current_position += ch;

            if is_escaped {
                value.push(ch);
                is_escaped = false;
                continue;
            }

            if ch == '\\' {
                is_escaped = true;
            } else if ch == '\"' {
                break;
            } else {
                value.push(ch);
            }
        }

        Some(value)
    }

    fn parse_value<T: Iterator<Item = char>>(
        current_position: &mut Position,
        src: &mut T,
        first_char: char,
    ) -> Option<(Self, Option<char>)> {
        let position = *current_position;
        *current_position += first_char;
        let mut value = first_char.to_string();

        let mut overeaten = None;
        while let Some(ch) = src.next() {
            *current_position += ch;

            if ch.is_whitespace() || ch == ',' {
                break;
            } else if ch == '}' || ch == ']' {
                overeaten = Some(ch);
                break;
            } else {
                value.push(ch);
            }
        }

        Some((Self::Value { position, value }, overeaten))
    }

    /// Try evaluate a pointer to the node it is pointing at.
    pub fn evaluate(&self, pointer: &Location) -> Option<&Self> {
        let segments = pointer.into_iter();

        let mut current_node = self;
        for segment in segments {
            match segment {
                LocationSegment::Property(tag) => {
                    current_node = current_node.get(Index::Tag(tag))?
                }
                LocationSegment::Index(index) => {
                    current_node = current_node.get(Index::Index(index))?
                }
            }
        }

        Some(current_node)
    }

    /// Try index the node.
    pub fn get<'a, 'b>(&'b self, index: Index<'a>) -> Option<&'b Self> {
        match &self {
            Self::Object { properties, .. } => {
                if let Index::Tag(index_tag) = index {
                    properties.iter().find_map(|(tag, property)| {
                        if tag.value == index_tag {
                            Some(property)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            }
            Self::Array { items, .. } => {
                if let Index::Index(index) = index {
                    items.get(index)
                } else {
                    None
                }
            }
            Self::Value { .. } => None,
        }
    }

    /// Return the position of the node.
    pub fn position(&self) -> Position {
        match &self {
            Self::Object { position, .. } => *position,
            Self::Array { position, .. } => *position,
            Self::Value { position, .. } => *position,
        }
    }
}

pub enum Index<'a> {
    Tag(&'a str),
    Index(usize),
}
