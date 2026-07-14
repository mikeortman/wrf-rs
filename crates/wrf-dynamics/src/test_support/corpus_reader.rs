use std::str::SplitWhitespace;

pub(crate) struct CorpusReader<'a> {
    tokens: SplitWhitespace<'a>,
}

impl<'a> CorpusReader<'a> {
    pub(crate) fn new(corpus: &'a str) -> Self {
        Self {
            tokens: corpus.split_whitespace(),
        }
    }

    pub(crate) fn read_usize(&mut self, context: &str) -> usize {
        self.read_token(context)
            .parse()
            .unwrap_or_else(|error| panic!("invalid usize for {context}: {error}"))
    }

    pub(crate) fn read_i32(&mut self, context: &str) -> i32 {
        self.read_token(context)
            .parse()
            .unwrap_or_else(|error| panic!("invalid i32 for {context}: {error}"))
    }

    pub(crate) fn read_seed(&mut self) -> u64 {
        self.read_token("case seed")
            .parse()
            .unwrap_or_else(|error| panic!("invalid case seed: {error}"))
    }

    pub(crate) fn read_f32(&mut self, context: &str) -> f32 {
        f32::from_bits(self.read_i32(context) as u32)
    }

    pub(crate) fn finish(mut self) {
        assert_eq!(
            self.tokens.next(),
            None,
            "randomized input corpus contains trailing tokens"
        );
    }

    fn read_token(&mut self, context: &str) -> &'a str {
        self.tokens
            .next()
            .unwrap_or_else(|| panic!("randomized input corpus ended while reading {context}"))
    }
}
