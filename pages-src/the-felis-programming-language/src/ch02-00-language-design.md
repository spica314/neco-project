# Language Design

## Design 01: No identifiers that conflict with keywords

In the Felis programming language, all keywords are prefixed with `#`, ensuring they do not conflict with identifiers.  Take for instance the keyword for type definition, which is `#type`. It allows for the `type` to be used as an identifier without any issues. 

This design strategy effectively removes the need to worry about naming conflicts between keywords and identifiers. This is particularly useful when implementing serializers and compilers.

## Design 02: You can use a `-` within identifiers

In the Felis programming language, you can use a `-` within identifiers.
For example, `kebab-case` is a valid identifier name.

In certain programming contexts, kebab-case is conventionally used. 
This design obviates the need for selecting different case styles in these situations.
