pub trait Input: Clone {
    /// Get the current char.
    fn curr_char(&self) -> Option<char>;

    /// Peek the next char, but not consume it.
    fn peek_char(&self) -> Option<char>;

    /// Advance the input and get the next char.
    fn advance(&mut self) -> Option<char>;
}

#[derive(Clone)]
pub struct StringInput<'a> {
    iter: std::str::Chars<'a>,
    curr: Option<char>,
    next: Option<char>,
}

impl<'a> Input for StringInput<'a> {
    fn curr_char(&self) -> Option<char> {
        self.curr
    }

    fn peek_char(&self) -> Option<char> {
        self.next
    }

    fn advance(&mut self) -> Option<char> {
        self.curr = self.next;
        self.next = self.iter.next();
        self.curr
    }
}

impl<'a> StringInput<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut iter = s.chars();
        let curr = iter.next();
        let next = iter.next();

        StringInput { iter, curr, next }
    }
}

impl<'a> From<&'a str> for StringInput<'a> {
    fn from(s: &'a str) -> Self {
        StringInput::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_input_should_work() {
        let s = "123";
        let mut input: StringInput = s.into();

        assert_eq!(Some('1'), input.curr_char());
        assert_eq!(Some('1'), input.curr_char());
        assert_eq!(Some('2'), input.peek_char());
        assert_eq!(Some('2'), input.peek_char());
        assert_eq!(Some('2'), input.advance());
        assert_eq!(Some('2'), input.curr_char());
        assert_eq!(Some('3'), input.peek_char());
        assert_eq!(Some('3'), input.advance());
        assert_eq!(None, input.advance());
        assert_eq!(None, input.curr_char());
    }

    #[test]
    fn short_string_should_return_none() {
        let s = "1";
        let mut input: StringInput = s.into();

        assert_eq!(Some('1'), input.curr_char());
        assert_eq!(None, input.peek_char());
        assert_eq!(None, input.advance());
        assert_eq!(None, input.curr_char());
    }

    #[test]
    fn zero_len_string_should_return_none() {
        let s = "";
        let input: StringInput = s.into();

        assert_eq!(None, input.curr_char());
        assert_eq!(None, input.peek_char());
    }
}
