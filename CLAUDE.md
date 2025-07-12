# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**neco-project** is a hobbyist compiler and programming language implementation:
- **neco**: Compiler for the Felis programming language
- **Felis**: Dependently-typed functional programming language implementing Calculus of Inductive Constructions (CIC)

## Workspace Architecture

This is a Rust workspace (edition 2024) with 6 crates forming a compiler pipeline:

1. **neco-cic** - Core CIC kernel with type theory, reduction, and type checking
2. **neco-felis-syn** - Frontend parser and syntax for Felis language
3. **neco-felis-type-check** - Bridges Felis syntax with CIC kernel
4. **neco-felis-compile** - Compiler frontend and main binary
5. **neco-felis-rename** - Variable renaming and scope resolution
6. **neco-scope** - Scope management utilities

## Common Commands

### Building and Testing
```bash
# Build entire workspace
cargo check

# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p neco-cic
cargo test -p neco-felis-syn
cargo test -p neco-felis-type-check
cargo test -p neco-felis-compile
cargo test -p neco-felis-rename
cargo test -p neco-scope

# Test with snapshots (neco-felis-syn uses insta)
cargo test -p neco-felis-syn
```

### Development
```bash
# Complete check (format, lint, test)
tools/check.sh

# Generate coverage report
tools/generate-coverage-report.sh

# Check newlines at end of files
tools/check-newlines.sh
```

## Architecture Notes

### Data Flow
Felis source → `neco-felis-syn` (parsing) → `neco-felis-rename` (scope resolution) → `neco-felis-type-check` (validation) → `neco-cic` (kernel evaluation) → `neco-felis-compile` (compilation)

### Key Design Patterns
- **Unique IDs**: Variables use unique IDs to avoid capture issues in substitution
- **Reference Counting**: Extensive use of `Rc<T>` for sharing AST nodes
- **Modular Type Theory**: Clean separation between frontend syntax and kernel implementation
- **Trees That Grow**: Extensible AST using phase-parameterized types for multi-stage compilation

### Testing Strategy
- **Unit tests**: Place tests at end of implementation files, one assertion per test function
- **Integration tests**: Use testcases in `/testcases/felis/single/` with both positive and negative cases (24 test files)
- **Snapshot testing**: `neco-felis-syn` uses `insta` for parser output validation
- **Coverage reporting**: Use `tools/generate-coverage-report.sh` to generate coverage reports with `cargo llvm-cov`
- **Test categories**: Basic CIC features, arithmetic operations (add, sub, mul, div, mod), floating-point arithmetic, arrays, structs, conditionals, let bindings, correctness variants, and failure cases

## File Structure

```
neco-cic/src/
├── term.rs              # CIC term AST
├── typechecker.rs       # Type checking rules
├── reduction.rs         # Term reduction/normalization
├── global_environment.rs # Global definitions
├── local_context.rs     # Local context management
├── inductive.rs         # Inductive type support
├── substitution.rs      # Variable substitution
└── id.rs                # Unique identifier system

neco-felis-syn/src/
├── phase.rs             # Trees That Grow phase definitions
├── token.rs             # Lexical analysis
├── parse.rs             # Parser framework
├── file.rs              # File-level parsing (phase-parameterized)
├── file_id.rs           # File identification
├── pos.rs               # Position tracking
├── item*.rs             # Top-level items (phase-parameterized)
│   ├── item.rs               # Main item types enum
│   ├── item_array.rs         # Array type definitions
│   ├── item_definition.rs    # Function definitions
│   ├── item_entrypoint.rs    # Entry point definitions
│   ├── item_inductive.rs     # Inductive type definitions
│   ├── item_inductive_branch.rs # Inductive constructor branches
│   ├── item_proc.rs          # Procedure definitions
│   ├── item_proc_block.rs    # Procedure block syntax
│   ├── item_struct.rs        # Struct type definitions
│   ├── item_theorem.rs       # Theorem definitions
│   └── item_use_builtin.rs   # Built-in imports
├── term*.rs             # Term-level syntax (phase-parameterized)
│   ├── term.rs               # Main term types enum
│   ├── term_apply.rs         # Function application
│   ├── term_arrow_dep.rs     # Dependent arrow types
│   ├── term_arrow_nodep.rs   # Non-dependent arrow types
│   ├── term_assign.rs        # Assignment operations
│   ├── term_constructor_call.rs # Constructor invocation
│   ├── term_field_access.rs  # Struct field access
│   ├── term_field_assign.rs  # Struct field assignment
│   ├── term_if.rs            # Conditional expressions
│   ├── term_let.rs           # Let bindings
│   ├── term_let_mut.rs       # Mutable let bindings
│   ├── term_match.rs         # Pattern matching
│   ├── term_match_branch.rs  # Match branches
│   ├── term_number.rs        # Numeric literals
│   ├── term_paren.rs         # Parenthesized expressions
│   ├── term_unit.rs          # Unit type/value
│   └── term_variable.rs      # Variable references
├── statements.rs        # Main statement parsing
├── statements_then.rs    # Statement sequencing
└── snapshots/           # Snapshot test files
    ├── neco_felis_syn__file__test__parse_*.snap
    └── neco_felis_syn__token__test__lex_*.snap

neco-felis-rename/src/
├── lib.rs               # Variable renaming logic
└── phase_renamed.rs     # Renamed phase definitions

neco-felis-compile/src/
├── lib.rs               # Compilation logic
├── main.rs              # Legacy main (deprecated)
└── bin/
    └── neco-felis-compile.rs  # Main compiler binary

neco-felis-type-check/src/
└── lib.rs               # Type checking bridge

neco-scope/src/
└── lib.rs               # Scope management utilities

testcases/felis/single/  # Test cases for language features (24 test files)
docs/                    # Documentation directory
└── design-principles.md # Design principles and architecture notes
```

