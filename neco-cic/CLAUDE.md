# Claude Development Guidelines for neco-cic

This file contains guidelines and context for Claude when working on the neco-cic project.

## Project Overview

neco-cic is an implementation of the Calculus of Inductive Constructions (CIC) in Rust. It includes:

- Core term structure with dependent types
- Type checker implementing CIC typing rules
- Reduction engine with β, ζ, and ι reduction
- Inductive types with constructors and pattern matching
- Global and local environments for definitions

## Development Guidelines

### Testing Guidelines

**Rule: One assertion per test function**

Each test function should test exactly one specific behavior or property. This makes tests:
- Easier to understand and maintain
- Provide clearer error messages when they fail
- Allow for better test organization and documentation

**Good Example:**
```rust
#[test]
fn test_set_has_type_type0() {
    let set = Term::Sort(TermSort { sort: Sort::Set });
    let ty = infer_type(&ctx, &env, &set).unwrap();
    assert_eq!(*ty, Term::Sort(TermSort { sort: Sort::Type(0) }));
}

#[test] 
fn test_type0_has_type_type1() {
    let type0 = Term::Sort(TermSort { sort: Sort::Type(0) });
    let ty = infer_type(&ctx, &env, &type0).unwrap();
    assert_eq!(*ty, Term::Sort(TermSort { sort: Sort::Type(1) }));
}
```

**Bad Example:**
```rust
#[test]
fn test_sort_typing() {
    // Test: Set : Type(0)
    let set = Term::Sort(TermSort { sort: Sort::Set });
    let ty = infer_type(&ctx, &env, &set).unwrap();
    assert_eq!(*ty, Term::Sort(TermSort { sort: Sort::Type(0) }));

    // Test: Type(0) : Type(1) 
    let type0 = Term::Sort(TermSort { sort: Sort::Type(0) });
    let ty = infer_type(&ctx, &env, &type0).unwrap();
    assert_eq!(*ty, Term::Sort(TermSort { sort: Sort::Type(1) }));
}
```

### Code Organization

- Keep modules focused and cohesive
- Use clear, descriptive names for functions and types
- Document public APIs with doc comments
- Implement error types with clear, helpful messages

### Type Checking Implementation

- Follow CIC typing rules precisely
- Handle variable capture correctly (unique IDs help with this)
- Implement proper conversion checking
- Use WHNF reduction where appropriate for type checking

### Testing Strategy

- Test each major component separately
- Include both positive and negative test cases
- Test edge cases and error conditions
- Use descriptive test names that explain what is being tested

## Architecture Notes

### Variable Handling

The implementation uses unique IDs for variables, which eliminates variable capture issues in substitution. Each binding (lambda, product, let-in, case branches) gets a fresh unique ID.

### Reduction Strategy

- `reduce_step`: performs one step of reduction
- `normalize`: reduces to normal form
- `whnf`: reduces to weak head normal form (used in type checking)

### Inductive Types

Inductive types are defined in the global environment and support:
- Parameterized types (like `List A`)
- Multiple constructors with different arities
- Pattern matching with case expressions
- Proper ι-reduction for case expressions

## Current Implementation Status

✅ Core term structure
✅ Substitution with unique ID handling
✅ β-reduction and ζ-reduction
✅ Basic type checker for CIC
✅ Inductive types with constructors
✅ Case expressions with ι-reduction
✅ Global and local environments
✅ Comprehensive test suite

## Future Improvements

- Better error messages with source locations
- Module system support
- Universe polymorphism
- Proof irrelevance for Prop
- Guard checking for recursive definitions
- Parser and pretty printer
- More sophisticated conversion checking