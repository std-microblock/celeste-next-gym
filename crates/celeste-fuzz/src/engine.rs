use std::collections::BTreeMap;

use celeste_physics::{CoreMode, PlayerSnapshot, PlayerState, Vec2};
use rhai::{AST, Dynamic, Engine, Module, Scope};

use crate::{FuzzError, HoldTime};

#[derive(Clone)]
pub(crate) struct CompiledExpression {
    pub(crate) source: String,
    pub(crate) ast: AST,
}

pub(crate) struct ExpressionContext<'a> {
    pub(crate) variables: &'a BTreeMap<String, i64>,
    pub(crate) initial: Option<&'a PlayerSnapshot>,
    /// Alias for the snapshot at the expression's evaluation point.  Training
    /// entry checks use `current`; for after-input checks it is the post-step
    /// snapshot and for before-input checks it is the pre-step snapshot.
    pub(crate) current: Option<&'a PlayerSnapshot>,
    pub(crate) before: Option<&'a PlayerSnapshot>,
    pub(crate) after: Option<&'a PlayerSnapshot>,
    pub(crate) final_state: Option<&'a PlayerSnapshot>,
    pub(crate) at: Option<i64>,
    pub(crate) held_time: Option<HoldTime>,
    pub(crate) input_index: Option<usize>,
    pub(crate) verify: Option<bool>,
}

pub(crate) fn build_engine(max_operations: u64) -> Engine {
    // `new_raw` leaves out Rhai's standard-library collection, string, file,
    // and package helpers.  The operators needed by expressions are core
    // language functionality; the only additional callable helpers below are
    // abs/min/max/sqrt.
    let mut engine = Engine::new_raw();
    engine.set_max_operations(max_operations);
    engine.register_fn("abs", |value: i64| value.abs());
    engine.register_fn("abs", |value: f64| value.abs());
    engine.register_fn("min", |left: i64, right: i64| left.min(right));
    engine.register_fn("max", |left: i64, right: i64| left.max(right));
    engine.register_fn("min", |left: f64, right: f64| left.min(right));
    engine.register_fn("max", |left: f64, right: f64| left.max(right));
    engine.register_fn("sqrt", |value: f64| value.sqrt());

    engine.register_type_with_name::<Vec2>("Vec2");
    engine.register_get("x", |value: &mut Vec2| value.x as f64);
    engine.register_get("y", |value: &mut Vec2| value.y as f64);

    engine.register_type_with_name::<PlayerState>("PlayerState");
    engine.register_fn("==", |left: PlayerState, right: PlayerState| left == right);
    engine.register_fn("!=", |left: PlayerState, right: PlayerState| left != right);
    let mut states = Module::new();
    states.set_var("normal", PlayerState::Normal);
    states.set_var("climb", PlayerState::Climb);
    states.set_var("dash", PlayerState::Dash);
    states.set_var("swim", PlayerState::Swim);
    states.set_var("boost", PlayerState::Boost);
    states.set_var("red_dash", PlayerState::RedDash);
    states.set_var("hit_squash", PlayerState::HitSquash);
    states.set_var("launch", PlayerState::Launch);
    states.set_var("pickup", PlayerState::Pickup);
    states.set_var("dream_dash", PlayerState::DreamDash);
    states.set_var("summit_launch", PlayerState::SummitLaunch);
    states.set_var("dummy", PlayerState::Dummy);
    states.set_var("frozen", PlayerState::Frozen);
    states.set_var("reflection_fall", PlayerState::ReflectionFall);
    states.set_var("star_fly", PlayerState::StarFly);
    states.set_var("temple_fall", PlayerState::TempleFall);
    states.set_var("cassette_fly", PlayerState::CassetteFly);
    states.set_var("attract", PlayerState::Attract);
    states.set_var("intro_walk", PlayerState::IntroWalk);
    states.set_var("intro_jump", PlayerState::IntroJump);
    states.set_var("intro_respawn", PlayerState::IntroRespawn);
    states.set_var("intro_wake_up", PlayerState::IntroWakeUp);
    states.set_var("bird_dash_tutorial", PlayerState::BirdDashTutorial);
    states.set_var("intro_moon_jump", PlayerState::IntroMoonJump);
    states.set_var("fling_bird", PlayerState::FlingBird);
    states.set_var("intro_think_for_a_bit", PlayerState::IntroThinkForABit);
    engine.register_static_module("state", states.into());

    engine.register_type_with_name::<CoreMode>("CoreMode");
    engine.register_fn("==", |left: CoreMode, right: CoreMode| left == right);
    engine.register_fn("!=", |left: CoreMode, right: CoreMode| left != right);
    let mut core_modes = Module::new();
    core_modes.set_var("none", CoreMode::None);
    core_modes.set_var("hot", CoreMode::Hot);
    core_modes.set_var("cold", CoreMode::Cold);
    engine.register_static_module("core_mode", core_modes.into());

    engine.register_type_with_name::<HoldTime>("HoldTime");
    engine.register_fn("==", |left: HoldTime, right: HoldTime| left == right);
    engine.register_fn("!=", |left: HoldTime, right: HoldTime| left != right);
    let mut holds = Module::new();
    holds.set_var("inf", HoldTime::Infinite);
    engine.register_static_module("hold", holds.into());

    engine.register_type_with_name::<PlayerSnapshot>("PlayerSnapshot");
    engine.register_get("pos", |value: &mut PlayerSnapshot| value.pos);
    engine.register_get("speed", |value: &mut PlayerSnapshot| value.speed);
    engine.register_get("state", |value: &mut PlayerSnapshot| value.state);
    engine.register_get("facing", |value: &mut PlayerSnapshot| value.facing);
    engine.register_get("dashes", |value: &mut PlayerSnapshot| value.dashes as i64);
    engine.register_get("stamina", |value: &mut PlayerSnapshot| value.stamina as f64);
    engine.register_get("on_ground", |value: &mut PlayerSnapshot| value.on_ground);
    engine.register_get("ducking", |value: &mut PlayerSnapshot| value.ducking);
    engine.register_get("dead", |value: &mut PlayerSnapshot| value.dead);
    engine.register_get("dash_dir", |value: &mut PlayerSnapshot| value.dash_dir);
    engine.register_get("last_aim", |value: &mut PlayerSnapshot| value.last_aim);
    engine.register_get("core_mode", |value: &mut PlayerSnapshot| value.core_mode);
    engine
}

