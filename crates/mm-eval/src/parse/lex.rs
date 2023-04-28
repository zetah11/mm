use logos::Logos;

#[derive(Debug, Eq, PartialEq, Logos)]
#[logos(skip r"\s+")]
pub enum Token<'src> {
    #[regex(r"\p{XID_Start}[\p{XID_Continue}_']*", |lex| lex.slice())]
    Name(&'src str),

    #[regex(r"[0-9][0-9_]*", |lex| lex.slice())]
    Number(&'src str),

    #[token("<>")]
    Pause,

    #[token("=")]
    Equal,
    #[token(",")]
    Comma,
    #[token("|")]
    Pipe,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,

    #[token("#")]
    Sharp,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
}

/*

it = bonk

*/
