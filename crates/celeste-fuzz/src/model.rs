use std::collections::BTreeMap;

use celeste_physics::PlayerSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value};
use thiserror::Error;

pub(crate) const RESERVED_NAMES: &[&str] = &["before", "after", "final", "initial", "trace"];

#[derive(Debug, Error)]
pub enum FuzzError {
    #[error("invalid Celeste Fuzz specification: {0}")]
    Spec(String),
    #[error("candidate limit exceeded: {count} candidates (limit {limit})")]
    CandidateLimit { count: u64, limit: u64 },
    #[error("expression error in {phase}{input}: {bindings}: {message}")]
    Expression {
        phase: String,
        input: String,
        bindings: String,
        message: String,
    },
    #[error("simulation failed: {0}")]
    Simulation(#[from] celeste_physics::SimulationError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NumberExpression {
    Integer(i64),
    Expression(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldTime {
    Infinite,
    Frames(i64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JumpHoldPolicy {
    /// Celeste's `VarJumpTime` is 0.2 seconds, or 12 native frames.
    #[default]
    MaxEffect,
}

impl JumpHoldPolicy {
    pub const fn frames(self) -> u32 {
        match self {
            Self::MaxEffect => 12,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    pub(crate) from: NumberExpression,
    pub(crate) to: NumberExpression,
    pub step: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Left,
    Right,
    Up,
    Down,
    Dash,
    CrouchDash,
    Jump,
    Grab,
}

impl Key {
    pub(crate) fn parse(value: &str) -> Result<Self, FuzzError> {
        match value {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "dash" => Ok(Self::Dash),
            "crouch_dash" => Ok(Self::CrouchDash),
            "jump" => Ok(Self::Jump),
            "grab" => Ok(Self::Grab),
            _ => Err(FuzzError::Spec(format!("unknown input key `{value}`"))),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
            Self::Dash => "dash",
            Self::CrouchDash => "crouch_dash",
            Self::Jump => "jump",
            Self::Grab => "grab",
        }
    }

    pub(crate) const fn is_direction(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Up | Self::Down)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputDeclaration {
    pub(crate) keys: Vec<Key>,
    pub(crate) at: NumberExpression,
    pub(crate) held_time: Option<NumberExpression>,
    pub(crate) hold_infinite: bool,
    pub(crate) before_input: Vec<String>,
    pub(crate) after_input: Vec<String>,
    pub verify: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectiveKind {
    Maximize,
    Minimize,
    Approach { target: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub expression: String,
}

/// An objective sampled immediately after the input frame at `at`.
#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    pub(crate) at: NumberExpression,
    pub objectives: Vec<Objective>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    Best,
    Windows,
    Coverage,
    /// Internal consumer output used by the training runtime.  Unlike `top`,
    /// this preserves every successful candidate so a live input can filter
    /// the feasible set without rerunning the search.
    Candidates,
    /// Every fully simulated candidate, including candidates that failed the
    /// final success expressions. Used by training objective timelines.
    Evaluations,
    Top(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchSpec {
    pub bindings: BTreeMap<String, i64>,
    pub output: Vec<OutputMode>,
}

impl Default for SearchSpec {
    fn default() -> Self {
        Self {
            bindings: BTreeMap::new(),
            output: vec![OutputMode::Best, OutputMode::Windows],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_candidates: u64,
    pub max_input_frames: u32,
    pub max_trie_nodes: u64,
    pub max_cache_bytes: u64,
    pub max_expression_operations: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_candidates: 1_000_000,
            max_input_frames: 600,
            max_trie_nodes: 5_000_000,
            max_cache_bytes: 536_870_912,
            max_expression_operations: 10_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuzzSpec {
    pub version: u32,
    pub variables: Vec<Variable>,
    pub inputs: Vec<InputDeclaration>,
    pub(crate) observe_until: NumberExpression,
    pub(crate) success: Vec<String>,
    pub checkpoints: Vec<Checkpoint>,
    pub objectives: Vec<Objective>,
    pub search: SearchSpec,
    pub limits: Limits,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// Tighter per-call cap for candidates. `None` uses the configuration cap.
    pub max_candidates: Option<u64>,
    /// Tighter per-call cap for prefix-cache nodes.
    pub max_trie_nodes: Option<u64>,
    /// Tighter per-call cap for estimated prefix-cache bytes.
    pub max_cache_bytes: Option<u64>,
    /// Requested Rayon worker count when the `parallel` feature is enabled.
    pub workers: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchStats {
    pub candidate_count: u64,
    pub successful_count: u64,
    pub pruned_before: u64,
    pub pruned_after: u64,
    pub naive_frame_count: u64,
    pub unique_simulated_frames: u64,
    pub saved_frames: u64,
    pub trie_nodes: u64,
    pub peak_cache_bytes: u64,
    pub rhai_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerifiedInput {
    pub input_index: usize,
    pub frame: u32,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateResult {
    pub bindings: BTreeMap<String, i64>,
    pub final_state: PlayerSnapshot,
    pub objective_values: Vec<f64>,
    pub verified_inputs: Vec<VerifiedInput>,
    pub successful: bool,
    #[serde(skip)]
    pub(crate) tuple: Vec<i64>,
}

pub type FrameInterval = (i64, i64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactWindow {
    pub prefix: BTreeMap<String, i64>,
    pub last_variable: String,
    pub intervals: Vec<FrameInterval>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionSummary {
    pub bounds: BTreeMap<String, FrameInterval>,
    pub successful_count: u64,
    pub density: f64,
    pub best: CandidateResult,
}

/// Backwards-friendly spelling for callers that use the prose name.
pub type ConnectedRegion = RegionSummary;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoverageEntry {
    pub prefix: BTreeMap<String, i64>,
    pub successful_count: u64,
    pub successful_width: u64,
    pub coverage_percent: f64,
    pub intervals: Vec<FrameInterval>,
    pub best: Option<CandidateResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub covered_variable: String,
    pub reference: String,
    pub entries: Vec<CoverageEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FuzzResult {
    pub best: Option<CandidateResult>,
    pub top: Vec<CandidateResult>,
    pub candidates: Vec<CandidateResult>,
    pub evaluations: Vec<CandidateResult>,
    pub exact_windows: Vec<ExactWindow>,
    pub connected_regions: Vec<RegionSummary>,
    pub coverage_report: Option<CoverageReport>,
    pub stats: SearchStats,
}

impl FuzzSpec {
    pub(crate) fn parse(json: &str) -> Result<Self, FuzzError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|error| FuzzError::Spec(format!("invalid JSON: {error}")))?;
        let root = object(&value, "top-level value")?;
        let version = required_i64(root, "version")?;
        if version != 1 {
            return Err(FuzzError::Spec(format!(
                "unsupported version {version}; expected 1"
            )));
        }
        let variables = match root.get("variables") {
            Some(value) => array(value, "variables")?
                .iter()
                .enumerate()
                .map(|(index, value)| parse_variable(value, index))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        validate_variables(&variables)?;
        let inputs = match root.get("inputs") {
            Some(value) => array(value, "inputs")?
                .iter()
                .enumerate()
                .map(|(index, value)| parse_input(value, index))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let observe_until =
            parse_number_expression(required(root, "observe_until")?, "observe_until")?;
        let success = match root.get("success") {
            Some(value) => expressions(value, "success")?,
            None => Vec::new(),
        };
        let checkpoints = match root.get("checkpoints") {
            Some(value) => array(value, "checkpoints")?
                .iter()
                .enumerate()
                .map(|(index, value)| parse_checkpoint(value, index))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let objectives = match root.get("objectives") {
            Some(value) => array(value, "objectives")?
                .iter()
                .enumerate()
                .map(|(index, value)| parse_objective(value, index))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let search = root
            .get("search")
            .map(parse_search)
            .transpose()?
            .unwrap_or_default();
        let limits = root
            .get("limits")
            .map(parse_limits)
            .transpose()?
            .unwrap_or_default();
        if limits.max_candidates == 0
            || limits.max_input_frames == 0
            || limits.max_trie_nodes == 0
            || limits.max_cache_bytes == 0
            || limits.max_expression_operations == 0
        {
            return Err(FuzzError::Spec(
                "all limits must be greater than zero".into(),
            ));
        }
        Ok(Self {
            version: version as u32,
            variables,
            inputs,
            observe_until,
            success,
            checkpoints,
            objectives,
            search,
            limits,
        })
    }
}

fn parse_variable(value: &Value, index: usize) -> Result<Variable, FuzzError> {
    let fields = object(value, &format!("variables[{index}]"))?;
    let name = required_string(fields, "name")?.to_owned();
    let range = object(
        required(fields, "range")?,
        &format!("variables[{index}].range"),
    )?;
    let from = parse_number_expression(required(range, "from")?, "range.from")?;
    let to = parse_number_expression(required(range, "to")?, "range.to")?;
    let step = range
        .get("step")
        .map(|value| integer(value, "range.step"))
        .transpose()?
        .unwrap_or(1);
    if step <= 0 {
        return Err(FuzzError::Spec(format!(
            "variable `{name}` has a non-positive step"
        )));
    }
    Ok(Variable {
        name,
        range: Range { from, to, step },
    })
}

fn parse_input(value: &Value, index: usize) -> Result<InputDeclaration, FuzzError> {
    let object = object(value, &format!("inputs[{index}]"))?;
    let keys = array(required(object, "keys")?, "input.keys")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| FuzzError::Spec("input keys must be strings".into()))
                .and_then(Key::parse)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if keys.is_empty() {
        return Err(FuzzError::Spec(format!("inputs[{index}] has no keys")));
    }
    if keys
        .iter()
        .enumerate()
        .any(|(position, key)| keys[..position].contains(key))
    {
        return Err(FuzzError::Spec(format!(
            "inputs[{index}] contains a duplicate key"
        )));
    }
    let at = parse_number_expression(required(object, "at")?, "input.at")?;
    let held_value = object.get("held_time");
    let hold_infinite = held_value.and_then(Value::as_str) == Some("hold::inf");
    let held_time = match held_value {
        Some(value) if !hold_infinite => Some(parse_number_expression(value, "input.held_time")?),
        _ => None,
    };
    let has_direction = keys.iter().any(|key| key.is_direction());
    let has_dash = keys
        .iter()
        .any(|key| matches!(key, Key::Dash | Key::CrouchDash));
    if has_direction && held_value.is_none() {
        return Err(FuzzError::Spec(format!(
            "inputs[{index}] uses a direction key but has no held_time"
        )));
    }
    // A combined directional dash such as right+down+dash needs a held
    // direction while Dash itself remains a one-frame edge.  The resolver
    // applies holds only to directional keys, so this is unambiguous.
    if has_dash && held_value.is_some() && !has_direction {
        return Err(FuzzError::Spec(format!(
            "inputs[{index}] uses dash/crouch_dash, which cannot have held_time"
        )));
    }
    let before_input = object
        .get("before_input")
        .map(|value| expressions(value, "before_input"))
        .transpose()?
        .unwrap_or_default();
    let after_input = object
        .get("after_input")
        .map(|value| expressions(value, "after_input"))
        .transpose()?
        .unwrap_or_default();
    let verify = object
        .get("verify")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Ok(InputDeclaration {
        keys,
        at,
        held_time,
        hold_infinite,
        before_input,
        after_input,
        verify,
    })
}

fn parse_objective(value: &Value, index: usize) -> Result<Objective, FuzzError> {
    let object = object(value, &format!("objectives[{index}]"))?;
    let expression = required_string(object, "expression")?.to_owned();
    let kind = match required_string(object, "type")? {
        "maximize" => ObjectiveKind::Maximize,
        "minimize" => ObjectiveKind::Minimize,
        "approach" => {
            let target = number(required(object, "target")?, "objective.target")?;
            if !target.is_finite() {
                return Err(FuzzError::Spec("objective target must be finite".into()));
            }
            ObjectiveKind::Approach { target }
        }
        other => return Err(FuzzError::Spec(format!("unknown objective type `{other}`"))),
    };
    Ok(Objective { kind, expression })
}

fn parse_checkpoint(value: &Value, index: usize) -> Result<Checkpoint, FuzzError> {
    let fields = object(value, &format!("checkpoints[{index}]"))?;
    let at = parse_number_expression(required(fields, "at")?, &format!("checkpoints[{index}].at"))?;
    let objectives = array(
        required(fields, "objectives")?,
        &format!("checkpoints[{index}].objectives"),
    )?
    .iter()
    .enumerate()
    .map(|(objective_index, value)| parse_objective(value, objective_index))
    .collect::<Result<Vec<_>, _>>()?;
    if objectives.is_empty() {
        return Err(FuzzError::Spec(format!(
            "checkpoints[{index}] has no objectives"
        )));
    }
    Ok(Checkpoint { at, objectives })
}

fn parse_search(value: &Value) -> Result<SearchSpec, FuzzError> {
    let fields = object(value, "search")?;
    let bindings = match fields.get("bindings") {
        Some(value) => object(value, "search.bindings")?
            .iter()
            .map(|(name, value)| Ok((name.clone(), integer(value, "search binding")?)))
            .collect::<Result<BTreeMap<_, _>, FuzzError>>()?,
        None => BTreeMap::new(),
    };
    let output = match fields.get("output") {
        Some(value) => array(value, "search.output")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| FuzzError::Spec("search output values must be strings".into()))
                    .and_then(parse_output)
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => SearchSpec::default().output,
    };
    Ok(SearchSpec { bindings, output })
}

fn parse_output(value: &str) -> Result<OutputMode, FuzzError> {
    match value {
        "best" => Ok(OutputMode::Best),
        "windows" => Ok(OutputMode::Windows),
        "coverage" => Ok(OutputMode::Coverage),
        "candidates" => Ok(OutputMode::Candidates),
        "evaluations" => Ok(OutputMode::Evaluations),
        _ => value
            .strip_prefix("top_")
            .ok_or_else(|| FuzzError::Spec(format!("unknown search output `{value}`")))
            .and_then(|count| {
                count
                    .parse::<usize>()
                    .ok()
                    .filter(|count| *count > 0)
                    .map(OutputMode::Top)
                    .ok_or_else(|| FuzzError::Spec(format!("invalid top output `{value}`")))
            }),
    }
}

fn parse_limits(value: &Value) -> Result<Limits, FuzzError> {
    let object = object(value, "limits")?;
    let defaults = Limits::default();
    Ok(Limits {
        max_candidates: optional_u64(object, "max_candidates", defaults.max_candidates)?,
        max_input_frames: optional_u64(
            object,
            "max_input_frames",
            defaults.max_input_frames as u64,
        )?
        .try_into()
        .map_err(|_| FuzzError::Spec("max_input_frames is too large".into()))?,
        max_trie_nodes: optional_u64(object, "max_trie_nodes", defaults.max_trie_nodes)?,
        max_cache_bytes: optional_u64(object, "max_cache_bytes", defaults.max_cache_bytes)?,
        max_expression_operations: optional_u64(
            object,
            "max_expression_operations",
            defaults.max_expression_operations,
        )?,
    })
}

fn validate_variables(variables: &[Variable]) -> Result<(), FuzzError> {
    let mut declared = Vec::new();
    for variable in variables {
        if RESERVED_NAMES.contains(&variable.name.as_str()) {
            return Err(FuzzError::Spec(format!(
                "`{}` is a reserved variable name",
                variable.name
            )));
        }
        if !valid_identifier(&variable.name) {
            return Err(FuzzError::Spec(format!(
                "`{}` is not a valid variable name",
                variable.name
            )));
        }
        if declared.contains(&variable.name) {
            return Err(FuzzError::Spec(format!(
                "duplicate variable `{}`",
                variable.name
            )));
        }
        for expression in [&variable.range.from, &variable.range.to] {
            if let NumberExpression::Expression(source) = expression {
                for name in identifier_words(source) {
                    if declared.contains(&name) || is_builtin_word(&name) {
                        continue;
                    }
                    // Rhai can report this later, but this message identifies
                    // the dependency-order issue at configuration load time.
                    if variables.iter().any(|candidate| candidate.name == name) {
                        return Err(FuzzError::Spec(format!(
                            "variable `{}` references later variable `{name}`",
                            variable.name
                        )));
                    }
                }
            }
        }
        declared.push(variable.name.clone());
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(character) if character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn identifier_words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn is_builtin_word(word: &str) -> bool {
    matches!(
        word,
        "abs" | "min" | "max" | "true" | "false" | "hold" | "inf"
    ) || word.chars().all(|character| character.is_ascii_digit())
}

fn expressions(value: &Value, field: &str) -> Result<Vec<String>, FuzzError> {
    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| FuzzError::Spec(format!("{field} must contain only strings")))
            })
            .collect(),
        _ => Err(FuzzError::Spec(format!(
            "{field} must be a string or array of strings"
        ))),
    }
}

fn parse_number_expression(value: &Value, field: &str) -> Result<NumberExpression, FuzzError> {
    match value {
        Value::Number(_) => Ok(NumberExpression::Integer(integer(value, field)?)),
        Value::String(value) if !value.trim().is_empty() => {
            Ok(NumberExpression::Expression(value.clone()))
        }
        _ => Err(FuzzError::Spec(format!(
            "{field} must be an integer or expression string"
        ))),
    }
}

fn object<'a>(value: &'a Value, field: &str) -> Result<&'a JsonMap<String, Value>, FuzzError> {
    value
        .as_object()
        .ok_or_else(|| FuzzError::Spec(format!("{field} must be an object")))
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], FuzzError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| FuzzError::Spec(format!("{field} must be an array")))
}

fn required<'a>(object: &'a JsonMap<String, Value>, field: &str) -> Result<&'a Value, FuzzError> {
    object
        .get(field)
        .ok_or_else(|| FuzzError::Spec(format!("missing required field `{field}`")))
}

fn required_string<'a>(
    object: &'a JsonMap<String, Value>,
    field: &str,
) -> Result<&'a str, FuzzError> {
    required(object, field)?
        .as_str()
        .ok_or_else(|| FuzzError::Spec(format!("{field} must be a string")))
}

fn required_i64(object: &JsonMap<String, Value>, field: &str) -> Result<i64, FuzzError> {
    integer(required(object, field)?, field)
}

fn integer(value: &Value, field: &str) -> Result<i64, FuzzError> {
    value
        .as_i64()
        .ok_or_else(|| FuzzError::Spec(format!("{field} must be an i64 integer")))
}

fn optional_u64(
    object: &JsonMap<String, Value>,
    field: &str,
    default: u64,
) -> Result<u64, FuzzError> {
    match object.get(field) {
        Some(value) => integer(value, field)?
            .try_into()
            .map_err(|_| FuzzError::Spec(format!("{field} must be non-negative"))),
        None => Ok(default),
    }
}

fn number(value: &Value, field: &str) -> Result<f64, FuzzError> {
    value
        .as_f64()
        .ok_or_else(|| FuzzError::Spec(format!("{field} must be numeric")))
}
