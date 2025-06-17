pub mod global_environment;
pub mod id;
pub mod local_context;
pub mod reduction;
pub mod substitution;
pub mod term;
pub mod typechecker;

#[cfg(test)]
mod reduction_test;
#[cfg(test)]
mod typechecker_test;