## Language Features (Felis)

Felis supports dependent types with syntax for:
- Inductive types: `#inductive nat : Set { O: nat, S: nat -> nat }`
- Definitions: `#definition add : (n : nat) -> (m : nat) -> nat { ... }`
- Theorems: `#theorem name : type { proof }`
- Pattern matching: `#match expr { pattern => body }`
- Dependent function types: `(A : Type) -> (x : A) -> ...`
- Entry points: `#entrypoint main;`
- Built-in imports: `#use_builtin "syscall" #as __syscall;`
- Procedures: `#proc name : type { body }`
- Mutable let bindings: `#let mut x = value;`
- Assignment operations: `x = value;`
- Numeric literals: `42u64`, `3.14f32`
- Non-dependent arrows: `A -> B` (in addition to dependent `(x : A) -> B`)
- Arrays and structs: `#array`, `#struct`
- Conditional expressions: `#if condition { ... } #else { ... }`
- Let bindings: `#let x = value;`
- Field access: `struct.field`
- Field assignment: `struct.field = value;`
- Constructor calls: `Constructor(args)`
- Parenthesized expressions: `(expr)`
- Statement sequencing with `#then`

## Development Guidelines

### Testing Rules (from neco-cic/CLAUDE.md)
- **One assertion per test function** - Makes failures clearer and tests more focused
- **Tests at end of implementation files** - Keep related code together
- Use descriptive test names explaining what behavior is tested

### Code Organization
- Keep modules focused and cohesive
- Use clear, descriptive names
- Document public APIs with doc comments
- Follow CIC typing rules precisely in kernel implementation

### Task Completion
- **Always run `tools/check.sh`** after completing tasks to verify format, lint, and tests pass
- This ensures code quality before considering a task complete

## Trees That Grow Architecture

**neco-felis-syn** implements the Trees That Grow design pattern for extensible syntax trees that can carry phase-specific information during different compilation stages.

### Phase System

The phase system is defined in `phase.rs`:

```rust
pub trait Phase {
    type FileExt: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash;
    type ItemExt: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash;
    type ItemDefinitionExt: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash;
    // ... and so on for all syntax node types
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseParse();

impl Phase for PhaseParse {
    type FileExt = ();
    type ItemExt = ();
    // ... all extension types are () for parsing phase
}
```

### Phase-Parameterized Syntax

All syntax structures are parameterized by phase:

```rust
pub struct File<P: Phase> {
    items: Vec<Item<P>>,
    ext: P::FileExt,
}

pub enum Item<P: Phase> {
    Inductive(ItemInductive<P>),
    Definition(ItemDefinition<P>),
    Theorem(ItemTheorem<P>),
    Ext(P::ItemExt),  // Extension point for new item types
}

pub enum Term<P: Phase> {
    Variable(TermVariable<P>),
    Apply(TermApply<P>),
    // ... other term variants
    Ext(P::TermExt),  // Extension point for new term types
}
```

### Phase Usage Guidelines

1. **Parse Phase**: Use `PhaseParse` for parsing operations
   - All `*Ext` types are `()` (empty)
   - Parse implementations target `Struct<PhaseParse>`

2. **Extension Points**: Each syntax node has an `Ext(P::*Ext)` variant
   - Allows adding new syntax without modifying existing code
   - Must handle with `unreachable!()` for phases that don't use extensions

3. **Trait Bounds**: All extension types must implement standard traits:
   - `Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash`

4. **Type Checking Integration**: `neco-felis-type-check` uses `PhaseParse` types
   - Must handle `Ext(_)` variants with `unreachable!()` patterns

### Benefits

- **Extensibility**: New phases can add information without changing existing code
- **Type Safety**: Phase mismatches caught at compile time
- **Clean Separation**: Each phase carries only relevant information
- **Future-Proof**: Easy to add new compilation phases or syntax variants
