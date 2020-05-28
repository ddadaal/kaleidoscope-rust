pub struct Buffer<I, T: Iterator<Item = I>> {
    iter: T,
    curr: Option<I>,
    next: Option<I>,
}

impl<I, T: Iterator<Item = I>> Buffer<I, T> {
    pub fn new(iter: T) -> Self {
        Buffer {
            iter,
            curr: None,
            next: None,
        }
    }

    pub fn init(&mut self) {
        self.curr = self.iter.next();
        self.next = self.iter.next();
    }

    pub fn curr(&self) -> Option<&I> {
        self.curr.as_ref()
    }

    pub fn peek(&self) -> Option<&I> {
        self.next.as_ref()
    }

    pub fn advance(&mut self) {
        self.curr = self.next.take();
        self.next = self.iter.next();
    }

    pub fn iter(&mut self) -> &T {
        &self.iter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_test() {
        let mut buffer = Buffer::new("123".chars());
        buffer.init();

        assert_eq!(buffer.curr(), Some(&'1'));
        assert_eq!(buffer.curr(), Some(&'1'));
        assert_eq!(buffer.peek(), Some(&'2'));
        assert_eq!(buffer.peek(), Some(&'2'));
        buffer.advance();
        assert_eq!(buffer.curr(), Some(&'2'));
        assert_eq!(buffer.peek(), Some(&'3'));
        buffer.advance();
        assert_eq!(buffer.curr(), Some(&'3'));
        assert_eq!(buffer.peek(), None);
        buffer.advance();
        assert_eq!(buffer.curr(), None);
        assert_eq!(buffer.peek(), None);
    }

    #[test]
    fn short_string_test() {
        let mut buffer = Buffer::new("12".chars());
        buffer.init();

        assert_eq!(buffer.curr(), Some(&'1'));
        assert_eq!(buffer.curr(), Some(&'1'));
        assert_eq!(buffer.peek(), Some(&'2'));
        assert_eq!(buffer.peek(), Some(&'2'));
        buffer.advance();
        assert_eq!(buffer.curr(), Some(&'2'));
        assert_eq!(buffer.peek(), None);
        buffer.advance();
        assert_eq!(buffer.curr(), None);
        assert_eq!(buffer.peek(), None);
    }
}
