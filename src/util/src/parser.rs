use logos::{Logos};
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Logos, Debug, PartialEq)]
enum Token<'a> {
    // CSS Tags take the form: element1, element2, .. elementN { ** CSS ** }
    // Regex Checks for any amount of words with a comma, followed by another word, followed by { ** CSS ** }
    // No return statement because these are not useful for indexing
    // #[regex(r"([A-z0-9^,]*,\s*)*[A-z0-9]+\s*\{[^\}]+\}")]
    // CSS,

    // HTML Elements take the forms <! **** COMMENT / DOCTYPE ****>, or <WORD attribute1=value attribute2=value>, or </WORD>
    // Regex first checks for a "<", then checks if there is a "!" character, in which case it will read until the next ">", since these are either comments or DOCTYPE declarations.
    // If no "!", it will look for "<" followed by an optional "/", followd by WORD, followed by any amount of "attribute=value", followed by optional whitespace and optional an "/", then ">"
    // No return statement because these are not useful for indexing
    #[regex(r"<(![^>]+|/?\w+((\s*[^\s=>])+=(\s*[^\s=>])+)*\s*/?)>")]
    HTMLTAG,

    // Regex checks for hyperlinks, which are words starting with http://, https://, or www., and any number of non-whitespace, html tags, or "/" is found (since including the specific subdirectory of the site is not useful for indexing)
    // The starting elements are then removed
    #[regex(r"(htt(p|ps)://|www\.)\S+")]
    HYPERLINK(&'a str),

    // Regex to check for emails, which take the form "word@word.word"
    // HTML tags and everything following @ is removed since searching for "gmail" to get a specific email address is unlikely
    #[regex(r"[A-z0-9]+@[A-z0-9]+\.[A-z0-9]+")]
    EMAIL(&'a str),

    // Regex to check for numbers, which include commas, decimals, and hyphens for phone numbers
    // Will not start with 0 since "01" and "1" should be the same in searches. Commas and hyphens are removed, as well as everything following the decimal since a search for "20.07" specifically would likely not be useful
    #[regex(r"[1-9](\d|,|\.|-)*")]
    NUMBER(&'a str),

    // Regex to remove common html entities like "&nbsp" which the parser was otherwise unable to detect
    // No return statement because these are not useful for indexing
    #[allow(non_camel_case_types)]
    #[regex(r"\&\w+")]
    HTML_ENTITY,

    // Words are similar to typical IDs, except with special inclusions for allowing specific punctuation so tokens don't become improperly split.
    // These start with an A-z character, and can be followed by more characters, digits, hyphens, apostrophes, html tags, and periods for abbreviations like "PH.D"
    // These additions are included so that "we'll" won't become "we" and "ll" separately, nor "<b>E</b>lephants" becoming "e" and "lephants". These are then removed with the re.sub expression to make for better indexing
    // Other punctuation marks, like ?, !, etc. don't typically connect words together, so these are not included
    #[regex(r"[A-z](\w|'|-|\.\w|<[^>]+>)*", priority=2)]
    WORD(&'a str),

    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r#"[ \[\]\+\$\|=%\*\{\}/\-#">\(\);:!\?\.,\t\xa0\x85\xe2\x00]+"#, logos::skip)]
    Error,
}

lazy_static! {
    static ref RE_CLEAN_LINK: Regex = Regex::new(r"(https://|http://|www|\.)").unwrap();
    static ref RE_CLEAN_EMAIL: Regex = Regex::new(r"(@.*|<[^>]+>)").unwrap();
    static ref RE_CLEAN_NUM: Regex = Regex::new(r"(,|-|\.\S*)").unwrap();
    static ref RE_CLEAN_WORD: Regex = Regex::new(r"(\.|-|'|<[^>]+>)").unwrap();
    static ref RE_CLEAN_NON_ASCII: Regex = Regex::new(r"[^\x00-\x7F]").unwrap();
}

fn clean_non_ascii(lex: &str) -> String {
    RE_CLEAN_NON_ASCII.replace_all(lex, "").to_string().to_ascii_lowercase()
}
fn clean_link(lex: &str) -> String {
    clean_non_ascii(&RE_CLEAN_LINK.replace_all(lex, "").to_string())
}
fn clean_email(lex: &str) -> String {
    clean_non_ascii(&RE_CLEAN_EMAIL.replace_all(lex, "").to_string())
}
fn clean_number(lex: &str) -> String {
    clean_non_ascii(&RE_CLEAN_NUM.replace_all(lex, "").to_string())
}
fn clean_word(lex: &str) -> String {
    clean_non_ascii(&RE_CLEAN_WORD.replace_all(lex, "").to_string())
}

pub fn parse(text: &str) -> Vec<String> {
    let mut vector = vec![];
    let mut lex = Token::lexer(text);
    while let Some(tok) = lex.next() {
        match tok {
            Token::HYPERLINK(token) => vector.push(clean_link(token)),
            Token::EMAIL(token) => vector.push(clean_email(token)),
            Token::NUMBER(token) => vector.push(clean_number(token)),
            Token::WORD(token) => vector.push(clean_word(token)),
            _ => ()
        }
    }
    vector
}