pub(crate) fn compile_expression(
    engine: &Engine,
    source: &str,
) -> Result<CompiledExpression, FuzzError> {
    validate_expression_surface(source)?;
    let ast = engine.compile_expression(source).map_err(|error| {
        FuzzError::Spec(format!("cannot compile expression `{source}`: {error}"))
    })?;
    Ok(CompiledExpression {
        source: source.to_owned(),
        ast,
    })
}

pub(crate) fn evaluate(
    engine: &Engine,
    expression: &CompiledExpression,
    context: ExpressionContext<'_>,
) -> Result<Dynamic, String> {
    let mut scope = Scope::new();
    for (name, value) in context.variables {
        scope.push(name.as_str(), *value);
    }
    if let Some(value) = context.initial {
        scope.push("initial", value.clone());
    }
    if let Some(value) = context.current {
        scope.push("current", value.clone());
    }
    if let Some(value) = context.before {
        scope.push("before", value.clone());
    }
    if let Some(value) = context.after {
        scope.push("after", value.clone());
    }
    if let Some(value) = context.final_state {
        scope.push("final", value.clone());
    }
    if let Some(value) = context.at {
        scope.push("at", value);
    }
    if let Some(value) = context.held_time {
        match value {
            HoldTime::Infinite => scope.push("held_time", HoldTime::Infinite),
            HoldTime::Frames(frames) => scope.push("held_time", frames),
        };
    }
    if let Some(value) = context.input_index {
        scope.push("input_index", value as i64);
    }
    if let Some(value) = context.verify {
        scope.push("verify", value);
    }
    engine
        .eval_ast_with_scope::<Dynamic>(&mut scope, &expression.ast)
        .map_err(|error| format!("{} ({})", error, expression.source))
}

pub fn evaluate_current_checks(
    current: &PlayerSnapshot,
    expressions: &[String],
) -> Result<bool, FuzzError> {
    let engine = build_engine(10_000);
    let variables = BTreeMap::new();
    for source in expressions {
        let expression = compile_expression(&engine, source)?;
        let result = evaluate(
            &engine,
            &expression,
            ExpressionContext {
                variables: &variables,
                initial: None,
                current: Some(current),
                before: None,
                after: None,
                final_state: None,
                at: None,
                held_time: None,
                input_index: None,
                verify: None,
            },
        )
        .map_err(FuzzError::Spec)?;
        if result.try_cast::<bool>() != Some(true) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn validate_expression_surface(source: &str) -> Result<(), FuzzError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(FuzzError::Spec("empty expression".into()));
    }
    if trimmed.contains('"') || trimmed.contains('\'') {
        return Err(FuzzError::Spec(format!(
            "string literals are not allowed in expression `{source}`; compare native enums instead"
        )));
    }
    if trimmed.contains(';') || trimmed.contains('{') || trimmed.contains('}') {
        return Err(FuzzError::Spec(format!(
            "statement syntax is not allowed in expression `{source}`"
        )));
    }
    for forbidden in [
        "fn", "while", "for", "loop", "import", "export", "eval", "this",
    ] {
        if identifier_tokens(trimmed).contains(&forbidden) {
            return Err(FuzzError::Spec(format!(
                "`{forbidden}` is not allowed in expression `{source}`"
            )));
        }
    }
    for module in static_module_tokens(trimmed) {
        if !matches!(module.as_str(), "state" | "core_mode" | "hold") {
            return Err(FuzzError::Spec(format!(
                "module `{module}` is not allowed in expression `{source}`"
            )));
        }
    }
    let bytes = trimmed.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_alphabetic() || bytes[cursor] == b'_' {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
            let identifier = &trimmed[start..cursor];
            let mut after = cursor;
            while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            if after < bytes.len()
                && bytes[after] == b'('
                && !matches!(identifier, "abs" | "min" | "max" | "sqrt")
            {
                return Err(FuzzError::Spec(format!(
                    "function `{identifier}` is not allowed in expression `{source}`"
                )));
            }
        } else {
            cursor += 1;
        }
    }
    Ok(())
}

fn identifier_tokens(source: &str) -> Vec<&str> {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|part| !part.is_empty())
        .collect()
}

fn static_module_tokens(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut modules = Vec::new();
    for index in 0..bytes.len().saturating_sub(1) {
        if bytes[index] == b':' && bytes[index + 1] == b':' {
            let prefix = &source[..index];
            let name = prefix
                .rsplit(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .next()
                .unwrap_or_default();
            modules.push(name.to_owned());
        }
    }
    modules
}
