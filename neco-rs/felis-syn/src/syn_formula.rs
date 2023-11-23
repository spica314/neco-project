use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    to_felis_string::ToFelisString,
    token::{
        Token, TokenArrow, TokenCamma, TokenColon, TokenIdent, TokenKeyword, TokenLParen,
        TokenRParen,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SynFormula<D: Decoration> {
    Forall(SynFormulaForall<D>),
    Implies(SynFormulaImplies<D>),
    Atom(SynFormulaAtom<D>),
    App(SynFormulaApp<D>),
    Paren(SynFormulaParen<D>),
}

impl<D: Decoration> ToFelisString for SynFormula<D> {
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

impl<D: Decoration> From<SynFormulaNoArg<D>> for SynFormula<D> {
    fn from(x: SynFormulaNoArg<D>) -> Self {
        match x {
            SynFormulaNoArg::Atom(atom) => SynFormula::Atom(atom),
            SynFormulaNoArg::App(app) => SynFormula::App(app),
            SynFormulaNoArg::Paren(paren) => SynFormula::Paren(paren),
        }
    }
}

impl Parse for SynFormula<UD> {
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

        let mut res: SynFormula<UD> = res.into();

        while let Some(arrow) = TokenArrow::parse(tokens, &mut k)? {
            let rhs = SynFormula::parse(tokens, &mut k)?;
            let Some(rhs) = rhs else {
                return Err(());
            };
            res = SynFormula::Implies(SynFormulaImplies {
                lhs: Box::new(res),
                arrow,
                rhs: Box::new(rhs),
                ext: (),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SynFormulaAtomOrParen<D: Decoration> {
    Atom(SynFormulaAtom<D>),
    Paren(SynFormulaParen<D>),
}

impl Parse for SynFormulaAtomOrParen<UD> {
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
enum SynFormulaNoArg<D: Decoration> {
    Atom(SynFormulaAtom<D>),
    App(SynFormulaApp<D>),
    Paren(SynFormulaParen<D>),
}

impl<D: Decoration> From<SynFormulaAtomOrParen<D>> for SynFormulaNoArg<D> {
    fn from(x: SynFormulaAtomOrParen<D>) -> Self {
        match x {
            SynFormulaAtomOrParen::Atom(atom) => SynFormulaNoArg::Atom(atom),
            SynFormulaAtomOrParen::Paren(paren) => SynFormulaNoArg::Paren(paren),
        }
    }
}

impl Parse for SynFormulaNoArg<UD> {
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
        let mut res: SynFormulaNoArg<UD> = res.into();

        for x in xs.into_iter().skip(1) {
            let x: SynFormulaNoArg<UD> = x.into();
            let x: SynFormula<UD> = x.into();
            res = SynFormulaNoArg::App(SynFormulaApp {
                fun: Box::new(res.into()),
                arg: Box::new(x),
                ext: (),
            });
        }

        *i = k;
        Ok(Some(res))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaForall<D: Decoration> {
    pub keyword_forall: TokenKeyword,
    pub lparen: TokenLParen,
    pub name: TokenIdent,
    pub colon: TokenColon,
    pub ty: Box<SynFormula<D>>,
    pub rparen: TokenRParen,
    pub camma: TokenCamma,
    pub child: Box<SynFormula<D>>,
    pub ext: D::FormulaForall,
}

impl<D: Decoration> ToFelisString for SynFormulaForall<D> {
    fn to_felis_string(&self) -> String {
        format!(
            "#forall ({} : {}), {}",
            self.name.as_str(),
            self.ty.to_felis_string(),
            self.child.to_felis_string()
        )
    }
}

impl Parse for SynFormulaForall<UD> {
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
            ext: (),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaImplies<D: Decoration> {
    pub lhs: Box<SynFormula<D>>,
    pub arrow: TokenArrow,
    pub rhs: Box<SynFormula<D>>,
    pub ext: D::FormulaImplies,
}

impl<D: Decoration> ToFelisString for SynFormulaImplies<D> {
    fn to_felis_string(&self) -> String {
        format!(
            "{} -> {}",
            self.lhs.to_felis_string(),
            self.rhs.to_felis_string()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaAtom<D: Decoration> {
    pub ident: TokenIdent,
    pub ext: D::FormulaAtom,
}

impl<D: Decoration> ToFelisString for SynFormulaAtom<D> {
    fn to_felis_string(&self) -> String {
        self.ident.as_str().to_string()
    }
}

impl Parse for SynFormulaAtom<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(ident) = TokenIdent::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        *i = k;
        Ok(Some(SynFormulaAtom { ident, ext: () }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaApp<D: Decoration> {
    pub fun: Box<SynFormula<D>>,
    pub arg: Box<SynFormula<D>>,
    pub ext: D::FormulaApp,
}

impl<D: Decoration> ToFelisString for SynFormulaApp<D> {
    fn to_felis_string(&self) -> String {
        format!(
            "{} {}",
            self.fun.to_felis_string(),
            self.arg.to_felis_string()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynFormulaParen<D: Decoration> {
    pub lparen: TokenLParen,
    pub child: Box<SynFormula<D>>,
    pub rparen: TokenRParen,
    pub ext: D::FormulaParen,
}

impl<D: Decoration> ToFelisString for SynFormulaParen<D> {
    fn to_felis_string(&self) -> String {
        format!("({})", self.child.to_felis_string())
    }
}

impl Parse for SynFormulaParen<UD> {
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
            ext: (),
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{decoration::UD, test_utils::parse_from_str, to_felis_string::ToFelisString};

    use super::SynFormula;

    #[test]
    fn test_felis_syn_formula_1() {
        let s = "#forall (A : Prop), #forall (B : Prop), And A B -> Or A B";
        let res = parse_from_str::<SynFormula<UD>>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_2() {
        let s = "And A B -> Or A B";
        let res = parse_from_str::<SynFormula<UD>>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_3() {
        let s = "Or A B";
        let res = parse_from_str::<SynFormula<UD>>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }

    #[test]
    fn test_felis_syn_formula_4() {
        let s = "A";
        let res = parse_from_str::<SynFormula<UD>>(s).unwrap();
        assert_eq!(res.unwrap().to_felis_string(), s);
    }
}
