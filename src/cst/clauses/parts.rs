use erl_tokenize::tokens::{KeywordToken, SymbolToken, VariableToken};
use erl_tokenize::values::{Keyword, Symbol};
use erl_tokenize::{Position, PositionRange};

use super::super::commons::parts::{Clauses, Sequence};
use super::super::commons::AtomOrVariable;
use super::super::GuardTest;
use crate::traits::{Parse, TokenRead};
use crate::{Parser, Result};

/// `AtomOrVariable` `:`
#[derive(Debug, Clone)]
pub struct ExceptionClass {
    pub class: AtomOrVariable,
    pub _colon: SymbolToken,
}
impl Parse for ExceptionClass {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(ExceptionClass {
            class: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
        })
    }
}
impl PositionRange for ExceptionClass {
    fn start_position(&self) -> Position {
        self.class.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct StackTrace {
    pub _colon: SymbolToken,
    pub variable: VariableToken,
}
impl Parse for StackTrace {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(StackTrace {
            _colon: track!(parser.expect(&Symbol::Colon))?,
            variable: track!(parser.parse())?,
        })
    }
}
impl PositionRange for StackTrace {
    fn start_position(&self) -> Position {
        self._colon.start_position()
    }
    fn end_position(&self) -> Position {
        self.variable.end_position()
    }
}

/// `when` `Clauses<Sequence<GuardTest>>`
#[derive(Debug, Clone)]
pub struct WhenGuard {
    pub _when: KeywordToken,
    pub seq: Clauses<Sequence<GuardTest>>,
}
impl Parse for WhenGuard {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(WhenGuard {
            _when: track!(parser.expect(&Keyword::When))?,
            seq: track!(parser.parse())?,
        })
    }
}
impl PositionRange for WhenGuard {
    fn start_position(&self) -> Position {
        self._when.start_position()
    }
    fn end_position(&self) -> Position {
        self.seq.end_position()
    }
}
