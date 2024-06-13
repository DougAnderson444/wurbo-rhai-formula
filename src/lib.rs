// Declare our module so it's included in our crate
#[allow(warnings)]
mod bindings;

use std::ops::Deref;

// We will likely have other guests, so let's alias this one to WurboGuest
use bindings::demo::form::context_types;
use bindings::demo::form::context_types::Context;
use bindings::demo::form::wurbo_in;
use bindings::exports::demo::form::wurbo_out::Guest as WurboGuest;
use rhai::Engine;
use rhai::Scope;
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

bindings::export!(Component with_types_in bindings);

/// PageContext is a struct of other structs that implement [StructObject],
/// which is why it is not a Newtype wrapper like the others are.
#[derive(Debug, Clone, Default)]
pub struct PageContext {
    var1: String,
    revenue: Revenue,
    expenses: Expenses,
    formula: Formula,
    target: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Var1(pub String);

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

/// Implement [`wurbo::prelude::Object`] for PageContext so we can use these values in our minijinja templates
impl Object for PageContext {
    fn get_value(self: &std::sync::Arc<Self>, key: &Value) -> Option<Value> {
        let engine = Engine::new();
        let mut scope = Scope::new();
        // var1
        scope.push(self.var1.as_str(), *self.revenue);
        scope.push("expenses", *self.expenses);

        let evaluated_formula = engine.eval_with_scope::<f64>(&mut scope, &self.formula.clone());

        match key.as_str()? {
            "id" => Some(Value::from(wurbo::utils::rand_id())),
            "var1" => Some(Value::from(self.var1.clone())),
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
}

/// Implement From<context_types::Context> for PageContext so we can convert the incoming context
/// into our PageContext struct
impl From<&context_types::Context> for PageContext {
    fn from(context: &context_types::Context) -> Self {
        // Route depends on context
        match context {
            context_types::Context::AllContent(all) => PageContext::from(all),
            // var1 is the name of the first variable in the formula
            context_types::Context::Var1(var1) => PageContext::from(Var1(var1.clone())),
            context_types::Context::Revenue(rev) => PageContext::from(Revenue(*rev)),
            context_types::Context::Expenses(exp) => PageContext::from(Expenses(*exp)),
            context_types::Context::Formula(form) => PageContext::from(Formula(form.clone())),
        }
    }
}

/// Implement From<context_types::AllContent> for PageContext so we can convert the incoming context
impl From<&context_types::Content> for PageContext {
    fn from(all: &context_types::Content) -> Self {
        PageContext {
            var1: "revenue".to_string(),
            revenue: Revenue(all.revenue.unwrap_or_default()),
            expenses: Expenses(all.expenses.unwrap_or_default()),
            formula: all.formula.clone().map(Formula).unwrap_or_default(),
            // None will use default of index.html, which is what we want
            target: None,
        }
    }
}

impl From<Var1> for PageContext {
    fn from(var1: Var1) -> Self {
        let mut last = LAST_STATE.lock().unwrap().clone().unwrap_or_default();
        last.var1 = var1.0;
        last
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
