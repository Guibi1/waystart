use crate::CalcError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
}

pub fn lex(input: &str) -> Result<Vec<Token>, CalcError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            c if c.is_whitespace() => i += 1,
            c if c.is_ascii_digit()
                || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) =>
            {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                // scientific notation: 1e3, 2.5e-4 (only when digits follow)
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    let mut j = i + 1;
                    if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                        j += 1;
                    }
                    if j < chars.len() && chars[j].is_ascii_digit() {
                        i = j;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                let text: String = chars[start..i].iter().collect();
                let value: f64 = text
                    .parse()
                    .map_err(|_| CalcError::InvalidNumber(text.clone()))?;
                tokens.push(Token::Number(value));
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' | '−' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' | '×' | '·' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' | '÷' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '%' => {
                tokens.push(Token::Percent);
                i += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '²' => {
                tokens.push(Token::Caret);
                tokens.push(Token::Number(2.0));
                i += 1;
            }
            '³' => {
                tokens.push(Token::Caret);
                tokens.push(Token::Number(3.0));
                i += 1;
            }
            'π' => {
                tokens.push(Token::Ident("pi".into()));
                i += 1;
            }
            '°' => {
                // °C / °F are single temperature units; bare ° is degrees
                if i + 1 < chars.len() && (chars[i + 1] == 'C' || chars[i + 1] == 'c') {
                    tokens.push(Token::Ident("°c".into()));
                    i += 2;
                } else if i + 1 < chars.len() && (chars[i + 1] == 'F' || chars[i + 1] == 'f') {
                    tokens.push(Token::Ident("°f".into()));
                    i += 2;
                } else {
                    tokens.push(Token::Ident("deg".into()));
                    i += 1;
                }
            }
            '\'' => {
                tokens.push(Token::Ident("ft".into()));
                i += 1;
            }
            '"' => {
                tokens.push(Token::Ident("in".into()));
                i += 1;
            }
            c if c.is_alphabetic() || c == 'µ' || c == 'μ' || c == 'Ω' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == 'µ' || chars[i] == 'μ' || chars[i] == 'Ω') {
                    i += 1;
                }
                // case is preserved: `GB` (gigabyte) and `Gb` (gigabit) differ;
                // the unit registry decides what matches case-insensitively
                let name: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(name));
            }
            c => return Err(CalcError::UnexpectedChar(c)),
        }
    }

    Ok(tokens)
}
