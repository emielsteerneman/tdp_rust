use serde::Deserialize;
use rmcp::schemars::JsonSchema;
use anyhow::Result;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BinaryOpArgs {
    pub a: f64,
    pub b: f64,
}

pub fn add(args: BinaryOpArgs) -> f64 {
    args.a + args.b
}

pub fn subtract(args: BinaryOpArgs) -> f64 {
    args.a - args.b
}

pub fn multiply(args: BinaryOpArgs) -> f64 {
    args.a * args.b
}

pub fn divide(args: BinaryOpArgs) -> Result<f64> {
    if args.b == 0.0 {
        anyhow::bail!("Cannot divide by zero");
    }
    Ok(args.a / args.b)
}
