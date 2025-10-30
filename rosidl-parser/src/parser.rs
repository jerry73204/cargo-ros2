use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, got {got}")]
    UnexpectedToken { expected: String, got: String },

    #[error("Unexpected end of input")]
    UnexpectedEOF,

    #[error("Invalid integer literal: {0}")]
    InvalidInteger(String),

    #[error("Invalid float literal: {0}")]
    InvalidFloat(String),

    #[error("Unknown type: {0}")]
    UnknownType(String),

    #[error("Lexer error: {0}")]
    LexerError(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<String> {
        match self.advance() {
            Some(token) if token.kind == kind => Ok(token.text.clone()),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                got: token.text.clone(),
            }),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_integer(&self, text: &str, kind: &TokenKind) -> ParseResult<i64> {
        let result = match kind {
            TokenKind::HexInteger => i64::from_str_radix(&text[2..], 16),
            TokenKind::BinaryInteger => i64::from_str_radix(&text[2..], 2),
            TokenKind::OctalInteger => i64::from_str_radix(&text[2..], 8),
            TokenKind::DecimalInteger => text.parse(),
            _ => return Err(ParseError::InvalidInteger(text.to_string())),
        };
        result.map_err(|_| ParseError::InvalidInteger(text.to_string()))
    }

    fn parse_field_type(&mut self) -> ParseResult<FieldType> {
        let token = self.advance().ok_or(ParseError::UnexpectedEOF)?;

        let base_type = match &token.kind {
            // Primitive types
            TokenKind::Bool => FieldType::Primitive(PrimitiveType::Bool),
            TokenKind::Byte => FieldType::Primitive(PrimitiveType::Byte),
            TokenKind::Char => FieldType::Primitive(PrimitiveType::Char),
            TokenKind::Int8 => FieldType::Primitive(PrimitiveType::Int8),
            TokenKind::UInt8 => FieldType::Primitive(PrimitiveType::UInt8),
            TokenKind::Int16 => FieldType::Primitive(PrimitiveType::Int16),
            TokenKind::UInt16 => FieldType::Primitive(PrimitiveType::UInt16),
            TokenKind::Int32 => FieldType::Primitive(PrimitiveType::Int32),
            TokenKind::UInt32 => FieldType::Primitive(PrimitiveType::UInt32),
            TokenKind::Int64 => FieldType::Primitive(PrimitiveType::Int64),
            TokenKind::UInt64 => FieldType::Primitive(PrimitiveType::UInt64),
            TokenKind::Float32 => FieldType::Primitive(PrimitiveType::Float32),
            TokenKind::Float64 => FieldType::Primitive(PrimitiveType::Float64),

            // String types
            TokenKind::String => {
                // Check for bounded string (string<=N)
                if matches!(self.current().map(|t| &t.kind), Some(TokenKind::LessEqual)) {
                    self.advance(); // consume <=
                    let size_token = self.advance().ok_or(ParseError::UnexpectedEOF)?;
                    let text = size_token.text.clone();
                    let kind = size_token.kind.clone();
                    let size = self.parse_integer(&text, &kind)?;
                    FieldType::BoundedString(size as usize)
                } else {
                    FieldType::String
                }
            }

            TokenKind::WString => {
                if matches!(self.current().map(|t| &t.kind), Some(TokenKind::LessEqual)) {
                    self.advance();
                    let size_token = self.advance().ok_or(ParseError::UnexpectedEOF)?;
                    let text = size_token.text.clone();
                    let kind = size_token.kind.clone();
                    let size = self.parse_integer(&text, &kind)?;
                    FieldType::BoundedWString(size as usize)
                } else {
                    FieldType::WString
                }
            }

            // Namespaced types (package/Type or Type)
            TokenKind::Identifier => {
                let name = token.text.clone();
                // Check for namespace separator
                if matches!(self.current().map(|t| &t.kind), Some(TokenKind::Slash)) {
                    self.advance(); // consume /
                    let type_name = self.expect(TokenKind::Identifier)?;
                    FieldType::NamespacedType {
                        package: Some(name),
                        name: type_name,
                    }
                } else {
                    FieldType::NamespacedType {
                        package: None,
                        name,
                    }
                }
            }

            _ => return Err(ParseError::UnknownType(token.text.clone())),
        };

        // Check for array/sequence specifiers
        if matches!(self.current().map(|t| &t.kind), Some(TokenKind::LBracket)) {
            self.advance(); // consume [

            match self.current().map(|t| &t.kind) {
                Some(TokenKind::RBracket) => {
                    // Unbounded sequence: type[]
                    self.advance();
                    Ok(FieldType::Sequence {
                        element_type: Box::new(base_type),
                    })
                }
                Some(TokenKind::LessEqual) => {
                    // Bounded sequence: type[<=N]
                    self.advance();
                    let size_token = self.advance().ok_or(ParseError::UnexpectedEOF)?;
                    let text = size_token.text.clone();
                    let kind = size_token.kind.clone();
                    let size = self.parse_integer(&text, &kind)?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(FieldType::BoundedSequence {
                        element_type: Box::new(base_type),
                        max_size: size as usize,
                    })
                }
                Some(
                    TokenKind::DecimalInteger
                    | TokenKind::HexInteger
                    | TokenKind::BinaryInteger
                    | TokenKind::OctalInteger,
                ) => {
                    // Fixed array: type[N]
                    let size_token = self.advance().ok_or(ParseError::UnexpectedEOF)?;
                    let text = size_token.text.clone();
                    let kind = size_token.kind.clone();
                    let size = self.parse_integer(&text, &kind)?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(FieldType::Array {
                        element_type: Box::new(base_type),
                        size: size as usize,
                    })
                }
                _ => Err(ParseError::UnexpectedToken {
                    expected: "array size or ]".to_string(),
                    got: self.current().map(|t| t.text.clone()).unwrap_or_default(),
                }),
            }
        } else {
            Ok(base_type)
        }
    }

    fn parse_constant_value(&mut self, _type_: &FieldType) -> ParseResult<ConstantValue> {
        let token = self.advance().ok_or(ParseError::UnexpectedEOF)?;
        let text = token.text.clone();
        let kind = token.kind.clone();

        match &kind {
            TokenKind::DecimalInteger
            | TokenKind::HexInteger
            | TokenKind::BinaryInteger
            | TokenKind::OctalInteger => {
                let value = self.parse_integer(&text, &kind)?;
                Ok(ConstantValue::Integer(value))
            }
            TokenKind::Float => {
                let value = text
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidFloat(text.clone()))?;
                Ok(ConstantValue::Float(value))
            }
            TokenKind::True => Ok(ConstantValue::Bool(true)),
            TokenKind::False => Ok(ConstantValue::Bool(false)),
            TokenKind::StringLiteral => {
                // Remove quotes
                let s = text.trim_matches(|c| c == '"' || c == '\'');
                Ok(ConstantValue::String(s.to_string()))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "constant value".to_string(),
                got: text,
            }),
        }
    }

    fn parse_field_or_constant(&mut self) -> ParseResult<(Option<Field>, Option<Constant>)> {
        let field_type = self.parse_field_type()?;
        let name = self.expect(TokenKind::Identifier)?;

        // Check if this is a constant (has = sign)
        if matches!(self.current().map(|t| &t.kind), Some(TokenKind::Equals)) {
            self.advance(); // consume =
            let value = self.parse_constant_value(&field_type)?;
            Ok((
                None,
                Some(Constant {
                    constant_type: field_type,
                    name,
                    value,
                }),
            ))
        } else {
            // It's a field, check for default value
            let default_value =
                if matches!(self.current().map(|t| &t.kind), Some(TokenKind::Equals)) {
                    self.advance(); // consume =
                    Some(self.parse_constant_value(&field_type)?)
                } else {
                    None
                };

            Ok((
                Some(Field {
                    field_type,
                    name,
                    default_value,
                }),
                None,
            ))
        }
    }

    fn parse_message_impl(&mut self) -> ParseResult<Message> {
        let mut message = Message::new();

        while self.current().is_some() {
            // Stop at triple dash (service/action separator)
            if matches!(self.current().map(|t| &t.kind), Some(TokenKind::TripleDash)) {
                break;
            }

            let (field, constant) = self.parse_field_or_constant()?;

            if let Some(field) = field {
                message.fields.push(field);
            }
            if let Some(constant) = constant {
                message.constants.push(constant);
            }
        }

        Ok(message)
    }
}

