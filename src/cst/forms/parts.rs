use erl_tokenize::tokens::{AtomToken, SymbolToken};
use erl_tokenize::values::Symbol;
use erl_tokenize::{Position, PositionRange};

use super::super::Expr;
use super::Type;
use crate::traits::{Parse, TokenRead};
use crate::{Parser, Result};

/// `AtomToken` `Option<RecordFieldDefault>` `Option<RecordFieldType>`
#[derive(Debug, Clone)]
pub struct RecordFieldDecl {
    pub field_name: AtomToken,
    pub field_default: Option<RecordFieldDefault>,
    pub field_type: Option<RecordFieldType>,
}
impl Parse for RecordFieldDecl {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(RecordFieldDecl {
            field_name: track!(parser.parse())?,
            field_default: track!(parser.parse())?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldDecl {
    fn start_position(&self) -> Position {
        self.field_name.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type
            .as_ref()
            .map(|t| t.end_position())
            .or_else(|| self.field_default.as_ref().map(|t| t.end_position()))
            .unwrap_or_else(|| self.field_name.end_position())
    }
}

/// `=` `Expr`
#[derive(Debug, Clone)]
pub struct RecordFieldDefault {
    pub _match: SymbolToken,
    pub value: Expr,
}
impl Parse for RecordFieldDefault {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(RecordFieldDefault {
            _match: track!(parser.expect(&Symbol::Match))?,
            value: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldDefault {
    fn start_position(&self) -> Position {
        self._match.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

/// `::` `Type`
#[derive(Debug, Clone)]
pub struct RecordFieldType {
    pub _double_colon: SymbolToken,
    pub field_type: Type,
}
impl Parse for RecordFieldType {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(RecordFieldType {
            _double_colon: track!(parser.expect(&Symbol::DoubleColon))?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldType {
    fn start_position(&self) -> Position {
        self._double_colon.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type.end_position()
    }
}
