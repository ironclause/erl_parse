use erl_tokenize::LexicalToken;

use Result;
use traits::{Expect, Parse, ParseTail, TokenRead};

#[derive(Debug)]
pub struct Parser<T> {
    reader: T,
    // TODO: optimize
    transactions: Vec<Vec<LexicalToken>>,
}
impl<T> Parser<T>
where
    T: TokenRead,
{
    pub fn new(reader: T) -> Self {
        Parser {
            reader,
            transactions: Vec::new(),
        }
    }
    pub fn parse<P: Parse>(&mut self) -> Result<P> {
        track!(P::parse(self))
    }
    pub fn parse_tail<P: ParseTail>(&mut self, head: P::Head) -> Result<P> {
        track!(P::parse_tail(self, head))
    }
    pub fn expect<P: Parse + Expect>(&mut self, expected: &P::Value) -> Result<P> {
        self.transaction(|parser| {
            let actual = track!(parser.parse::<P>())?;
            track!(actual.expect(expected))?;
            Ok(actual)
        })
    }
    pub fn expect_any<P: Parse + Expect>(&mut self, expected: &[&P::Value]) -> Result<P> {
        let actual = track!(self.parse::<P>())?;
        let mut last_error = None;
        for e in expected.iter() {
            if let Err(e) = track!(actual.expect(e)) {
                last_error = Some(e);
            } else {
                last_error = None;
                break;
            }
        }
        if let Some(e) = last_error {
            Err(e)
        } else {
            Ok(actual)
        }
    }
    pub fn peek<F, P>(&mut self, f: F) -> Result<P>
    where
        F: FnOnce(&mut Self) -> Result<P>,
    {
        self.start_transaction();
        let result = track!(f(self));
        self.abort_transaction();
        result
    }
    pub fn transaction<F, P>(&mut self, f: F) -> Result<P>
    where
        F: FnOnce(&mut Self) -> Result<P>,
    {
        self.start_transaction();
        let result = track!(f(self));
        if result.is_ok() {
            self.commit_transaction();
        } else {
            self.abort_transaction();
        }
        result
    }
    pub fn eos(&mut self) -> Result<bool> {
        if let Some(t) = track!(self.reader.try_read_token())? {
            self.reader.unread_token(t);
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<LexicalToken> {
        match self.reader.read_token() {
            Err(e) => Err(e),
            Ok(t) => {
                if let Some(tail) = self.transactions.last_mut() {
                    tail.push(t.clone());
                }
                Ok(t)
            }
        }
    }

    fn start_transaction(&mut self) {
        self.transactions.push(Vec::new());
    }
    fn commit_transaction(&mut self) {
        let last = self.transactions.pop().unwrap();
        if let Some(tail) = self.transactions.last_mut() {
            tail.extend(last);
        }
    }
    fn abort_transaction(&mut self) {
        let last = self.transactions.pop().unwrap();
        for t in last.into_iter().rev() {
            self.reader.unread_token(t);
        }
    }
}
impl<T> Parser<T> {
    pub fn reader(&self) -> &T {
        &self.reader
    }
    pub fn reader_mut(&mut self) -> &mut T {
        &mut self.reader
    }
    pub fn into_reader(self) -> T {
        self.reader
    }
}
