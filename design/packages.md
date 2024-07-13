# Packages, libraries, modules, etc

This files goes over the options that can be used, what was decided and the rational behind it.
_Note: this is subject to change if needed_

## Packages in other languages

Other langauges generally support a feature that can be associated with the concept of a package, below is a sampling of langauges and how they handle it

### C

C does not have an inherent concept of packages, instead it expects to be provided as a series of header files and associated compiler-specific library files.

### C++

C++ has 2 systems to handle it, the first is conceptually identical to C, but with the advant of C++20, modules were introduced.
Modules seem to be mostly a replacement for header files and seem to only encapsulate 1 translation unit, what would be represented by just 1 file in most languages.
Although it allows for multiple modules to be under the same module names, essentailly packing up these together.

Only 1 module can be defined as the first unit of a type and has the following syntax (all lines represent separate files):
```
export module A;   // declares the primary module interface unit for the named module 'A'
module A;          // declares a module implementation for the named module 'A'
module A;          // another module implementation for the named module 'A'
export module A.B; // declares the primiary module interface for name module 'A.B'
export module A.B; // declares a module implementation for the named module 'A.B'
```

Although it would seem that 'A.B' would be a submodule of 'A', dots do not actually mean that, they are mainly used to represent an informal hierachy.

There is also a concept of module partitions, written as `module A:B`, partitions represent private 'sub-modules' within a given module.
These cannot be accessed from outside the module, but their content can be exported from 'A' directly.

### Rust

Rust compilation process centers on artifacts called crates, which represent the output of a single compilation process, and produces a library or binary.

Crates themselve consist out of modules, there is generally a main module, in a file called `lib.rs` or `main.rs`, representing a library of binary respectively.
These can consists out of as many sub-modules as neccesary.

The general way used to support what could be called sub-crates, would be by e.g. having crate called 'foo', with crates that are related to them named 'foo-bar',
meaning prefixing crates that are related with the name of the main crate.

### Swift

Swift consists out of top-level packages, a package has one or more products within it.
Produces are the actual libraries and executables, and these in turn contain libraries

### Go

Go represent a package as a collection of compiled files.
The package to which a file belongs is declared at the top of the file

### Kotlin

Kotlin consist out of a top-level package that can consist out of a set of nested packages.
Packages are declared at the top of each file.

Packages are also referred to as libraries

### Zig

Zig version of package would be basic libraries, these bundle a set of code files into a single unit for distribution

### Odin

Odin's top level grouping are called library collection, these consist out of packages.
Packages themselves represent a folder with odin files.
Packages can be manually

### Carbon

Carbon represents their packages as a collection of libraries.
There is a distinguishment between API libraries and implementation binaries, where the former defines the API and the latter the implementation of that API.

Packages and libraries are declared within the files themselves

# Xenon package organization

As programs generally aren't fully self contained, Xenon needs to have a system to share code acress project.
The solution for this is to use a package system to allow for this.

## Packages

Xenon packages are very similar to those that are in switch, although with some slight name changes.
So it was decided to make a 'package' the upper level of the hierarchy of artifacts and the main unit of distribution.

Packages themselves are not the direct result of compilation, but play an integral part if code organization, including how packages are 'imported'.
A package can contain any number of artifacts, allowing allow related code to be shared as a single unit,
meaning that if a project is split up in modularized components, they can still be easilty distributed, without having to result to sub-naming.

## Artifacts

Artifacts, unlike packages, are the direct result of a compilation process or stage.

An artifact consts out of 3 distinct types:
- binaries
- static libraries
- dynamic libraries

Artifact themselves are made up from modules.

### Binaries

Binaries are the resulting runnable executables, these are not meant to be 'imported', as they miss all the data required for it.
These can be delivered together with binaries not only be used as the final application, but also tools used for any operation

### Static ibraries

A static library is a library that is meant to be linked into any code using it.
It contains all info needed to 'import' and use it in other code, including the bytecode for all the relavent issues.

If possible, the compiler can inline any code within the static library.

### Dynamic ibraries

A dynamic library is a library that is meant to be referenced by code linking to it, unlike a static binary, this is not linked directly into the code, but lives as it's own file right next to io.
Dynamic libraries actually generates 2 resulting file: a xenon library and a OS-specific dynamic library.
The xenon library is similar to that produces for static libraries, but does not contain all data that the static library has, but only includes what is needed to successfully build and reference the dynamic library in code using it.

## Modules

A module generally represents a single file or inlined module definition (if a file is not direclty included within another file).
Each module is allowed to have multiple sub-modules.

Each artifact has it's own main module, which by default uses the following files:
- binaries: main.xn
- static and dynamic libraries: lib.xn

## Examples

An example hierarchy could be:
```
package
├─ static library
│  └─ module
├─ dynamc library
│  └─ module
└─ binary
   ├─ module
   │  └─ module
   └─ module
```