use felis_rename::SerialId;

pub trait IsTerm {
    // value: 0
    // type of value: 1
    // type of (type of value): 2
    // ...
    fn level(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Atom(TermAtom),
    Star(TermStar),
    Map(TermMap),
    DependentMap(TermDependentMap),
    App(TermApp),
    Match(TermMatch),
}

impl Term {
    pub fn final_return_type(&self) -> Term {
        match self {
            Term::Map(map) => map.to.final_return_type().clone(),
            Term::DependentMap(dep_map) => dep_map.to.final_return_type().clone(),
            _ => self.clone(),
        }
    }
}

impl IsTerm for Term {
    fn level(&self) -> usize {
        match self {
            Term::Atom(atom) => atom.level(),
            Term::Star(star) => star.level(),
            Term::Map(map) => map.level(),
            Term::DependentMap(dep_map) => dep_map.level(),
            Term::App(app) => app.level(),
            Term::Match(term_match) => term_match.level(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermAtom {
    pub level: usize,
    pub id: SerialId,
}

impl TermAtom {
    pub fn new(level: usize, id: SerialId) -> TermAtom {
        TermAtom { level, id }
    }
    pub fn id(&self) -> SerialId {
        self.id
    }
}

impl From<TermAtom> for Term {
    fn from(value: TermAtom) -> Self {
        Term::Atom(value)
    }
}

impl IsTerm for TermAtom {
    fn level(&self) -> usize {
        self.level
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermStar {
    level: usize,
}

impl IsTerm for TermStar {
    fn level(&self) -> usize {
        self.level
    }
}

impl TermStar {
    pub fn new(level: usize) -> TermStar {
        TermStar { level }
    }
}

impl From<TermStar> for Term {
    fn from(value: TermStar) -> Self {
        Term::Star(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermMap {
    pub from: Box<Term>,
    pub to: Box<Term>,
}

impl IsTerm for TermMap {
    fn level(&self) -> usize {
        self.to.level()
    }
}

impl From<TermMap> for Term {
    fn from(value: TermMap) -> Self {
        Term::Map(value)
    }
}

impl TermMap {
    pub fn new(from: Term, to: Term) -> TermMap {
        TermMap {
            from: Box::new(from),
            to: Box::new(to),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermDependentMap {
    pub from: (TermAtom, Box<Term>),
    pub to: Box<Term>,
}

impl IsTerm for TermDependentMap {
    fn level(&self) -> usize {
        self.to.level()
    }
}

impl From<TermDependentMap> for Term {
    fn from(value: TermDependentMap) -> Self {
        Term::DependentMap(value)
    }
}

impl TermDependentMap {
    pub fn new(from: (TermAtom, Term), to: Term) -> TermDependentMap {
        TermDependentMap {
            from: (from.0, Box::new(from.1)),
            to: Box::new(to),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermApp {
    pub fun: Box<Term>,
    pub arg: Box<Term>,
}

impl TermApp {
    pub fn new(fun: Term, arg: Term) -> TermApp {
        TermApp {
            fun: Box::new(fun),
            arg: Box::new(arg),
        }
    }
}

impl IsTerm for TermApp {
    fn level(&self) -> usize {
        self.fun.level()
    }
}

impl From<TermApp> for Term {
    fn from(value: TermApp) -> Self {
        Term::App(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedTerm {
    pub term: Term,
    pub ty: Term,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermMatch {
    pub expr: Box<Term>,
    pub arms: Vec<(SerialId, Vec<SerialId>, Term)>,
}

impl IsTerm for TermMatch {
    fn level(&self) -> usize {
        self.arms[0].2.level()
    }
}

impl TermMatch {}

pub fn remap_term(remap: &(SerialId, Term), term: &Term) -> Term {
    match term {
        Term::Atom(atom) => {
            if atom.id() == remap.0 {
                remap.1.clone()
            } else {
                Term::Atom(atom.clone())
            }
        }
        Term::Star(star) => Term::Star(star.clone()),
        Term::Map(map) => {
            let from = remap_term(remap, &map.from);
            let to = remap_term(remap, &map.to);
            let t = TermMap {
                from: Box::new(from),
                to: Box::new(to),
            };
            Term::Map(t)
        }
        Term::DependentMap(dep_map) => {
            let from_atom = &dep_map.from.0;
            let from_ty = remap_term(remap, &dep_map.from.1);
            let to = remap_term(remap, &dep_map.to);
            let t = TermDependentMap {
                from: (from_atom.clone(), Box::new(from_ty)),
                to: Box::new(to),
            };
            Term::DependentMap(t)
        }
        Term::App(app) => {
            let fun = remap_term(remap, &app.fun);
            let arg = remap_term(remap, &app.arg);
            let t = TermApp {
                fun: Box::new(fun),
                arg: Box::new(arg),
            };
            Term::App(t)
        }
        Term::Match(_term_match) => {
            panic!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputeAppedTypedTermError {
    pub expected_ty: Term,
    pub actual_ty: Term,
}

pub fn compute_apped_typed_term(
    fun: &TypedTerm,
    arg: &TypedTerm,
) -> Result<TypedTerm, ComputeAppedTypedTermError> {
    match &fun.ty {
        Term::Map(map) => {
            let from_ty = &map.from;
            if from_ty.as_ref() != &arg.ty {
                return Err(ComputeAppedTypedTermError {
                    expected_ty: from_ty.as_ref().clone(),
                    actual_ty: arg.ty.clone(),
                });
            }
            let to_ty = &map.to;
            let app = Term::App(TermApp {
                fun: Box::new(fun.term.clone()),
                arg: Box::new(arg.term.clone()),
            });
            Ok(TypedTerm {
                term: app,
                ty: to_ty.as_ref().clone(),
            })
        }
        Term::DependentMap(dep_map) => {
            let from_ty = &dep_map.from.1;
            if from_ty.as_ref() != &arg.ty {
                return Err(ComputeAppedTypedTermError {
                    expected_ty: from_ty.as_ref().clone(),
                    actual_ty: arg.ty.clone(),
                });
            }
            let to_ty = remap_term(&(dep_map.from.0.id(), arg.term.clone()), &dep_map.to);
            let app = Term::App(TermApp {
                fun: Box::new(fun.term.clone()),
                arg: Box::new(arg.term.clone()),
            });
            Ok(TypedTerm {
                term: app,
                ty: to_ty,
            })
        }
        _ => panic!(),
    }
}
