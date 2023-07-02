# Language Design

## Design 01: No identifiers that conflict with keywords

In the Felis programming language, all keywords are prefixed with `#`, ensuring they do not conflict with identifiers.  Take for instance the keyword for type definition, which is `#type`. It allows for the `type` to be used as an identifier without any issues. 

This design strategy effectively removes the need to worry about naming conflicts between keywords and identifiers. This is particularly useful when implementing serializers and compilers.
