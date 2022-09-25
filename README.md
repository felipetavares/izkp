# Interactive Zero-Knowledge Proofs

Direct implementation of a simple IZK proof system that is able to prove
knowledge of $x$ in $x + 1 = 2$.

There's a long explanation provided [in my blog][article].

[article]: https://felipetavares.com/post/interactive-zero-knowledge-proofs/

# How does it work?

Broadly, this is what is done:

1. Convert the statement into a boolean expression.
2. Prove the boolean expression by:
   - a) Convert it to a 3-colorable graph.
   - b) Prove a coloring exists.

The code essentially does step (2.b), everything else is done offline.
