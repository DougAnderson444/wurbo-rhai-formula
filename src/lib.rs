#![feature(lazy_cell)]
// Declare our module so it's included in our crate
mod bindings;

use std::ops::Deref;

// We will likely have other guests, so let's alias this one to WurboGuest
use bindings::demo::form::context_types;
use bindings::demo::form::context_types::Context;
use bindings::demo::form::wurbo_in;
use bindings::exports::demo::form::wurbo_out::Guest as WurboGuest;
use rhai::Scope;
use rhai::{Engine, EvalAltResult};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use wurbo::jinja::{Entry, Index, Rest, Templates};
use wurbo::prelude_bindgen;

/// Struct that we use to implement the WIT Guest function upon
struct Component;

const OUTPUT_HTML: &str = "output.html";
const INDEX_HTML: &str = "index.html";
const INPUT_HTML: &str = "input.html";

/// We need to provide the templates for the macro to pull in
fn get_templates() -> Templates {
    let templates = Templates::new(
        Index::new(INDEX_HTML, include_str!("templates/index.html")),
        Entry::new(OUTPUT_HTML, include_str!("templates/output.html")),
        Rest::new(vec![Entry::new(
            INPUT_HTML,
            include_str!("templates/input.html"),
        )]),
    );
    templates
}

// Macro builds the Component struct and implements the Guest trait for us, saving copy-and-paste
prelude_bindgen! {WurboGuest, Component, PageContext, Context, LAST_STATE}

/// PageContext is a struct of other structs that implement [StructObject],
/// which is why it is not a Newtype wrapper like the others are.
#[derive(Debug, Clone, Default)]
pub struct PageContext {
    revenue: Revenue,
    expenses: Expenses,
    formula: Formula,
    target: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Revenue(pub f64);

impl Deref for Revenue {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Expenses(pub f64);

impl Deref for Expenses {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Formula String
#[derive(Debug, Clone)]
struct Formula(String);

impl Default for Formula {
    fn default() -> Self {
        Formula("revenue - expenses".to_string())
    }
}

impl Deref for Formula {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implement StructObject for PageContext so we can use these values in our minijinja templates
impl StructObject for PageContext {
    fn get_field(&self, name: &str) -> Option<Value> {
        let engine = Engine::new();
        let mut scope = Scope::new();
        scope.push("revenue", *self.revenue);
        scope.push("expenses", *self.expenses);

        let evaluated_formula = engine.eval_with_scope::<f64>(&mut scope, &self.formula.clone());

        match name {
            "id" => {
                let id = wurbo::utils::rand_id();
                println!("id: {}", id);
                Some(Value::from(id))
            }
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

/// Implement From<context_types::Context> for PageContext so we can convert the incoming context
/// into our PageContext struct
impl From<&context_types::Context> for PageContext {
    fn from(context: &context_types::Context) -> Self {
        // Route depends on context
        match context {
            context_types::Context::AllContent(all) => PageContext::from(all),
            context_types::Context::Revenue(rev) => PageContext::from(Revenue(*rev)),
            context_types::Context::Expenses(exp) => PageContext::from(Expenses(*exp)),
            context_types::Context::Formula(form) => PageContext::from(Formula(form.clone())),
        }
    }
}

/// Implement From<context_types::AllContent> for PageContext so we can convert the incoming context
impl From<&context_types::Content> for PageContext {
    fn from(all: &context_types::Content) -> Self {
        println!("AllContent: {:?}", all);
        PageContext {
            revenue: Revenue(all.revenue.unwrap_or_default()),
            expenses: Expenses(all.expenses.unwrap_or_default()),
            formula: all.formula.clone().map(Formula).unwrap_or_default(),
            // None will use default of index.html, which is what we want
            target: None,
        }
    }
}

impl From<Revenue> for PageContext {
    fn from(rev: Revenue) -> Self {
        let mut last = LAST_STATE.lock().unwrap().clone().unwrap_or_default();
        last.revenue = rev.into();
        last
    }
}

impl From<Expenses> for PageContext {
    fn from(exp: Expenses) -> Self {
        let mut last = LAST_STATE.lock().unwrap().clone().unwrap_or_default();
        last.expenses = exp.into();
        last
    }
}

impl From<Formula> for PageContext {
    fn from(form: Formula) -> Self {
        let mut last = LAST_STATE.lock().unwrap().clone().unwrap_or_default();
        last.formula = form;
        last
    }
}
