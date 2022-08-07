use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
enum Token<'a> {
    // Tokens can be literal strings, of any length.
    #[token("fast")]
    Fast,

    #[token(".")]
    Period,

    // Or regular expressions.
    #[regex("[a-zA-Z]+")]
    Text(&'a str),

    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub fn parse(text: &str) -> Vec<&str> {
    let mut vector = vec![];
    let mut lex = Token::lexer(text);
    while let Some(tok) = lex.next() {
        match tok {
            Token::Text(num) => vector.push(num),
            _ => ()
        }
    }
    vector
}