# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**neco-project** is a hobbyist compiler and programming language implementation:
- **neco**: Compiler for the Felis programming language
- **Felis**: Dependently-typed functional programming language implementing Calculus of Inductive Constructions (CIC)

## Workspace Architecture

This is a Rust workspace with 3 crates forming a compiler pipeline:

1. **neco-cic** - Core CIC kernel with type theory, reduction, and type checking
2. **neco-felis-syn** - Frontend parser and syntax for Felis language
3. **neco-felis-type-check** - Bridges Felis syntax with CIC kernel

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

# Test with snapshots (neco-felis-syn uses insta)
cargo test -p neco-felis-syn
```

### Development
```bash
# Run clippy
cargo clippy --workspace

# Format code
cargo fmt --workspace
```

## Architecture Notes

### Data Flow
Felis source → `neco-felis-syn` (parsing) → `neco-felis-type-check` (validation) → `neco-cic` (kernel evaluation)

### Key Design Patterns
- **Unique IDs**: Variables use unique IDs to avoid capture issues in substitution
- **Reference Counting**: Extensive use of `Rc<T>` for sharing AST nodes
- **Modular Type Theory**: Clean separation between frontend syntax and kernel implementation

### Testing Strategy
- **Unit tests**: Place tests at end of implementation files, one assertion per test function
- **Integration tests**: Use testcases in `/testcases/felis/single/` with both positive and negative cases
- **Snapshot testing**: `neco-felis-syn` uses `insta` for parser output validation

## File Structure

```
neco-cic/src/
├── term.rs              # CIC term AST
├── typechecker.rs       # Type checking rules
├── reduction.rs         # Term reduction/normalization
├── global_environment.rs # Global definitions
├── inductive.rs         # Inductive type support
└── substitution.rs      # Variable substitution

neco-felis-syn/src/
├── token.rs             # Lexical analysis
├── parse.rs             # Parser framework
├── file.rs              # File-level parsing
├── item_*.rs            # Top-level items
└── term_*.rs            # Term-level syntax

testcases/felis/single/  # Test cases for language features
```

## Language Features (Felis)

Felis supports dependent types with syntax for:
- Inductive types: `#inductive nat : Set { O: nat, S: nat -> nat }`
- Definitions: `#definition add : (n : nat) -> (m : nat) -> nat { ... }`
- Theorems: `#theorem name : type { proof }`
- Pattern matching: `#match expr { pattern => body }`
- Dependent function types: `(A : Type) -> (x : A) -> ...`

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
