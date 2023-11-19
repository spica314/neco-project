use crate::{
    decoration::{Decoration, UD},
    parse::Parse,
    syn_statement::SynStatement,
    to_felis_string::ToFelisString,
    token::{Token, TokenLBrace, TokenRBrace},
    SynTreeId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynExprBlock<D: Decoration> {
    id: SynTreeId,
    pub lbrace: TokenLBrace,
    pub statements: Vec<SynStatement<D>>,
    pub rbrace: TokenRBrace,
    ext: D::ExprBlock,
}

impl<D: Decoration> SynExprBlock<D> {
    pub fn syn_tree_id(&self) -> SynTreeId {
        self.id
    }
}

impl Parse for SynExprBlock<UD> {
    fn parse(tokens: &[Token], i: &mut usize) -> Result<Option<Self>, ()> {
        let mut k = *i;

        let Some(lbrace) = TokenLBrace::parse(tokens, &mut k)? else {
            return Ok(None);
        };

        let mut statements = vec![];
        while let Some(statement) = SynStatement::parse(tokens, &mut k)? {
            statements.push(statement);
        }

        let Some(rbrace) = TokenRBrace::parse(tokens, &mut k)? else {
            return Err(());
        };

        *i = k;
        Ok(Some(SynExprBlock {
            id: Default::default(),
            lbrace,
            statements,
            rbrace,
            ext: (),
        }))
    }
}

impl<D: Decoration> ToFelisString for SynExprBlock<D> {
    fn to_felis_string(&self) -> String {
        let mut res = String::new();
        res.push('{');
        for statement in self.statements.iter() {
            res.push(' ');
            res.push_str(&statement.to_felis_string());
        }
        res.push(' ');
        res.push('}');
        res
    }
}
