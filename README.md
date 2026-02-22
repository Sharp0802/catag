# CATAG : a Context-Aware Typed Algebraic Grammar for modular parsing

> [!NOTE]
> :building_construction: Work in progress...

**CATAG** (pronounced `/kʰa.tak/`, *kah-tak*) is next-generation meta-language
for generating type-safe parsers with declarative, structural error recovery.

## it's *Typed Algebraic*

**Your grammar is your type system.**

- **Algebraic Data Types**:
  Rules and entrypoint are defined as enums and tuples.

  When you define a rule,
  CATAG automatically guarantees a 1:1 isomorphic AST generation.
  
  No more  double-maintenance.

### Parametric Polymorphism (Generics)

```
rule Separated<T, D>(items: T[rest.., last]) = (rest D)* last

rule CsvLine(items: Value[]) = Separated<Value, ','>(items) <-- Reuse Separated with ','
rule Paths(items: Value[])   = Separated<Path, ':'>(items)  <-- Reuse Separated with ':'
```

Why copy-paste rules?
CATAG supports higher-order rules like `Separated<T, D>`,
allowing you to parameterize grammars
just like you parameterize functions in modern programming languages.

### Implicit Token Subtyping

If your lexer defines `PUNC = [.,;?]` and you use the literal `','` in your grammar,
CATAG calculates the inclusion relationship at compile time.

It isolates `','` into a high-priority singleton token,
but automatically permits it wherever `PUNC` is expected.

You get strict literal matching without destroying generic token categories.

## it's *Context-Aware*

In CATAG, Parsers know where they are and what they are trying to do.

### Semantic Error Reporting

```
rule Field(name: IDENT, value: LITERAL) = name ':' value ;
```

Human-readable errors tied to grammatical contexts right in the syntax like:

- `some-name: <EOF>` : `Expected value while parsing Field ... (details)`
- `:<EOF>` : `Expected name or ... while parsing ... (details)`

Even if you need to describe special edge-case, just do:

```
rule JsonName(name: STRING)
    = name
    | IDENT -> error("Use double-quoted string instead")
    ;
```

### Declarative Error Recovery

```
rule Block(statements: Stmt[]) = '{' (recover<Stmt, ';' | '}'>(statements) ';')+ '}'
```

Instead of relying on hidden engine heuristics,
you explicitly declare "Synchronization Sets" using union types.

The parser understands its structural boundaries and skips exactly what it needs to safely resume parsing.

```
{
  Stmt;
  Non-Stmt <-- Expected statements (got ...) while parsing Block.
  ;        <-- (continue parsing from here)
  Stmt;
}
```
