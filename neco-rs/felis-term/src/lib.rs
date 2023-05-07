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
}

impl IsTerm for Term {
    fn level(&self) -> usize {
        match self {
            Term::Atom(atom) => atom.level(),
            Term::Star(star) => star.level(),
            Term::Map(map) => map.level(),
            Term::DependentMap(dep_map) => dep_map.level(),
            Term::App(app) => app.level(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermAtom {
    level: usize,
    id: SerialId,
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
