/// A hacky wrapper to allow to "`Clone`" `Parser`s
pub struct FnParser<P>(fn() -> P, Option<P>);

impl<P> Clone for FnParser<P> {
    fn clone(&self) -> Self {
        FnParser(self.0, None)
    }
}

impl<P> FnParser<P> {
    pub fn new(init: fn() -> P) -> Self {
        Self(init, None)
    }
}

impl<'a, P: nom::Parser<&'a str>> nom::Parser<&'a str> for FnParser<P> {
    type Output = P::Output;
    type Error = P::Error;
    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a str,
    ) -> nom::PResult<OM, &'a str, Self::Output, Self::Error> {
        if let Some(parser) = self.1.as_mut() {
            parser.process::<OM>(input)
        } else {
            let mut parser = self.0();
            let res = parser.process::<OM>(input);
            self.1 = Some(parser);
            res
        }
    }
}
