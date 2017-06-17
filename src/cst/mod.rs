use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, StringToken,
                           VariableToken, SymbolToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parse, Preprocessor, Parser, ErrorKind, TryInto};

pub mod building_blocks;
pub mod collections;
pub mod exprs;

#[derive(Debug)]
pub enum RightKind {
    LocalCall,
    RemoteCall,
    None,
}
impl RightKind {
    fn guess<T>(parser: &mut Parser<T>) -> Self
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        match parser.read_token() {
            Ok(LexicalToken::Symbol(t)) => {
                match t.value() {
                    Symbol::OpenParen => RightKind::LocalCall,
                    Symbol::Colon => RightKind::RemoteCall,
                    _ => RightKind::None,
                }
            }
            _ => RightKind::None,
        }
    }
}

#[derive(Debug)]
pub enum LeftKind {
    Literal,
    Variable,
    Tuple,
    Map,
    Record,
    List,
    ListComprehension,
    Block,
    Parenthesized,
    Catch,
}
impl LeftKind {
    fn guess<T, U>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        U: Parse,
    {
        Ok(match track!(parser.read_token())? {
            LexicalToken::Symbol(t) => {
                match t.value() {
                    Symbol::OpenBrace => LeftKind::Tuple,
                    Symbol::OpenParen => LeftKind::Parenthesized,
                    Symbol::OpenSquare => {
                        let maybe_comprehension = parser.parse::<U>().is_ok() &&
                            parser
                                .expect::<SymbolToken>(&Symbol::DoubleVerticalBar)
                                .is_ok();
                        if maybe_comprehension {
                            LeftKind::ListComprehension
                        } else {
                            LeftKind::List
                        }
                    }
                    Symbol::Sharp => {
                        if track!(parser.read_token())?.as_atom_token().is_some() {
                            LeftKind::Record
                        } else {
                            LeftKind::Map
                        }
                    }
                    _ => track_panic!(ErrorKind::UnexpectedToken(t.into())),
                }
            }
            LexicalToken::Keyword(t) => {
                match t.value() {
                    Keyword::Begin => LeftKind::Block,
                    Keyword::Catch => LeftKind::Catch,
                    _ => track_panic!(ErrorKind::UnexpectedToken(t.into())),
                }
            }
            LexicalToken::Variable(_) => LeftKind::Variable,
            _ => LeftKind::Literal,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(VariableToken),
    Tuple(Box<exprs::Tuple>),
    Map(Box<exprs::Map>),
    Record(Box<exprs::Record>),
    List(Box<exprs::List>),
    ListComprehension(Box<exprs::ListComprehension>),
    Block(Box<exprs::Block>),
    Parenthesized(Box<exprs::Parenthesized>),
    Catch(Box<exprs::Catch>),
    LocalCall(Box<exprs::LocalCall>),
    RemoteCall(Box<exprs::RemoteCall>),
}
impl Parse for Expr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Expr>(parser)))?;
        let expr = match kind {
            LeftKind::Literal => Expr::Literal(track!(parser.parse())?),
            LeftKind::Variable => Expr::Variable(track!(parser.parse())?),
            LeftKind::Tuple => Expr::Tuple(track!(parser.parse())?),
            LeftKind::Map => Expr::Map(track!(parser.parse())?),
            LeftKind::Record => Expr::Record(track!(parser.parse())?),
            LeftKind::List => Expr::List(track!(parser.parse())?),            
            LeftKind::ListComprehension => Expr::ListComprehension(track!(parser.parse())?),
            LeftKind::Block => Expr::Block(track!(parser.parse())?),
            LeftKind::Parenthesized => Expr::Parenthesized(track!(parser.parse())?),
            LeftKind::Catch => Expr::Catch(track!(parser.parse())?),
        };

        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind::LocalCall => Ok(Expr::LocalCall(track!(parser.parse_left_recur(expr))?)),
            RightKind::RemoteCall => Ok(Expr::RemoteCall(track!(parser.parse_left_recur(expr))?)),
            RightKind::None => Ok(expr),
        }
    }
}
impl TryInto<exprs::LocalCall> for Expr {
    fn try_into(self) -> Result<exprs::LocalCall> {
        if let Expr::LocalCall(x) = self {
            Ok(*x)
        } else {
            track_panic!(ErrorKind::InvalidInput, "Not a LocalCall: {:?}", self)
        }
    }
}
impl PositionRange for Expr {
    fn start_position(&self) -> Position {
        match *self {
            Expr::Literal(ref x) => x.start_position(),
            Expr::Variable(ref x) => x.start_position(),
            Expr::Tuple(ref x) => x.start_position(),
            Expr::Map(ref x) => x.start_position(),
            Expr::Record(ref x) => x.start_position(),
            Expr::List(ref x) => x.start_position(),
            Expr::ListComprehension(ref x) => x.start_position(),
            Expr::Block(ref x) => x.start_position(),
            Expr::Parenthesized(ref x) => x.start_position(),
            Expr::Catch(ref x) => x.start_position(),
            Expr::LocalCall(ref x) => x.start_position(),
            Expr::RemoteCall(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Expr::Literal(ref x) => x.end_position(),
            Expr::Variable(ref x) => x.end_position(),
            Expr::Tuple(ref x) => x.end_position(),
            Expr::Map(ref x) => x.end_position(),
            Expr::Record(ref x) => x.end_position(),
            Expr::List(ref x) => x.end_position(),
            Expr::ListComprehension(ref x) => x.end_position(),
            Expr::Block(ref x) => x.end_position(),
            Expr::Parenthesized(ref x) => x.end_position(),
            Expr::Catch(ref x) => x.end_position(),
            Expr::LocalCall(ref x) => x.end_position(),
            Expr::RemoteCall(ref x) => x.end_position(),            
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Variable(VariableToken),
}
impl Parse for Pattern {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Pattern>(parser)))?;
        let pattern = match kind {
            LeftKind::Literal => Pattern::Literal(track!(parser.parse())?),
            LeftKind::Variable => Pattern::Variable(track!(parser.parse())?),
            _ => track_panic!(ErrorKind::UnexpectedToken(track!(parser.read_token())?)),
        };
        Ok(pattern)
    }
}
impl PositionRange for Pattern {
    fn start_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.start_position(),
            Pattern::Variable(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.end_position(),
            Pattern::Variable(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Atom(AtomToken),
    Char(CharToken),
    Float(FloatToken),
    Integer(IntegerToken),
    String(StringToken),
}
impl Parse for Literal {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        match track!(parser.read_token())? {
            LexicalToken::Atom(t) => Ok(Literal::Atom(t)),
            LexicalToken::Char(t) => Ok(Literal::Char(t)),
            LexicalToken::Float(t) => Ok(Literal::Float(t)),
            LexicalToken::Integer(t) => Ok(Literal::Integer(t)),
            LexicalToken::String(t) => Ok(Literal::String(t)),
            token => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for Literal {
    fn start_position(&self) -> Position {
        match *self {
            Literal::Atom(ref x) => x.start_position(),
            Literal::Char(ref x) => x.start_position(),
            Literal::Float(ref x) => x.start_position(),
            Literal::Integer(ref x) => x.start_position(),
            Literal::String(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Literal::Atom(ref x) => x.end_position(),
            Literal::Char(ref x) => x.end_position(),
            Literal::Float(ref x) => x.end_position(),
            Literal::Integer(ref x) => x.end_position(),
            Literal::String(ref x) => x.end_position(),
        }
    }
}
