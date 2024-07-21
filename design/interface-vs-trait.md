# Interface vs trait naming

This files goes over the options tha can be used, what was decided and the raional behind it.
_Note: this is subject to change if needed_

## Interfaces in other languages

### C++

The closest c++ has to interfaces are pure-virtual classes, containing only virtual functions that are defined as `... = 0;`

### Rust

Rust calls its equivalent of interfaces "traits".
Traits can contain the following elements:
- functions & methods
- associated types
- associated constants

Traits can also have additional bounds applied on them, these can be seen as super-traits of the current traits and need to be implemented to allow a trait to be implemented.

Implmenetations of interfaces happen in individual impl blocks, 1 impl per trait.

### Swift

Swift's interfaces are known as protocols.
Traits can contain the following elements:
- methods
- properties

### Go

Go interfaces alloow the following elements:
- methods
- types

Unlike other interfaces, go types are a constraint on what types can implement the type

Interfaces are implemented implicitly by just having the matching methods and type restrictions

### Kotlin

Kotlin interfaces allow the following elements:
- methods
- properties

Interfaces that are 'derived' from other interfaces can override the default behavior of it's parent interface

If a diamond is encountered in interfaces, i.e.
```
 A
/ \
B C
\ /
 D
```

where both B and C override the default implementation of A, D needs to specify it's own overiding implementation for the conflicting functions

### Carbon

Interfaces are represented by absract classes

## Naming

### Interface

The term interface generally just means a collection of methods that the type needs to match.
This also indirectly includes the idea of properties

### Traits

Traits generally define a combination of an interface (defining methods), with mixins (having default method implementations)

# Xenon

As the term `trait` seems to better describe what they actually do, we will use that term from now on in xenon.