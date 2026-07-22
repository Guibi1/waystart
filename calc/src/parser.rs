use crate::CalcError;
use crate::lexer::Token;
use crate::quantity::Quantity;
use crate::unit;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// whether an explicit conversion (`to`/`in`/trailing unit) was applied
    pub converted: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            converted: false,
        }
    }

    pub fn parse(mut self) -> Result<(Quantity, bool), CalcError> {
        let q = self.parse_conversion()?;
        if let Some(tok) = self.peek() {
            return Err(CalcError::UnexpectedToken(format!("{}", tok)));
        }
        Ok((q, self.converted))
    }

    /// An expression followed by optional conversions:
    /// `expr to unit`, `expr in unit`, or a bare trailing unit (`5 in ft`).
    fn parse_conversion(&mut self) -> Result<Quantity, CalcError> {
        let mut q = self.parse_expr()?;
        loop {
            match self.peek() {
                Some(Token::Ident(name)) if is_keyword(name, "to") || is_keyword(name, "in") => {
                    self.pos += 1;
                    let target = self.parse_expr()?;
                    q = q.convert_to(&target)?;
                    self.converted = true;
                }
                // implicit conversion: a unit directly after a quantity
                Some(Token::Ident(name)) if unit::lookup(name).is_some() => {
                    let target = self.parse_expr()?;
                    q = q.convert_to(&target)?;
                    self.converted = true;
                }
                _ => break,
            }
        }
        Ok(q)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn parse_expr(&mut self) -> Result<Quantity, CalcError> {
        self.parse_add()
    }

    fn parse_add(&mut self) -> Result<Quantity, CalcError> {
        let mut lhs = self.parse_mul()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let rhs = self.parse_mul()?;
                    lhs = lhs.add(&rhs)?;
                }
                Some(Token::Minus) => {
                    self.advance();
                    let rhs = self.parse_mul()?;
                    lhs = lhs.sub(&rhs)?;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Quantity, CalcError> {
        let mut lhs = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = lhs.mul(&rhs)?;
                }
                Some(Token::Slash) => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = lhs.div(&rhs)?;
                }
                // `50% of 200`, `8 mod 3`
                Some(Token::Ident(name)) if is_keyword(name, "of") || is_keyword(name, "mod") => {
                    let is_mod = is_keyword(name, "mod");
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = if is_mod {
                        if !lhs.is_dimensionless() || !rhs.is_dimensionless() {
                            return Err(CalcError::DimensionfulModulo);
                        }
                        if rhs.si == 0.0 {
                            return Err(CalcError::DivisionByZero);
                        }
                        Quantity::plain(lhs.si % rhs.si)
                    } else {
                        lhs.mul(&rhs)?
                    };
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Quantity, CalcError> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                self.parse_unary()?.neg()
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_percent(),
        }
    }

    fn parse_percent(&mut self) -> Result<Quantity, CalcError> {
        let mut q = self.parse_power()?;
        while matches!(self.peek(), Some(Token::Percent)) {
            self.advance();
            q = q.percent();
        }
        Ok(q)
    }

    fn parse_power(&mut self) -> Result<Quantity, CalcError> {
        let base = self.parse_compact()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let exp = self.parse_unary()?; // right associative
            return base.pow(&exp);
        }
        Ok(base)
    }

    /// A primary with a tightly-bound trailing unit (`30m`, `2s`, `2m²`)
    /// or constant (`2pi`). Binds tighter than `*`/`/` so `30m/2s` = (30 m)/(2 s).
    /// A second juxtaposed unit is left for the conversion layer (`5 in ft`).
    fn parse_compact(&mut self) -> Result<Quantity, CalcError> {
        let mut q = self.parse_primary()?;
        let mut attached_unit = false;
        loop {
            let name = match self.peek() {
                Some(Token::Ident(name)) => name.clone(),
                _ => break,
            };
            if is_keyword(&name, "to") || is_keyword(&name, "of") || is_keyword(&name, "mod") {
                break;
            }
            if let Some(def) = unit::lookup(&name) {
                if attached_unit {
                    break;
                }
                self.advance();
                let mut exp: i8 = 1;
                // exponent binds to the unit: `2m^2` = 2 m², `3s^-1`
                if matches!(self.peek(), Some(Token::Caret)) {
                    let neg = matches!(self.peek_at(1), Some(Token::Minus));
                    let num_idx = if neg { 2 } else { 1 };
                    if let Some(Token::Number(n)) = self.peek_at(num_idx) {
                        if n.fract() == 0.0 && n.abs() <= 16.0 {
                            exp = *n as i8;
                            if neg {
                                exp = -exp;
                            }
                            self.pos += num_idx + 1;
                        }
                    }
                }
                q = q.with_unit(def.disp_unit(exp), scale_dims(def.dims, exp))?;
                attached_unit = true;
            } else if is_keyword(&name, "pi") {
                self.advance();
                q = q.mul(&Quantity::plain(std::f64::consts::PI))?;
            } else {
                break;
            }
        }
        Ok(q)
    }

    fn parse_primary(&mut self) -> Result<Quantity, CalcError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Quantity::plain(n)),
            Some(Token::Ident(name)) => {
                if is_keyword(&name, "pi") {
                    Ok(Quantity::plain(std::f64::consts::PI))
                } else if let Some(def) = unit::lookup(&name) {
                    Ok(Quantity::plain(1.0).with_unit(def.disp_unit(1), def.dims)?)
                } else {
                    Err(CalcError::UnknownUnit(name))
                }
            }
            Some(Token::LParen) => {
                let q = self.parse_conversion()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(q),
                    _ => Err(CalcError::UnclosedParen),
                }
            }
            Some(tok) => Err(CalcError::UnexpectedToken(format!("{}", tok))),
            None => Err(CalcError::Empty),
        }
    }
}

fn is_keyword(ident: &str, keyword: &str) -> bool {
    ident.eq_ignore_ascii_case(keyword)
}

fn scale_dims(d: crate::quantity::Dims, n: i8) -> crate::quantity::Dims {
    let mut out = d;
    for e in &mut out {
        *e *= n;
    }
    out
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "'{}'", s),
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Star => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Percent => write!(f, "'%'"),
            Token::Caret => write!(f, "'^'"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
        }
    }
}
