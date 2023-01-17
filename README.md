# Lox but in Rust (😎?)

This is an attempt of following the first part of the book [Crafting Interpreters](https://craftinginterpreters.com).
Currently it's missing the parser synchronization, an implementation for classes and objects, and
sensible error reporting on the stages of resolving and executing the code.

The prime takeaway of this project is this: **RUST IS NOT FOR PROTOTYPING**.

Even though I enjoyed greatly some of the features of Rust, the enforcing of its many (and often useful)
contracts makes it really hard to follow the objective of this interpreter, which I understand to be a
quick and simple way of understanding the syntax and semantics of Lox.

