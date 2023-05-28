use crate::{
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{
        Token, TokenArrow, TokenCamma, TokenColon, TokenIdent, TokenKeyword, TokenLParen,
        TokenRParen,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynFormula {
    Forall(SynFormulaForall),
    Implies(SynFormulaImplies),
    Atom(SynFormulaAtom),
    App(SynFormulaApp),
    Paren(SynFormulaParen),
}

impl ToFelisString for SynFormula {
    fn to_felis_string(&self) -> String {
        match self {
            SynFormula::Forall(forall) => forall.to_felis_string(),
            SynFormula::Implies(implies) => implies.to_felis_string(),
            SynFormula::Atom(atom) => atom.to_felis_string(),
            SynFormula::App(app) => app.to_felis_string(),
            SynFormula::Paren(paren) => paren.to_felis_string(),
        }
    }
}

impl From<SynFormulaNoArg> for SynFormula {
    fn from(x: SynFormulaNoArg) -> Self {
        match x {
            SynFormulaNoArg::Atom(atom) => SynFormula::Atom(atom),
            SynFormulaNoArg::App(app) => SynFormula::App(app),
            SynFormulaNoArg::Paren(paren) => SynFormula::Paren(paren),
        }
    }
}

impl Parse for SynFormula {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        if let Some(forall) = SynFormulaForall::parse(tokens, &mut k)? {
            *i = k;
            return Ok(Some(SynFormula::Forall(forall)));
        }

        let res = SynFormulaNoArg::parse(tokens, &mut k)?;
        let Some(res) = res else {
            return Err(());
        };

        let mut res: SynFormula = res.into();

        while let Some(arrow) = TokenArrow::parse(tokens, &mut k)? {
            let rhs = SynFormula::parse(tokens, &mut k)?;
            let Some(rhs) = rhs else {
                return Err(());
            };
            res = SynFormula::Implies(SynFormulaImplies {
                lhs: Box::new(res),
                arrow,
                rhs: Box::new(rhs),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynFormulaAtomOrParen {
    Atom(SynFormulaAtom),
    Paren(SynFormulaParen),
}

impl Parse for SynFormulaAtomOrParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        if let Some(paren) = SynFormulaParen::parse(tokens, i)? {
            return Ok(Some(SynFormulaAtomOrParen::Paren(paren)));
        }

        if let Some(atom) = SynFormulaAtom::parse(tokens, i)? {
            return Ok(Some(SynFormulaAtomOrParen::Atom(atom)));
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynFormulaNoArg {
    Atom(SynFormulaAtom),
    App(SynFormulaApp),
    Paren(SynFormulaParen),
}

impl From<SynFormulaAtomOrParen> for SynFormulaNoArg {
    fn from(x: SynFormulaAtomOrParen) -> Self {
        match x {
            SynFormulaAtomOrParen::Atom(atom) => SynFormulaNoArg::Atom(atom),
            SynFormulaAtomOrParen::Paren(paren) => SynFormulaNoArg::Paren(paren),
        }
    }
}

impl Parse for SynFormulaNoArg {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let mut xs = vec![];

        while let Some(x) = SynFormulaAtomOrParen::parse(tokens, &mut k)? {
            xs.push(x);
        }

        if xs.is_empty() {
            return Ok(None);
        }

        let res = xs[0].clone();
        let mut res: SynFormulaNoArg = res.into();

        for x in xs.into_iter().skip(1) {
            let x: SynFormulaNoArg = x.into();
            let x: SynFormula = x.into();
            res = SynFormulaNoArg::App(SynFormulaApp {
                fun: Box::new(res.into()),
                arg: Box::new(x),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaForall {
    pub keyword_forall: TokenKeyword,
    pub lparen: TokenLParen,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: Box<SynFormula>,
    pub rparen: TokenRParen,
    pub camma: TokenCamma,
    pub child: Box<SynFormula>,
}

impl ToFelisString for SynFormulaForall {
    fn to_felis_string(&self) -> String {
        format!(
            "#forall ({} : {}), {}",
            self.name.as_str(),
            self.ty.to_felis_string(),
            self.child.to_felis_string()
        )
    }
}

impl Parse for SynFormulaForall {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(keyword_forall) = TokenKeyword::parse(tokens, &mut k)? else {
            return Ok(None);
        };
        if keyword_forall.keyword != "#forall" {
            return Ok(None);
        }

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(name) = TokenIdent::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(colon) = TokenColon::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(ty) = SynFormula::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(camma) = TokenCamma::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(child) = SynFormula::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFormulaForall {
            keyword_forall,
            lparen,
            name,
            colon,
            ty: Box::new(ty),
            rparen,
            camma,
            child: Box::new(child),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaImplies {
    pub lhs: Box<SynFormula>,
    pub arrow: TokenArrow,
    pub rhs: Box<SynFormula>,
}

impl ToFelisString for SynFormulaImplies {
    fn to_felis_string(&self) -> String {
        format!(
            "{} -> {}",
            self.lhs.to_felis_string(),
            self.rhs.to_felis_string()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaAtom {
    pub ident: TokenIdent,
}

impl ToFelisString for SynFormulaAtom {
    fn to_felis_string(&self) -> String {
        self.ident.as_str().to_string()
    }
}

impl Parse for SynFormulaAtom {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynFormulaAtom { ident }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaApp {
    pub fun: Box<SynFormula>,
    pub arg: Box<SynFormula>,
}

impl ToFelisString for SynFormulaApp {
    fn to_felis_string(&self) -> String {
        format!(
            "{} {}",
            self.fun.to_felis_string(),
            self.arg.to_felis_string()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaParen {
    pub lparen: TokenLParen,
    pub child: Box<SynFormula>,
    pub rparen: TokenRParen,
}

impl ToFelisString for SynFormulaParen {
    fn to_felis_string(&self) -> String {
        format!("({})", self.child.to_felis_string())
    }
}

impl Parse for SynFormulaParen {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lparen) = TokenLParen::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let Some(child) = SynFormula::parse(tokens, &mut k)? else {
            return Err(());
        };

        let Some(rparen) = TokenRParen::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynFormulaParen {
            lparen,
            child: Box::new(child),
            rparen,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{test_utils::parse_from_str, to_felis_string::ToFelisString};

    use super::SynFormula;

    #[test]
    fn test_felis_syn_formula_1() {
        let s = "#forall (A : Prop), #forall (B : Prop), And A B -> Or A B";
        let res = parse_from_str::<SynFormula>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_2() {
        let s = "And A B -> Or A B";
        let res = parse_from_str::<SynFormula>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_3() {
        let s = "Or A B";
        let res = parse_from_str::<SynFormula>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_4() {
        let s = "A";
        let res = parse_from_str::<SynFormula>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }
}
