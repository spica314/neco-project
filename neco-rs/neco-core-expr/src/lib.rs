use std::str::FromStr;

/// コンパイラ内部で使う表現
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreExpr {
    /// e.g. `tag`
    Atom(String),
    /// e.g. `(tag arg1 arg2)`
    Node(Vec<CoreExpr>),
}

impl CoreExpr {
    pub fn node_tag(&self) -> Option<&str> {
        match self {
            CoreExpr::Atom(_) => None,
            CoreExpr::Node(xs) => {
                if xs.len() >= 1 {
                    match &xs[0] {
                        CoreExpr::Atom(x) => Some(&x),
                        CoreExpr::Node(_) => None,
                    }
                } else {
                    None
                }
            }
        }
    }
    pub fn list(&self) -> Option<&[CoreExpr]> {
        match self {
            CoreExpr::Atom(_) => None,
            CoreExpr::Node(xs) => Some(&xs),
        }
    }
    pub fn tail(&self) -> Option<&[CoreExpr]> {
        match self {
            CoreExpr::Atom(_) => None,
            CoreExpr::Node(xs) => Some(&xs[1..]),
        }
    }
    pub fn is_atom_of(&self, s: &str) -> bool {
        match self {
            CoreExpr::Atom(atom) => atom == s,
            CoreExpr::Node(_) => false,
        }
    }
    pub fn is_atom(&self) -> bool {
        matches!(self, CoreExpr::Atom(_))
    }
}

impl FromStr for CoreExpr {
    type Err = ();

    fn from_str(s: &str) -> Result<CoreExpr, ()> {
        let cs: Vec<_> = s.chars().collect();
        let mut i = 0;
        let res = parse_core_expr(&cs, &mut i)?;
        while i < cs.len() && cs[i].is_whitespace() {
            i += 1;
        }
        if i != cs.len() {
            eprintln!(
                "err: i = {}, rem = {:?}",
                i,
                &cs[i..std::cmp::min(i + 20, cs.len())]
            );
            return Err(());
        }
        Ok(res)
    }
}

impl ToString for CoreExpr {
    fn to_string(&self) -> String {
        match self {
            CoreExpr::Atom(s) => s.clone(),
            CoreExpr::Node(xs) => {
                let mut res = String::new();
                res.push('(');
                if xs.len() >= 1 {
                    res.push_str(&xs[0].to_string());
                }
                for x in xs.iter().skip(1) {
                    res.push(' ');
                    res.push_str(&x.to_string());
                }
                res.push(')');
                res
            }
        }
    }
}

fn skip_whitespaces(cs: &[char], i: &mut usize) {
    while *i < cs.len() && cs[*i].is_whitespace() {
        *i += 1;
    }
}

fn is_atom_char(c: char) -> bool {
    is_ident_char(c) || is_single_symbol_char(c) || is_multi_symbol_char(c)
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_$.".contains(c)
}

fn is_single_symbol_char(c: char) -> bool {
    ",:".contains(c)
}

fn is_multi_symbol_char(c: char) -> bool {
    "-=>".contains(c)
}

fn parse_atom(cs: &[char], i: &mut usize) -> Result<String, ()> {
    let mut k = *i;
    if k >= cs.len() {
        return Err(());
    }
    assert!(k < cs.len());
    let mut res = String::new();
    if is_ident_char(cs[k]) {
        while k < cs.len() && is_ident_char(cs[k]) {
            res.push(cs[k]);
            k += 1;
        }
    } else if is_single_symbol_char(cs[k]) {
        res.push(cs[k]);
        k += 1;
    } else if is_multi_symbol_char(cs[k]) {
        while k < cs.len() && is_multi_symbol_char(cs[k]) {
            res.push(cs[k]);
            k += 1;
        }
    }
    *i = k;
    Ok(res)
}

fn parse_node(cs: &[char], i: &mut usize) -> Result<CoreExpr, ()> {
    let mut k = *i;
    if k >= cs.len() {
        return Err(());
    }

    // skip '('
    if cs[k] != '(' {
        return Err(());
    }
    k += 1;

    // parse children
    let mut children = vec![];
    loop {
        skip_whitespaces(cs, &mut k);
        if k < cs.len() && cs[k] == ')' {
            k += 1;
            break;
        }
        let x = parse_core_expr(cs, &mut k)?;
        children.push(x);
    }

    *i = k;
    Ok(CoreExpr::Node(children))
}

fn parse_core_expr(cs: &[char], i: &mut usize) -> Result<CoreExpr, ()> {
    let mut k = *i;

    skip_whitespaces(cs, &mut k);
    while k < cs.len() && cs[k] == '\'' {
        k += 1;
    }
    skip_whitespaces(cs, &mut k);

    if k < cs.len() {
        if cs[k] == '(' {
            let res = parse_node(cs, &mut k)?;
            *i = k;
            return Ok(res);
        } else if is_atom_char(cs[k]) {
            let res = parse_atom(cs, &mut k)?;
            *i = k;
            return Ok(CoreExpr::Atom(res));
        }
    }

    Err(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str_1() {
        let s = "(tag1)";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node(vec![CoreExpr::Atom("tag1".to_string())]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_from_str_2() {
        let s = "(tag1 arg1)";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node(vec![
            CoreExpr::Atom("tag1".to_string()),
            CoreExpr::Atom("arg1".to_string()),
        ]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_from_str_3() {
        let s = "(tag1 arg1 (tag2 arg2))";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node(vec![
            CoreExpr::Atom("tag1".to_string()),
            CoreExpr::Atom("arg1".to_string()),
            CoreExpr::Node(vec![
                CoreExpr::Atom("tag2".to_string()),
                CoreExpr::Atom("arg2".to_string()),
            ]),
        ]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_from_str_4() {
        let s = "(x : Nat)";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node(vec![
            CoreExpr::Atom("x".to_string()),
            CoreExpr::Atom(":".to_string()),
            CoreExpr::Atom("Nat".to_string()),
        ]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_to_string_1() {
        let expr = CoreExpr::Node(vec![CoreExpr::Atom("tag1".to_string())]);
        let s = expr.to_string();
        let expect = "(tag1)";
        assert_eq!(s, expect);
    }

    #[test]
    fn test_to_string_2() {
        let expr = CoreExpr::Node(vec![
            CoreExpr::Atom("tag1".to_string()),
            CoreExpr::Atom("arg1".to_string()),
        ]);
        let s = expr.to_string();
        let expect = "(tag1 arg1)";
        assert_eq!(s, expect);
    }

    #[test]
    fn test_to_string_3() {
        let expr = CoreExpr::Node(vec![
            CoreExpr::Atom("tag1".to_string()),
            CoreExpr::Atom("arg1".to_string()),
            CoreExpr::Node(vec![
                CoreExpr::Atom("tag2".to_string()),
                CoreExpr::Atom("arg2".to_string()),
            ]),
        ]);
        let s = expr.to_string();
        let expect = "(tag1 arg1 (tag2 arg2))";
        assert_eq!(s, expect);
    }

    #[test]
    fn test_to_string_4() {
        let expr = CoreExpr::Node(vec![
            CoreExpr::Atom("x".to_string()),
            CoreExpr::Atom(":".to_string()),
            CoreExpr::Atom("Nat".to_string()),
        ]);
        let s = expr.to_string();
        let expect = "(x : Nat)";
        assert_eq!(s, expect);
    }
}
