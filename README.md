# Wurbo Rhai Formula

A simple example of how you can build a powerful dynamic formula system using:

- [`Rhai`](https://rhai.rs/) Dynamic formula
- [`Wurbo`](https://github.com/DougAnderson444/wurbo): Interactive WIT-Wasm, uses [`minijinja`](https://docs.rs/minijinja/latest/minijinja/) under the hood, and 
- [`WIT`](https://component-model.bytecodealliance.org/) Powerful WebAssembly components

## Why not just use ____?

Why not just use:

- `JavaScript`? JS can easily exfiltrate data, so it's a security risk. WIT WebAssembly components are sandboxed, so your data goes nowhere unless you say so.
- `Excel/Sheets`? Excel cannot separate data from the template, and is easily copied/pirated. 

## A simple, flexible formula.

We want our user to be able to calculate something amongst `revenue` and `expenses`, but we want to give them the flexibility to use any formula they want.

Our system allows user to define their own formula, say they want basic profit:

```rust
let formula = "revenue - expenses";
```

Say another wants to calculate profit margin after taxes of 50%:

```rust
let formula = "(revenue - expenses) * 0.5";
```

This form can handle either of these formulas.

## How to use

The key part is the `PageContext` struct that implements `StructObject` for `Rhai`. This allows us to use the values in the struct in our Rhai templates.

```rust
/// Implement StructObject for PageContext so we can use these values in our minijinja templates
impl StructObject for PageContext {
    fn get_field(&self, name: &str) -> Option<Value> {
        let engine = Engine::new();
        let mut scope = Scope::new();
        scope.push("revenue", *self.revenue);
        scope.push("expenses", *self.expenses);

        let evaluated_formula = engine.eval_with_scope::<f64>(&mut scope, &self.formula.clone());

        match name {
            "id" => Some(Value::from(wurbo::utils::rand_id())),
            "revenue" => Some(Value::from(*self.revenue)),
            "expenses" => Some(Value::from(*self.expenses)),
            "output" => Some(Value::from(*self.revenue - *self.expenses)),
            "formula" => Some(Value::from(self.formula.as_str())),
            "evaluated_formula" => match evaluated_formula {
                Ok(result) => Some(Value::from(result)),
                Err(err) => Some(Value::from(err.to_string())),
            },
            _ => None,
        }
    }

    /// So that debug will show the values
    fn static_fields(&self) -> Option<&'static [&'static str]> {
        Some(&["id", "revenue", "expenses", "output"])
    }
}
```
