mod lexer;
mod parser;
mod quantity;
mod unit;

pub use quantity::Quantity;

use lexer::Token;
use quantity::format_number;

/// Evaluate a math expression with unit support, Raycast-style.
///
/// Bare `value unit` queries convert to a sensible default target:
///
/// ```
/// let q = calc::eval("30m/2s").unwrap();
/// assert_eq!(q.to_string(), "15 m/s");
/// let q = calc::eval("5c").unwrap();
/// assert_eq!(q.to_string(), "41 °F");
/// ```
pub fn eval(input: &str) -> Result<Quantity, CalcError> {
    Ok(run(input)?.quantity)
}

/// Structured result of a conversion query.
pub struct Conversion {
    /// the input value (in the source unit)
    pub value: f64,
    /// the source unit, if the input started with `value unit`
    pub from: Option<UnitInfo>,
    /// the resulting unit, if the result has a single unit
    pub to: Option<UnitInfo>,
    /// the result value (in the target unit)
    pub result: f64,
    /// human-readable display string
    pub display: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitInfo {
    pub id: &'static str,
    pub name: &'static str,
}

/// Evaluate a conversion query, Raycast-style: `5c`, `1 km to mi`,
/// `100 celcius to fahrenheit`. Bare `value unit` inputs convert to a
/// default target unit (`5c` → 41 Fahrenheit).
pub fn convert(input: &str) -> Result<Conversion, CalcError> {
    let outcome = run(input)?;
    let q = &outcome.quantity;
    let from = outcome
        .leading
        .map(|(_, def)| UnitInfo {
            id: def.id,
            name: def.name,
        });
    let to = if q.disp.len() == 1 {
        Some(UnitInfo {
            id: q.disp[0].id,
            name: q.disp[0].name,
        })
    } else {
        None
    };
    let result = q.display_value();
    let value = outcome.leading.map(|(v, _)| v).unwrap_or(result);
    let display = match (from, to) {
        (Some(from), Some(to)) if outcome.converted => format!(
            "{} {} = {} {}",
            format_number(value),
            from.name,
            format_number(result),
            to.name
        ),
        _ => q.to_string(),
    };
    Ok(Conversion {
        value,
        from,
        to,
        result,
        display,
    })
}

struct Outcome {
    quantity: Quantity,
    /// explicit or default conversion was applied
    converted: bool,
    /// leading `value unit` of the input, if any
    leading: Option<(f64, &'static unit::UnitDef)>,
}

fn run(input: &str) -> Result<Outcome, CalcError> {
    let tokens = lexer::lex(input)?;
    if tokens.is_empty() {
        return Err(CalcError::Empty);
    }
    let (leading, simple) = leading_unit(&tokens);
    let (mut quantity, mut converted) = parser::Parser::new(tokens).parse()?;

    // bare `value unit` query: convert to the default target (`5c` → °F)
    if simple && !converted {
        if let Some((_, def)) = leading {
            if let Some(target) = unit::default_target(def) {
                quantity = quantity.convert_to(&unit_quantity(target))?;
                converted = true;
            }
        }
    }

    Ok(Outcome {
        quantity,
        converted,
        leading,
    })
}

fn unit_quantity(def: &'static unit::UnitDef) -> Quantity {
    Quantity::plain(1.0)
        .with_unit(def.disp_unit(1), def.dims)
        .expect("a bare unit always attaches")
}

/// Extract a leading `(number) (unit)` pair; `simple` is true when that
/// pair is the entire input (optionally signed).
fn leading_unit(tokens: &[Token]) -> (Option<(f64, &'static unit::UnitDef)>, bool) {
    let mut idx = 0;
    let mut sign = 1.0;
    while let Some(tok) = tokens.get(idx) {
        match tok {
            Token::Minus => sign = -sign,
            Token::Plus => {}
            _ => break,
        }
        idx += 1;
    }
    let leading = match (tokens.get(idx), tokens.get(idx + 1)) {
        (Some(Token::Number(n)), Some(Token::Ident(name))) => {
            unit::lookup(name).map(|def| (sign * n, def))
        }
        _ => None,
    };
    let simple = leading.is_some() && tokens.len() == idx + 2;
    (leading, simple)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    Empty,
    InvalidNumber(String),
    UnexpectedChar(char),
    UnexpectedToken(String),
    UnclosedParen,
    UnknownUnit(String),
    DivisionByZero,
    NotFinite,
    DimensionOverflow,
    DimensionfulExponent,
    FractionalDimensionfulExponent,
    DimensionfulModulo,
    AffineArithmetic,
    InvalidConversionTarget,
    MismatchedDimensions { from: String, to: String },
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::Empty => write!(f, "empty expression"),
            CalcError::InvalidNumber(n) => write!(f, "invalid number: {n}"),
            CalcError::UnexpectedChar(c) => write!(f, "unexpected character '{c}'"),
            CalcError::UnexpectedToken(t) => write!(f, "unexpected token {t}"),
            CalcError::UnclosedParen => write!(f, "unclosed parenthesis"),
            CalcError::UnknownUnit(name) => write!(f, "unknown unit or variable '{name}'"),
            CalcError::DivisionByZero => write!(f, "division by zero"),
            CalcError::NotFinite => write!(f, "result is not finite"),
            CalcError::DimensionOverflow => write!(f, "unit exponent too large"),
            CalcError::DimensionfulExponent => write!(f, "exponent must be dimensionless"),
            CalcError::FractionalDimensionfulExponent => {
                write!(f, "fractional exponents require a dimensionless base")
            }
            CalcError::DimensionfulModulo => write!(f, "modulo requires dimensionless operands"),
            CalcError::AffineArithmetic => {
                write!(f, "temperature units cannot be used in compound arithmetic")
            }
            CalcError::InvalidConversionTarget => write!(f, "conversion target is not a unit"),
            CalcError::MismatchedDimensions { from, to } => {
                write!(f, "incompatible units: {from} and {to}")
            }
        }
    }
}

impl std::error::Error for CalcError {}
