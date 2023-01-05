use std::str::FromStr;

/// コンパイラ内部で使う表現
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreExpr {
    /// e.g. `tag`
    Atom(String),
    /// e.g. `(tag arg1 arg2)`
    Node(String, Vec<CoreExpr>),
}

impl FromStr for CoreExpr {
    type Err = ();

    fn from_str(s: &str) -> Result<CoreExpr, ()> {
        let cs: Vec<_> = s.chars().collect();
        let mut i = 0;
        let res = parse_core_expr(&cs, &mut i)?;
        if i != cs.len() {
            eprintln!("err: i = {}", i);
            return Err(());
        }
        Ok(res)
    }
}

impl ToString for CoreExpr {
    fn to_string(&self) -> String {
        match self {
            CoreExpr::Atom(s) => s.clone(),
            CoreExpr::Node(tag, args) => {
                let mut res = String::new();
                res.push('(');
                res.push_str(tag);
                for arg in args {
                    res.push(' ');
                    res.push_str(&arg.to_string());
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
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

fn parse_atom(cs: &[char], i: &mut usize) -> Result<String, ()> {
    let mut k = *i;
    if k >= cs.len() {
        return Err(());
    }
    let mut res = String::new();
    while k < cs.len() && is_atom_char(cs[k]) {
        res.push(cs[k]);
        k += 1;
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

    // parse tag
    skip_whitespaces(cs, &mut k);
    let tag = parse_atom(cs, &mut k)?;
    eprintln!("tag = {}", tag);

    // parse args
    let mut args = vec![];
    loop {
        skip_whitespaces(cs, &mut k);
        if k < cs.len() && cs[k] == ')' {
            k += 1;
            break;
        }
        let arg = parse_core_expr(cs, &mut k)?;
        args.push(arg);
    }

    *i = k;
    Ok(CoreExpr::Node(tag, args))
}

fn parse_core_expr(cs: &[char], i: &mut usize) -> Result<CoreExpr, ()> {
    let mut k = *i;

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
        let expect = CoreExpr::Node("tag1".to_string(), vec![]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_from_str_2() {
        let s = "(tag1 arg1)";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node("tag1".to_string(), vec![CoreExpr::Atom("arg1".to_string())]);
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_from_str_3() {
        let s = "(tag1 arg1 (tag2 arg2))";
        let expr = CoreExpr::from_str(s).unwrap();
        let expect = CoreExpr::Node(
            "tag1".to_string(),
            vec![
                CoreExpr::Atom("arg1".to_string()),
                CoreExpr::Node("tag2".to_string(), vec![CoreExpr::Atom("arg2".to_string())]),
            ],
        );
        assert_eq!(expr, expect);
    }

    #[test]
    fn test_to_string_1() {
        let expr = CoreExpr::Node("tag1".to_string(), vec![]);
        let s = expr.to_string();
        let expect = "(tag1)";
        assert_eq!(s, expect);
    }

    #[test]
    fn test_to_string_2() {
        let expr = CoreExpr::Node("tag1".to_string(), vec![CoreExpr::Atom("arg1".to_string())]);
        let s = expr.to_string();
        let expect = "(tag1 arg1)";
        assert_eq!(s, expect);
    }

    #[test]
    fn test_to_string_3() {
        let expr = CoreExpr::Node(
            "tag1".to_string(),
            vec![
                CoreExpr::Atom("arg1".to_string()),
                CoreExpr::Node("tag2".to_string(), vec![CoreExpr::Atom("arg2".to_string())]),
            ],
        );
        let s = expr.to_string();
        let expect = "(tag1 arg1 (tag2 arg2))";
        assert_eq!(s, expect);
    }
}
