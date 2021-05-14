use rustc_ap_rustc_lexer::unescape::{unescape_literal, EscapeError, Mode};

// edn tags? https://github.com/edn-format/edn
#[derive(Clone, Debug, PartialEq)]
pub enum ExprType {
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Hashmap(Vec<Expr>),
    // Cow?
    String(String),
    // intern?
    Symbol(String),
    //Keyword(String),
    Comment(String),
    Float(f64),
    Int(i64),
}

macro_rules! write_list {
    ($f:ident, $fmt:expr, $a:ident) => {
        write!(
            $f,
            $fmt,
            $a.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
}

impl std::fmt::Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(a) => write_list!(f, "({})", a),
            Self::Vector(a) => write_list!(f, "[{}]", a),
            Self::Hashmap(a) => write_list!(f, "{{{}}}", a),
            // does this need to be escaped?
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Symbol(s) => write!(f, "{}", s),
            Self::Float(n) => write!(f, "{}", n),
            Self::Int(n) => write!(f, "{}", n),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub prefix: Option<String>,
    pub expr: ExprType,
}

impl Expr {
    pub fn new(prefix: Option<String>, expr: ExprType) -> Self {
        Self { prefix, expr }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prefix.as_ref() {
            Some(prefix) => write!(f, "{}{}", prefix, self.expr),
            _ => write!(f, "{}", self.expr),
        }
    }
}

// macro_rules! expr {}

pub use parse::exprs as from_str;

pub fn unescape(s: &str) -> Result<String, (std::ops::Range<usize>, EscapeError)> {
    let mut buf = Ok(String::with_capacity(s.len()));
    unescape_literal(s, Mode::Str, &mut |range, c| {
        if let Ok(b) = &mut buf {
            match c {
                Ok(c) => b.push(c),
                Err(e) => buf = Err((range, e)),
            }
        }
    });
    buf
}

peg::parser! {
    pub grammar parse() for str {
        pub rule exprs() -> Vec<Expr>
            = _ e:(comment() / expr())* _ { e }

        // order matters
        pub rule expr() -> Expr
            = _ p:prefix()? e:(seq() / string() / float() / integer() / symbol()) _ { Expr::new(p, e) }

        rule seq() -> ExprType = e:list() / e:vector() / e:hashmap() { e }
        rule list() -> ExprType = "(" _ e:sep(<expr()>) _ ")" { ExprType::List(e) }
        rule vector() -> ExprType = "[" e:expr() ** _ _ "]" { ExprType::Vector(e) }
        rule hashmap() -> ExprType = "{" e:expr() ** _ _ "}" { ExprType::Hashmap(e) }

        rule nl() = "\r"? "\n"
        rule comment() -> Expr = e:$(";" (!nl() [_])*) { Expr::new(None, ExprType::Comment(e.to_string())) }
        //rule term() = comment()? nl()
        rule ws()
            = [' ' | '\t' | '\n' | '\r' | '\0' | '\u{0B}' | '\u{0C}' ] / "\\" nl()
        rule _()  = quiet!{ (ws())* }
        rule __() = quiet!{ (ws())+ }

        // quoting, etc.
        rule prefix() -> String = quiet!{ p:$(['\'' | ',' | '`' | '~' | '@' | '^']+) { p.to_string() } } / expected!("prefix")

        // commas are optional but a comma without a following space will be considered a prefix
        rule sep<T>(x: rule<T>) -> Vec<T> = v:(x() ** (_ ","? _)) ((_ "," _)*)? {v}

        // enforce edn prefix/name?
        rule symbol() -> ExprType
            = quiet!{ i:$(['a'..='z' | 'A'..= 'Z' | '0'..='9' |
                           '!' | '$' | '%' | '&' | '*' | '+' | '-' | '.' |
                           '/' | ':' | '<' | '?' | '=' | '>' | '@' | '^' | '_']+)
                      { ExprType::Symbol(i.to_string()) } } / expected!("symbol")

        rule integer() -> ExprType
            = quiet!{ i:$("-"?['0'..='9']+) { ExprType::Int(i.parse().unwrap()) } } / expected!("integer")

        rule float() -> ExprType
            = quiet!{ i:$("-"?['0'..='9']+ "." !"." ['0'..='9']*) { ExprType::Float(i.parse().unwrap()) } } / expected!("float")

        rule string() -> ExprType
        //= quiet!{ "\"" s:quoted() "\"" { ExprType::String(s.to_string()) } } / expected!("string")
            = "\"" s:quoted() "\"" { ExprType::String(s.to_string()) }

        //rule hex() = ['0'..='9' | 'a'..='f' | 'A'..='F']

        // read until unescaped quote
        rule quoted() -> String = s:$("\\" [_] / [^ '"'])* { s.join("") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> String {
        parse::exprs(s)
            .unwrap()
            .iter()
            .map(|a| a.to_string())
            .collect::<String>()
    }

    #[test]
    fn test1() {
        let s = (
            r#"   ( ( ) "\\\\" '1, "2 is \"two\"" 3.14 test? ([{ :yes true }]) ) ;x"#,
            r#"(() "\\\\" '1 "2 is \"two\"" 3.14 test? ([{:yes true}]))"#,
        );

        assert_eq!(&parse(s.0), s.1);
    }
}
