use std::collections::{HashMap, HashSet};

use crate::{type_term::TypeTerm, TypeId, VarId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeChecker {
    types: HashMap<TypeId, TypeTerm>,
    vars: HashMap<VarId, TypeTerm>,
    relations: HashSet<TypeRelation>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            vars: HashMap::new(),
            relations: HashSet::new(),
        }
    }

    pub fn add_type(&mut self, ty_ty: TypeTerm) -> TypeId {
        let id = TypeId(self.types.len());
        self.types.insert(id, ty_ty);
        id
    }

    pub fn add_var(&mut self, ty: TypeTerm) -> VarId {
        let id = VarId(self.vars.len());
        self.vars.insert(id, ty);
        id
    }

    pub fn add_relation(&mut self, s: TypeTerm, t: TypeTerm) -> bool {
        self.relations.insert(TypeRelation { s, t })
    }

    pub fn expand(&self, t: &TypeTerm) -> TypeTerm {
        match t {
            TypeTerm::Base(base) => TypeTerm::Base(*base),
            TypeTerm::Var(var) => self.expand(self.vars.get(var).unwrap()),
            TypeTerm::App(s, t) => {
                let s = self.expand(s);
                let t = self.expand(t);
                TypeTerm::App(Box::new(s), Box::new(t))
            }
            TypeTerm::Arrow(s, t) => {
                let s = self.expand(s);
                let t = self.expand(t);
                TypeTerm::Arrow(Box::new(s), Box::new(t))
            }
            TypeTerm::Star(level) => TypeTerm::Star(level.clone()),
            TypeTerm::Candidates(_) => todo!(),
            TypeTerm::Unknown => TypeTerm::Unknown,
        }
    }

    pub fn get_all(&self) -> HashMap<VarId, TypeTerm> {
        let mut map = HashMap::new();
        for (id, ty) in &self.vars {
            map.insert(*id, self.expand(ty));
        }
        map
    }

    pub fn resolve(&mut self) {
        loop {
            let mut changed = false;
            let mut add_rel = vec![];
            for relation in self.relations.iter() {
                match (&relation.s, &relation.t) {
                    (TypeTerm::Var(s), TypeTerm::Var(t)) => {
                        if self.vars.get(s).unwrap().is_unknown()
                            && !self.vars.get(t).unwrap().is_unknown()
                        {
                            self.vars.insert(*s, self.vars.get(t).unwrap().clone());
                            changed = true;
                        }
                        if !self.vars.get(s).unwrap().is_unknown()
                            && self.vars.get(t).unwrap().is_unknown()
                        {
                            self.vars.insert(*t, self.vars.get(s).unwrap().clone());
                            changed = true;
                        }
                    }
                    (TypeTerm::Var(s), TypeTerm::Base(t)) => {
                        if self.vars.get(s).unwrap().is_unknown() {
                            self.vars.insert(*s, TypeTerm::Base(*t));
                            changed = true;
                        }
                    }
                    (TypeTerm::Base(s), TypeTerm::Var(t)) => {
                        if self.vars.get(t).unwrap().is_unknown() {
                            self.vars.insert(*t, TypeTerm::Base(*s));
                            changed = true;
                        }
                    }
                    (TypeTerm::Var(s), TypeTerm::App(t1, t2)) => {
                        let t1 = self.expand(t1);
                        if let TypeTerm::Arrow(t11, t12) = t1 {
                            add_rel.push((*t11.clone(), *t2.clone()));
                            add_rel.push((TypeTerm::Var(*s), *t12.clone()));
                        }
                    }
                    (TypeTerm::App(s1, s2), TypeTerm::Var(t)) => {
                        let s1 = self.expand(s1);
                        if let TypeTerm::Arrow(s11, s12) = s1 {
                            add_rel.push((*s11.clone(), *s2.clone()));
                            add_rel.push((TypeTerm::Var(*t), *s12.clone()));
                        }
                    }
                    _ => {}
                }
            }
            for (s, t) in add_rel {
                if self.add_relation(s, t) {
                    changed = true;
                }
            }
            eprintln!("---rel ---");
            for rel in &self.relations {
                eprintln!("{:?}", rel);
            }
            eprintln!();
            if !changed {
                break;
            }
        }
    }

    pub fn get(&self, id: VarId) -> Option<&TypeTerm> {
        self.vars.get(&id)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeRelation {
    s: TypeTerm,
    t: TypeTerm,
}

#[cfg(test)]
mod test {
    use crate::type_term::TypeLevel;

    use super::*;

    #[test]
    fn test_1() {
        let mut type_checker = TypeChecker::new();
        let str_ty_id = type_checker.add_type(TypeTerm::Star(TypeLevel::new(2)));
        let unit_ty_id = type_checker.add_type(TypeTerm::Star(TypeLevel::new(2)));

        let v1 = type_checker.add_var(TypeTerm::Unknown);
        let v2 = type_checker.add_var(TypeTerm::Base(str_ty_id));
        let v3 = type_checker.add_var(TypeTerm::Arrow(
            Box::new(TypeTerm::Base(str_ty_id)),
            Box::new(TypeTerm::Base(unit_ty_id)),
        ));
        let v4 = type_checker.add_var(TypeTerm::Unknown);

        type_checker.add_relation(TypeTerm::Var(v1), TypeTerm::Var(v2));
        type_checker.add_relation(
            TypeTerm::App(Box::new(TypeTerm::Var(v3)), Box::new(TypeTerm::Var(v1))),
            TypeTerm::Var(v4),
        );

        type_checker.resolve();

        assert_eq!(type_checker.get(v1), Some(&TypeTerm::Base(str_ty_id)));
        assert_eq!(type_checker.get(v2), Some(&TypeTerm::Base(str_ty_id)));
        assert_eq!(
            type_checker.get(v3),
            Some(&TypeTerm::Arrow(
                Box::new(TypeTerm::Base(str_ty_id)),
                Box::new(TypeTerm::Base(unit_ty_id))
            ))
        );
        assert_eq!(type_checker.get(v4), Some(&TypeTerm::Base(unit_ty_id)));
    }
}