pub fn parse_message(input: &str) -> ParseResult<Message> {
    let tokens = crate::lexer::lex(input).map_err(ParseError::LexerError)?;
    let mut parser = Parser::new(tokens);
    parser.parse_message_impl()
}

pub fn parse_service(input: &str) -> ParseResult<Service> {
    let tokens = crate::lexer::lex(input).map_err(ParseError::LexerError)?;
    let mut parser = Parser::new(tokens);

    let request = parser.parse_message_impl()?;

    // Expect separator
    parser.expect(TokenKind::TripleDash)?;

    let response = parser.parse_message_impl()?;

    Ok(Service { request, response })
}

pub fn parse_action(input: &str) -> ParseResult<Action> {
    let tokens = crate::lexer::lex(input).map_err(ParseError::LexerError)?;
    let mut parser = Parser::new(tokens);

    let goal = parser.parse_message_impl()?;
    parser.expect(TokenKind::TripleDash)?;

    let result = parser.parse_message_impl()?;
    parser.expect(TokenKind::TripleDash)?;

    let feedback = parser.parse_message_impl()?;

    Ok(Action {
        spec: ActionSpec {
            goal,
            result,
            feedback,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_primitive_field() {
        let msg = parse_message("int32 x\nuint8 y\nfloat64 z\n").unwrap();
        assert_eq!(msg.fields.len(), 3);
        assert_eq!(msg.fields[0].name, "x");
    }

    #[test]
    fn parse_string_field() {
        let msg = parse_message("string name\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(msg.fields[0].field_type, FieldType::String));
    }

    #[test]
    fn parse_bounded_string() {
        let msg = parse_message("string<=256 name\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(
            msg.fields[0].field_type,
            FieldType::BoundedString(256)
        ));
    }

    #[test]
    fn parse_fixed_array() {
        let msg = parse_message("int32[5] data\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(msg.fields[0].field_type, FieldType::Array { .. }));
    }

    #[test]
    fn parse_unbounded_sequence() {
        let msg = parse_message("int32[] data\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(
            msg.fields[0].field_type,
            FieldType::Sequence { .. }
        ));
    }

    #[test]
    fn parse_bounded_sequence() {
        let msg = parse_message("int32[<=100] data\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(
            msg.fields[0].field_type,
            FieldType::BoundedSequence { .. }
        ));
    }

    #[test]
    fn parse_constant() {
        let msg = parse_message("int32 MAX_SIZE=100\n").unwrap();
        assert_eq!(msg.constants.len(), 1);
        assert_eq!(msg.constants[0].name, "MAX_SIZE");
        assert!(matches!(
            msg.constants[0].value,
            ConstantValue::Integer(100)
        ));
    }

    #[test]
    fn parse_hex_constant() {
        let msg = parse_message("int32 HEX=0xFF\n").unwrap();
        assert_eq!(msg.constants.len(), 1);
        assert!(matches!(
            msg.constants[0].value,
            ConstantValue::Integer(255)
        ));
    }

    #[test]
    fn parse_namespaced_type() {
        let msg = parse_message("geometry_msgs/Point position\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        if let FieldType::NamespacedType { package, name } = &msg.fields[0].field_type {
            assert_eq!(package.as_ref().unwrap(), "geometry_msgs");
            assert_eq!(name, "Point");
        } else {
            panic!("Expected NamespacedType");
        }
    }

    #[test]
    fn parse_simple_service() {
        let srv = parse_service("int64 a\nint64 b\n---\nint64 sum\n").unwrap();
        assert_eq!(srv.request.fields.len(), 2);
        assert_eq!(srv.response.fields.len(), 1);
    }

    #[test]
    fn parse_simple_action() {
        let act =
            parse_action("int32 order\n---\nint32[] sequence\n---\nint32[] sequence\n").unwrap();
        assert_eq!(act.spec.goal.fields.len(), 1);
        assert_eq!(act.spec.result.fields.len(), 1);
        assert_eq!(act.spec.feedback.fields.len(), 1);
    }
}
