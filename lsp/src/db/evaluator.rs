use super::resolution::*;
use super::types::*;

/// Errors that can occur during @computed expression evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    FieldNotFound { field: String },
    UnresolvedHub(String),
    TypeMismatch { expected: String, actual: String },
    ParseError(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::FieldNotFound { field } => {
                write!(f, "Field '{}' not found on current instance", field)
            }
            EvalError::UnresolvedHub(name) => {
                write!(f, "Cannot resolve hub reference '{}'", name)
            }
            EvalError::TypeMismatch { expected, actual } => {
                write!(f, "Expected type {}, got {}", expected, actual)
            }
            EvalError::ParseError(msg) => {
                write!(f, "Expression parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Minimal expression AST for @computed formulas.
/// ponytail: Hand-rolled tokenizer/parser instead of pulling in nom/pom.
/// The expression grammar is simple enough that a recursive descent parser is fewer lines than any framework's boilerplate.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(ExprValue),
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
    },
    /// Identifier or "this" reference
    Ident(String),
    /// Dot access: obj.field or obj.role or result.length
    DotAccess {
        target: Box<Expr>,
        member: String,
    },
    /// Parenthesized sub-expression
    Group(Box<Expr>),
    /// Call expression (for collection operators)
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Arrow function / lambda expression
    Arrow {
        param: String,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Tokenizer for expression strings like `first_name + ' ' + last_name` or `this.companions.length`.
struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
}

#[derive(Debug, Clone)]
enum Tok {
    Ident(String),
    Number(f64),
    Str(String),
    Bool(bool),
    Op(String),
    Dot,
    LParen,
    RParen,
    EOF,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn next(&mut self) -> Tok {
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Tok::EOF;
        }

        let ch = self.src.as_bytes()[self.pos];

        // String literal
        if ch == b'\'' || ch == b'"' || ch == b'`' {
            return self.read_string(ch);
        }

        // Identifier / keyword (must come before number so "this" isn't broken)
        if ch.is_ascii_alphabetic() || ch == b'_' {
            return self.read_ident();
        }

        // Dot must be checked before number, since "." is also a valid number start
        if ch == b'.' {
            // Look ahead: if next char is a digit, it's a decimal number like ".5"
            if self.pos + 1 < self.src.len() && self.src.as_bytes()[self.pos + 1].is_ascii_digit() {
                return self.read_number();
            }
            // Otherwise it's a dot access operator
            self.pos += 1;
            return Tok::Dot;
        }

        // Number (digits only, since "." is handled above)
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        if ch == b'=' {
            if self.pos + 1 < self.src.len() && self.src.as_bytes()[self.pos + 1] == b'>' {
                self.pos += 2;
                return Tok::Op("=>".to_string());
            }
        }

        // Operators and punctuation
        self.pos += 1;
        let byte = ch as char;
        match byte {
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            c if "+-*/".contains(c) => Tok::Op(c.to_string()),
            _ => Tok::Op(byte.to_string()),
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn read_string(&mut self, quote: u8) -> Tok {
        self.pos += 1; // skip opening quote
        let start = self.pos;
        while self.pos < self.src.len() && self.src.as_bytes()[self.pos] != quote {
            self.pos += 1;
        }
        let s = self.src[start..self.pos].to_string();
        if self.pos < self.src.len() {
            self.pos += 1; // skip closing quote
        }
        Tok::Str(s)
    }

    fn read_number(&mut self) -> Tok {
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src.as_bytes()[self.pos].is_ascii_digit()
                || self.src.as_bytes()[self.pos] == b'.')
        {
            self.pos += 1;
        }
        let s = &self.src[start..self.pos];
        Tok::Number(s.parse::<f64>().unwrap_or(0.0))
    }

    fn read_ident(&mut self) -> Tok {
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src.as_bytes()[self.pos].is_ascii_alphanumeric()
                || self.src.as_bytes()[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let s = &self.src[start..self.pos];
        match s {
            "true" => Tok::Bool(true),
            "false" => Tok::Bool(false),
            _ => Tok::Ident(s.to_string()),
        }
    }
}

/// Recursive descent parser. Grammar (simplified):
///   expr       -> add_expr
///   add_expr   -> mul_expr (('+' | '-') mul_expr)*
///   mul_expr   -> unary_expr (('*' | '/') unary_expr)*
///   unary_expr -> ('-' | '!') unary_expr | member_access
///   member_access -> primary ('.' ident)*
///   primary    -> NUMBER | STRING | BOOL | IDENT | '(' expr ')'
pub fn parse_expression(src: &str) -> Option<Expr> {
    let mut tokens = Vec::new();
    let mut tz = Tokenizer::new(src);
    loop {
        let tok = tz.next();
        if matches!(&tok, Tok::EOF) {
            break;
        }
        tokens.push(tok);
    }
    let mut parser = Parser { tokens, pos: 0 };
    parser.parse_arrow()
}

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.tokens.get(self.pos).unwrap_or(&Tok::EOF)
    }

    fn eat(&mut self) -> Tok {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn parse_arrow(&mut self) -> Option<Expr> {
        if self.pos + 1 < self.tokens.len() {
            if let (Tok::Ident(param), Tok::Op(ref op)) =
                (&self.tokens[self.pos], &self.tokens[self.pos + 1])
            {
                if op == "=>" {
                    let param = param.clone();
                    self.pos += 2; // consume param and =>
                    let body = self.parse_arrow()?;
                    return Some(Expr::Arrow {
                        param,
                        body: Box::new(body),
                    });
                }
            }
        }
        self.parse_add()
    }

    fn parse_add(&mut self) -> Option<Expr> {
        let mut left = self.parse_mul()?;
        while matches!(self.peek(), Tok::Op(ref o) if *o == "+" || *o == "-") {
            let op = self.eat();
            let right = self.parse_mul()?;
            left = Expr::Binary {
                op: match op {
                    Tok::Op(o) => o,
                    _ => unreachable!(),
                },
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_mul(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Tok::Op(ref o) if *o == "*" || *o == "/") {
            let op = self.eat();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op: match op {
                    Tok::Op(o) => o,
                    _ => unreachable!(),
                },
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if matches!(self.peek(), Tok::Op(ref o) if *o == "-" || *o == "!") {
            let op = self.eat();
            let operand = self.parse_unary()?;
            return Some(Expr::Unary {
                op: match op {
                    Tok::Op(o) => o,
                    _ => unreachable!(),
                },
                operand: Box::new(operand),
            });
        }
        self.parse_member()
    }

    fn parse_member(&mut self) -> Option<Expr> {
        let mut target = self.parse_primary()?;
        loop {
            match self.peek().clone() {
                Tok::Dot => {
                    self.eat(); // consume dot
                    match self.peek().clone() {
                        Tok::Ident(member) => {
                            self.eat();
                            target = Expr::DotAccess {
                                target: Box::new(target),
                                member: member.to_string(),
                            };
                        }
                        _ => return None,
                    }
                }
                Tok::LParen => {
                    self.eat(); // consume LParen
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Tok::RParen) {
                        loop {
                            let arg = self.parse_arrow()?;
                            args.push(arg);
                            if matches!(self.peek(), Tok::Op(ref o) if o == ",") {
                                self.eat(); // consume comma
                            } else {
                                break;
                            }
                        }
                    }
                    if matches!(self.peek(), Tok::RParen) {
                        self.eat(); // consume RParen
                    } else {
                        return None;
                    }
                    target = Expr::Call {
                        target: Box::new(target),
                        args,
                    };
                }
                _ => break,
            }
        }
        Some(target)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek().clone() {
            Tok::Number(n) => {
                self.eat();
                Some(Expr::Literal(ExprValue::Number(n)))
            }
            Tok::Str(s) => {
                self.eat();
                Some(Expr::Literal(ExprValue::String(s.to_string())))
            }
            Tok::Bool(b) => {
                self.eat();
                Some(Expr::Literal(ExprValue::Boolean(b)))
            }
            Tok::Ident(i) => {
                self.eat();
                Some(Expr::Ident(i.to_string()))
            }
            Tok::LParen => {
                self.eat();
                let inner = self.parse_arrow()?;
                if matches!(self.peek(), Tok::RParen) {
                    self.eat();
                }
                Some(Expr::Group(Box::new(inner)))
            }
            Tok::EOF | _ => None,
        }
    }
}

/// Evaluate an AST against a HubInstance, resolving field names and role traversals.
pub fn evaluate_ast(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    expr: &Expr,
) -> Result<HubValue, EvalError> {
    evaluate_ast_impl(db, workspace, instance, expr, None)
}

fn evaluate_ast_impl(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    expr: &Expr,
    lambda_ctx: Option<(&str, &HubValue)>,
) -> Result<HubValue, EvalError> {
    match expr {
        Expr::Literal(ExprValue::Number(n)) => Ok(HubValue::Number(RawF64::from_f64(*n))),
        Expr::Literal(ExprValue::String(s)) => Ok(HubValue::Text(s.clone())),
        Expr::Literal(ExprValue::Boolean(b)) => Ok(HubValue::Boolean(*b)),

        Expr::Ident(name) => {
            if let Some((param_name, param_val)) = lambda_ctx {
                if name == param_name {
                    return Ok(param_val.clone());
                }
            }
            match resolve_field_or_this(db, workspace, instance, name)? {
                Some(val) => Ok(val),
                None => Err(EvalError::FieldNotFound {
                    field: name.to_string(),
                }),
            }
        }

        Expr::Binary { op, left, right } => {
            let lval = evaluate_ast_impl(db, workspace, instance, left, lambda_ctx)?;
            let rval = evaluate_ast_impl(db, workspace, instance, right, lambda_ctx)?;
            apply_binary(op, &lval, &rval)
        }

        Expr::Unary { op, operand } => {
            let oval = evaluate_ast_impl(db, workspace, instance, operand, lambda_ctx)?;
            apply_unary(op, &oval)
                .ok_or_else(|| EvalError::ParseError(format!("Invalid unary operation: {}", op)))
        }

        Expr::Group(inner) => {
            let inner_val = evaluate_ast_impl(db, workspace, instance, inner, lambda_ctx)?;
            Ok(inner_val)
        }

        Expr::DotAccess { target, member } => {
            if let Expr::Ident(ref id) = **target {
                if id == "this" {
                    return resolve_this_member(db, workspace, instance, member).map(|opt| {
                        opt.ok_or_else(|| EvalError::FieldNotFound {
                            field: member.to_string(),
                        })
                    })?;
                }
                if let Some((param_name, param_val)) = lambda_ctx {
                    if id == param_name {
                        return resolve_dot_member(db, workspace, instance, param_val, member)
                            .map(|opt| {
                                opt.ok_or_else(|| EvalError::FieldNotFound {
                                    field: format!("{}", param_name),
                                })
                            })?;
                    }
                }
            }
            let target_val = evaluate_ast_impl(db, workspace, instance, target, lambda_ctx)?;
            match resolve_dot_member(db, workspace, instance, &target_val, member)? {
                Some(val) => Ok(val),
                None => Err(EvalError::FieldNotFound {
                    field: member.to_string(),
                }),
            }
        }

        Expr::Call { target, args } => {
            if let Expr::DotAccess {
                target: ref sub_target,
                ref member,
            } = **target
            {
                let target_val =
                    evaluate_ast_impl(db, workspace, instance, sub_target, lambda_ctx)?;
                match member.as_str() {
                    "len" => {
                        if let HubValue::Array(vals) = target_val {
                            return Ok(HubValue::Number(RawF64::from_f64(vals.len() as f64)));
                        }
                    }
                    "map" => {
                        if let HubValue::Array(vals) = target_val {
                            if let Some(Expr::Arrow {
                                ref param,
                                ref body,
                            }) = args.first()
                            {
                                let mut mapped_vals = Vec::new();
                                for val in vals {
                                    match evaluate_lambda(
                                        db, workspace, instance, body, param, &val,
                                    ) {
                                        Ok(res) => mapped_vals.push(res),
                                        Err(e) => return Err(e),
                                    }
                                }
                                return Ok(HubValue::Array(mapped_vals));
                            }
                        }
                    }
                    "join" => {
                        if let HubValue::Array(vals) = target_val {
                            if let Some(arg_expr) = args.first() {
                                let delim_result = evaluate_ast_impl(
                                    db, workspace, instance, arg_expr, lambda_ctx,
                                )?;
                                if let HubValue::Text(delim) = delim_result {
                                    let mut string_parts = Vec::new();
                                    for val in vals {
                                        if let Ok(s) = String::try_from(&val) {
                                            string_parts.push(s);
                                        }
                                    }
                                    return Ok(HubValue::Text(string_parts.join(&delim)));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(EvalError::ParseError("Unknown call expression".to_string()))
        }

        Expr::Arrow { .. } => {
            return Err(EvalError::ParseError(
                "Arrow expressions cannot be evaluated directly".to_string(),
            ));
        }
    }
}

fn evaluate_lambda(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    body: &Expr,
    param: &str,
    val: &HubValue,
) -> Result<HubValue, EvalError> {
    evaluate_ast_impl(db, workspace, instance, body, Some((param, val)))
}

/// Resolve a member access on "this" (the current instance).
fn resolve_this_member(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    member: &str,
) -> Result<Option<HubValue>, EvalError> {
    // Try as a field first
    if let Ok(Some(val)) = compute_field_value(db, workspace, instance, member.to_string()) {
        return Ok(Some(val));
    }
    // Try as a role -> returns array of linked instance identifiers
    Ok(resolve_role_targets(db, workspace, instance, member))
}

/// Resolve a bare identifier: "this" returns the current instance context,
/// otherwise look it up as a field name.
fn resolve_field_or_this(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    name: &str,
) -> Result<Option<HubValue>, EvalError> {
    if name == "this" {
        return Ok(Some(HubValue::Identifier(name.to_string())));
    }
    compute_field_value(db, workspace, instance, name.to_string())
}

/// Resolve a dot member access. Handles:
/// - `ident.role_name` -> traverse role on the named instance to get linked instances as an Array
/// - `array.length` -> return array length as a Number
/// - `instance.field` -> resolve field on another instance
/// - `array.name` -> resolve field on the first element of the array (for single-target roles)
fn resolve_dot_member(
    db: &dyn Db,
    workspace: Workspace,
    _self_inst: HubInstance<'_>,
    target_val: &HubValue,
    member: &str,
) -> Result<Option<HubValue>, EvalError> {
    match target_val {
        HubValue::Identifier(id) => {
            if id == "this" {
                return Ok(None);
            }
            let inst = resolve_reference(db, workspace, id.clone())
                .ok_or_else(|| EvalError::UnresolvedHub(id.clone()))?;
            if let Some(val) = compute_field_value(db, workspace, inst, member.to_string())? {
                return Ok(Some(val));
            }
            Ok(resolve_role_targets(db, workspace, inst, member))
        }

        HubValue::Array(vals) => {
            // .length on an array
            if member == "length" {
                return Ok(Some(HubValue::Number(RawF64::from_f64(vals.len() as f64))));
            }
            // For single-target roles (e.g. this.owner.name), resolve field on the first element
            if let Some(HubValue::Identifier(ref id)) = vals.first() {
                if let Some(inst) = resolve_reference(db, workspace, id.clone()) {
                    if let Some(val) = compute_field_value(db, workspace, inst, member.to_string())?
                    {
                        return Ok(Some(val));
                    }
                }
            }
            Ok(None)
        }

        _ => Ok(None),
    }
}

/// When the target of a dot access is an identifier, and the member is a role name,
/// resolve it to an Array of HubValue::Identifier for each linked instance.
fn resolve_role_targets(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    role_name: &str,
) -> Option<HubValue> {
    let type_name = instance.type_name(db);
    let file = instance.file(db);
    let hub_type = resolve_type(db, workspace, file, type_name)?;

    let roles = hub_type.roles(db);
    let role_def = roles.iter().find(|r| r.name == role_name)?;

    let assignments = instance.assignments(db);
    let assignment = assignments.iter().find(|a| a.name == role_name)?;

    let refs = assignment.value.extract_refs();

    let mut targets = Vec::new();
    for ref_name in refs {
        if let Some(target_inst) = resolve_reference(db, workspace, ref_name.clone()) {
            // ponytail: Polymorphic filtering - child types satisfy parent roles
            let target_type_name = target_inst.type_name(db);
            let allowed = if role_def.allowed_types.contains(&target_type_name) {
                true
            } else if let Some(parent_type) = resolve_type(
                db,
                workspace,
                target_inst.file(db),
                target_type_name.clone(),
            ) {
                crate::db::polymorphic::hub_type_allows(
                    db,
                    workspace,
                    &parent_type,
                    &role_def.allowed_types,
                )
            } else {
                false
            };
            if allowed {
                targets.push(HubValue::Identifier(ref_name));
            }
        }
    }

    Some(HubValue::Array(targets))
}

/// Apply a binary operator to two HubValues.
fn apply_binary(op: &str, left: &HubValue, right: &HubValue) -> Result<HubValue, EvalError> {
    match op {
        "+" => {
            if matches!(left, HubValue::Text(_)) || matches!(right, HubValue::Text(_)) {
                let ls: String = left
                    .try_into()
                    .map_err(|_| EvalError::ParseError("Expected string".to_string()))?;
                let rs: String = right
                    .try_into()
                    .map_err(|_| EvalError::ParseError("Expected string".to_string()))?;
                return Ok(HubValue::Text(format!("{}{}", ls, rs)));
            }
            let ln: f64 = left
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            let rn: f64 = right
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            Ok(HubValue::Number(RawF64::from_f64(ln + rn)))
        }
        "-" => {
            let ln: f64 = left
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            let rn: f64 = right
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            Ok(HubValue::Number(RawF64::from_f64(ln - rn)))
        }
        "*" => {
            let ln: f64 = left
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            let rn: f64 = right
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            Ok(HubValue::Number(RawF64::from_f64(ln * rn)))
        }
        "/" => {
            let ln: f64 = left
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            let rn: f64 = right
                .try_into()
                .map_err(|_| EvalError::ParseError("Expected number".to_string()))?;
            if rn == 0.0 {
                return Err(EvalError::ParseError("Division by zero".to_string()));
            }
            Ok(HubValue::Number(RawF64::from_f64(ln / rn)))
        }
        _ => Err(EvalError::ParseError(format!("Unknown operator: {}", op))),
    }
}

fn apply_unary(op: &str, val: &HubValue) -> Option<HubValue> {
    match op {
        "-" => {
            let n = f64::try_from(val).ok()?;
            Some(HubValue::Number(RawF64::from_f64(-n)))
        }
        "!" => match val {
            HubValue::Boolean(b) => Some(HubValue::Boolean(!b)),
            HubValue::Number(_) => {
                let n = f64::try_from(val).ok()?;
                Some(HubValue::Boolean(n == 0.0))
            }
            _ => None,
        },
        _ => None,
    }
}

/// Check if a field definition has @default and the instance did NOT assign it.
/// Returns Ok(true) if default should be applied, Ok(false) if overridden, Err if type not found.
pub fn needs_default(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    field_name: &str,
) -> Result<bool, EvalError> {
    let type_name = instance.type_name(db).to_string();
    let file = instance.file(db);
    let hub_type = resolve_type(db, workspace, file, type_name.clone())
        .ok_or_else(|| EvalError::UnresolvedHub(type_name))?;

    let fields = hub_type.fields(db);
    let field_def = fields
        .iter()
        .find(|f| f.name == field_name)
        .ok_or_else(|| EvalError::FieldNotFound {
            field: field_name.to_string(),
        })?;
    let has_default = field_def.decorator.as_deref() == Some("@default");
    if !has_default {
        return Ok(false);
    }
    let is_assigned = instance
        .assignments(db)
        .iter()
        .any(|a| a.name == field_name);
    Ok(!is_assigned)
}

/// Get the default expression for a field if it has one and wasn't overridden by the instance.
pub fn get_default_value(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    field_name: &str,
) -> Result<Option<HubValue>, EvalError> {
    let needs = needs_default(db, workspace, instance, field_name)?;
    if !needs {
        return Ok(None);
    }

    let type_name = instance.type_name(db).to_string();
    let file = instance.file(db);
    let hub_type = resolve_type(db, workspace, file, type_name.clone())
        .ok_or_else(|| EvalError::UnresolvedHub(type_name))?;
    let fields = hub_type.fields(db);
    let field_def = fields
        .iter()
        .find(|f| f.name == field_name)
        .ok_or_else(|| EvalError::FieldNotFound {
            field: field_name.to_string(),
        })?;
    let expr_str = field_def.expression.as_ref().ok_or_else(|| {
        EvalError::ParseError(format!("Field '{}' has no expression", field_name))
    })?;

    let ast = parse_expression(expr_str)
        .ok_or_else(|| EvalError::ParseError(format!("Failed to parse '{}'", expr_str)))?;
    evaluate_ast(db, workspace, instance, &ast).map(Some)
}
