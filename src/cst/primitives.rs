use std::ops::Deref;
use erl_tokenize::tokens::{AtomToken, IntegerToken};

use {Result, TokenReader, Parse, TokenRange};
use super::symbols;

// non empty
#[derive(Debug)]
pub struct Seq<T> {
    pub position: usize,
    pub items: Vec<SeqItem<T>>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Seq<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let mut items = Vec::new();
        loop {
            let x = track_try!(SeqItem::parse(reader));
            let is_last = x.delimiter.is_none();
            items.push(x);
            if is_last {
                break;
            }
        }
        Ok(Seq { position, items })
    }
}
impl<T> TokenRange for Seq<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.items.last().expect("Non empty").token_end()
    }
}

#[derive(Debug)]
pub struct SeqItem<T> {
    pub item: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for SeqItem<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let item = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(SeqItem { item, delimiter })
    }
}
impl<T> TokenRange for SeqItem<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.item.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.item.token_end(), |d| d.token_end())
    }
}

#[derive(Debug)]
pub struct Clauses<T> {
    pub position: usize,
    pub clauses: Vec<Clause<T>>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Clauses<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let mut clauses = Vec::new();
        loop {
            let c = track_try!(Clause::parse(reader));
            let is_last = c.delimiter.is_none();
            clauses.push(c);
            if is_last {
                break;
            }
        }
        Ok(Clauses { position, clauses })
    }
}
impl<T> TokenRange for Clauses<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.clauses.last().map_or(self.position, |c| c.token_end())
    }
}

#[derive(Debug)]
pub struct Clause<T> {
    pub clause: T,
    pub delimiter: Option<symbols::Semicolon>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Clause<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let clause = track_try!(T::parse(reader));
        let delimiter = symbols::Semicolon::try_parse(reader);
        Ok(Clause { clause, delimiter })
    }
}
impl<T> TokenRange for Clause<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.clause.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.clause.token_end(), |d| d.token_end())
    }
}


#[derive(Debug)]
pub struct List<T> {
    pub open: symbols::OpenSquare,
    pub elements: Vec<ListElement<T>>,
    pub close: symbols::CloseSquare,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for List<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let open = track_try!(symbols::OpenSquare::parse(reader));
        let mut elements = Vec::new();
        if let Some(close) = symbols::CloseSquare::try_parse(reader) {
            return Ok(List {
                          open,
                          elements,
                          close,
                      });
        }
        loop {
            let e = track_try!(ListElement::parse(reader));
            let is_last = e.delimiter.is_none();
            elements.push(e);
            if is_last {
                break;
            }
        }
        let close = track_try!(symbols::CloseSquare::parse(reader));
        Ok(List {
               open,
               elements,
               close,
           })
    }
}
impl<T> TokenRange for List<T> {
    fn token_start(&self) -> usize {
        self.open.token_start()
    }
    fn token_end(&self) -> usize {
        self.close.token_end()
    }
}

#[derive(Debug)]
pub struct ListElement<T> {
    pub value: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for ListElement<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let value = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(ListElement { value, delimiter })
    }
}
impl<T> TokenRange for ListElement<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.value.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.value.token_end(), |d| d.token_end())
    }
}

#[derive(Debug)]
pub struct Args<T> {
    pub open: symbols::OpenParen,
    pub args: Vec<Arg<T>>,
    pub close: symbols::CloseParen,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Args<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let open = track_try!(Parse::parse(reader));
        let mut args = Vec::new();
        if let Some(close) = symbols::CloseParen::try_parse(reader) {
            return Ok(Args { open, args, close });
        }

        loop {
            let a = track_try!(Arg::parse(reader));
            let is_last = a.delimiter.is_none();
            args.push(a);
            if is_last {
                break;
            }
        }
        let close = track_try!(Parse::parse(reader));
        Ok(Args { open, args, close })
    }
}
impl<T> TokenRange for Args<T> {
    fn token_start(&self) -> usize {
        self.open.token_start()
    }
    fn token_end(&self) -> usize {
        self.close.token_end()
    }
}

#[derive(Debug)]
pub struct Arg<T> {
    pub arg: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Arg<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let arg = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(Arg { arg, delimiter })
    }
}
impl<T> TokenRange for Arg<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.arg.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.arg.token_end(), |d| d.token_end())
    }
}

#[derive(Debug)]
pub struct Atom<'token, 'text: 'token> {
    position: usize,
    value: &'token AtomToken<'text>,
}
impl<'token, 'text: 'token> Deref for Atom<'token, 'text> {
    type Target = AtomToken<'text>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Atom<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_atom());
        Ok(Atom { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Atom<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct ModuleAtom<'token, 'text: 'token> {
    pub module_name: Atom<'token, 'text>,
    pub colon: symbols::Colon,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ModuleAtom<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(ModuleAtom {
               module_name: track_try!(Parse::parse(reader)),
               colon: track_try!(Parse::parse(reader)),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for ModuleAtom<'token, 'text> {
    fn token_start(&self) -> usize {
        self.module_name.token_start()
    }
    fn token_end(&self) -> usize {
        self.colon.token_end()
    }
}

#[derive(Debug)]
pub struct Integer<'token, 'text: 'token> {
    position: usize,
    value: &'token IntegerToken<'text>,
}
impl<'token, 'text: 'token> Deref for Integer<'token, 'text> {
    type Target = IntegerToken<'text>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Integer<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_integer());
        Ok(Integer { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Integer<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct Export<'token, 'text: 'token> {
    pub name: Atom<'token, 'text>,
    pub delimiter: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Export<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let name = track_try!(Atom::parse(reader));
        let delimiter = track_try!(symbols::Slash::parse(reader));
        let arity = track_try!(Integer::parse(reader));
        Ok(Export {
               name,
               delimiter,
               arity,
           })
    }
}
impl<'token, 'text: 'token> TokenRange for Export<'token, 'text> {
    fn token_start(&self) -> usize {
        self.name.token_start()
    }
    fn token_end(&self) -> usize {
        self.arity.token_end()
    }
}