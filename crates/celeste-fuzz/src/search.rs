use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    mem::size_of,
};

use celeste_physics::{InputState, Map, PlayerSnapshot, Simulator};
use rhai::{Dynamic, Engine};

use crate::model::{Key, NumberExpression};
use crate::{
    CandidateResult, CoverageEntry, CoverageReport, ExactWindow, FrameInterval, FuzzError,
    FuzzResult, FuzzSpec, HoldTime, InputDeclaration, JumpHoldPolicy, ObjectiveKind, OutputMode,
    RegionSummary, SearchOptions, SearchStats, Variable, VerifiedInput,
    engine::{CompiledExpression, ExpressionContext, build_engine, compile_expression, evaluate},
};

/// Parse a version-1 Celeste Fuzz JSON document.
pub fn parse_spec(json: &str) -> Result<FuzzSpec, FuzzError> {
    FuzzSpec::parse(json)
}

/// Compile expressions and input declarations once before searching.
pub fn compile(spec: FuzzSpec) -> Result<CompiledFuzz, FuzzError> {
    CompiledFuzz::new(spec)
}

pub struct CompiledFuzz {
    spec: FuzzSpec,
    variables: Vec<CompiledVariable>,
    inputs: Vec<CompiledInput>,
    observe_until: CompiledNumber,
    success: Vec<CompiledExpression>,
    objectives: Vec<CompiledObjective>,
    estimated_candidates: u64,
}

#[derive(Clone)]
struct CompiledVariable {
    name: String,
    from: CompiledNumber,
    to: CompiledNumber,
    step: i64,
}

#[derive(Clone)]
struct CompiledInput {
    keys: Vec<Key>,
    at: CompiledNumber,
    held_time: Option<CompiledNumber>,
    hold_infinite: bool,
    before_input: Vec<CompiledExpression>,
    after_input: Vec<CompiledExpression>,
    verify: bool,
}

#[derive(Clone)]
struct CompiledObjective {
    kind: ObjectiveKind,
    expression: CompiledExpression,
}

#[derive(Clone)]
enum CompiledNumber {
    Integer(i64),
    Expression(CompiledExpression),
}

#[derive(Clone)]
struct ResolvedCandidate {
    variables: BTreeMap<String, i64>,
    tuple: Vec<i64>,
    observe_until: u32,
    inputs: Vec<InputState>,
    checkpoints: BTreeMap<u32, Vec<usize>>,
    metadata: Vec<ResolvedInputMetadata>,
    verified_inputs: Vec<VerifiedInput>,
    rhai_evaluations: u64,
}

#[derive(Clone, Copy)]
struct ResolvedInputMetadata {
    at: u32,
    held_time: Option<HoldTime>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct InputKey {
    move_x: i8,
    move_y: i8,
    jump_pressed: bool,
    jump_held: bool,
    dash_pressed: bool,
    crouch_dash_pressed: bool,
    grab_held: bool,
    talk_pressed: bool,
}

impl From<InputState> for InputKey {
    fn from(input: InputState) -> Self {
        Self {
            move_x: input.move_x,
            move_y: input.move_y,
            jump_pressed: input.jump_pressed,
            jump_held: input.jump_held,
            dash_pressed: input.dash_pressed,
            crouch_dash_pressed: input.crouch_dash_pressed,
            grab_held: input.grab_held,
            talk_pressed: input.talk_pressed,
        }
    }
}

struct TrieNode {
    children: BTreeMap<InputKey, usize>,
    cached: Option<Simulator>,
}

struct PrefixTrie {
    nodes: Vec<TrieNode>,
    max_nodes: u64,
    max_cache_bytes: u64,
    cache_bytes: u64,
    peak_cache_bytes: u64,
}

impl PrefixTrie {
    // A `Simulator` owns cloneable entity runtime vectors and a map.  Rust's
    // shallow `size_of` cannot see their capacities; a conservative 64 KiB
    // accounting unit makes the configured byte cap protective even on a
    // dense room.  It also bounds forked map retention independently from the
    // number of trie node headers.
    const CACHE_ENTRY_BYTES: u64 = 64 * 1024;

    fn new(root: Simulator, max_nodes: u64, max_cache_bytes: u64) -> Self {
        let root_cost = Self::CACHE_ENTRY_BYTES;
        Self {
            nodes: vec![TrieNode {
                children: BTreeMap::new(),
                cached: Some(root),
            }],
            max_nodes,
            max_cache_bytes,
            cache_bytes: root_cost.min(max_cache_bytes),
            peak_cache_bytes: root_cost.min(max_cache_bytes),
        }
    }

    fn root(&self) -> Simulator {
        self.nodes[0]
            .cached
            .as_ref()
            .expect("prefix trie always retains root simulator")
            .fork()
    }

    fn cached_at(&self, inputs: &[InputState], depth: u32) -> Option<Simulator> {
        let mut node = 0usize;
        for input in &inputs[..depth as usize] {
            node = *self.nodes[node].children.get(&InputKey::from(*input))?;
        }
        self.nodes[node].cached.as_ref().map(Simulator::fork)
    }

    fn cache(&mut self, prefix: &[InputState], simulator: &Simulator) {
        if prefix.is_empty()
            || self.cache_bytes.saturating_add(Self::CACHE_ENTRY_BYTES) > self.max_cache_bytes
        {
            return;
        }
        let mut node = 0usize;
        for input in prefix {
            let key = InputKey::from(*input);
            let next = if let Some(next) = self.nodes[node].children.get(&key) {
                *next
            } else {
                if self.nodes.len() as u64 >= self.max_nodes {
                    return;
                }
                let next = self.nodes.len();
                self.nodes.push(TrieNode {
                    children: BTreeMap::new(),
                    cached: None,
                });
                self.nodes[node].children.insert(key, next);
                next
            };
            node = next;
        }
        if self.nodes[node].cached.is_none() {
            self.nodes[node].cached = Some(simulator.fork());
            self.cache_bytes = self.cache_bytes.saturating_add(Self::CACHE_ENTRY_BYTES);
            self.peak_cache_bytes = self.peak_cache_bytes.max(self.cache_bytes);
        }
    }
}

impl CompiledFuzz {
    fn new(spec: FuzzSpec) -> Result<Self, FuzzError> {
        let engine = build_engine(spec.limits.max_expression_operations);
        let mut cache = HashMap::new();
        let variables = spec
            .variables
            .iter()
            .map(|variable| compile_variable(variable, &engine, &mut cache))
            .collect::<Result<Vec<_>, _>>()?;
        let inputs = spec
            .inputs
            .iter()
            .map(|input| compile_input(input, &engine, &mut cache))
            .collect::<Result<Vec<_>, _>>()?;
        let observe_until = compile_number(&spec.observe_until, &engine, &mut cache)?;
        let success = spec
            .success
            .iter()
            .map(|source| compile_cached(source, &engine, &mut cache))
            .collect::<Result<Vec<_>, _>>()?;
        let objectives = spec
            .objectives
            .iter()
            .map(|objective| {
                Ok(CompiledObjective {
                    kind: objective.kind.clone(),
                    expression: compile_cached(&objective.expression, &engine, &mut cache)?,
                })
            })
            .collect::<Result<Vec<_>, FuzzError>>()?;
        let compiled = Self {
            spec,
            variables,
            inputs,
            observe_until,
            success,
            objectives,
            estimated_candidates: 0,
        };
        compiled.validate_configured_binding_names()?;
        let estimate_engine = build_engine(compiled.spec.limits.max_expression_operations);
        let estimated_candidates = compiled.count_bindings(
            &estimate_engine,
            &BTreeMap::new(),
            compiled.spec.limits.max_candidates,
        )?;
        Ok(Self {
            estimated_candidates,
            ..compiled
        })
    }

    pub fn spec(&self) -> &FuzzSpec {
        &self.spec
    }

    pub fn estimate_candidates(&self) -> u64 {
        self.estimated_candidates
    }

    pub fn search(
        &self,
        initial: PlayerSnapshot,
        map: &Map,
        runtime_bindings: HashMap<String, i64>,
        output: Vec<OutputMode>,
        options: SearchOptions,
    ) -> Result<FuzzResult, FuzzError> {
        let output = if output.is_empty() {
            self.spec.search.output.clone()
        } else {
            output
        };
        let mut bindings = self.spec.search.bindings.clone();
        bindings.extend(bindings_from_hash_map(runtime_bindings));
        self.search_with_bindings(initial, map, bindings, output, options)
    }

    fn search_with_bindings(
        &self,
        initial: PlayerSnapshot,
        map: &Map,
        bindings: BTreeMap<String, i64>,
        output: Vec<OutputMode>,
        options: SearchOptions,
    ) -> Result<FuzzResult, FuzzError> {
        self.validate_runtime_binding_names(&bindings)?;
        let max_candidates = min_limit(options.max_candidates, self.spec.limits.max_candidates);
        let max_nodes = min_limit(options.max_trie_nodes, self.spec.limits.max_trie_nodes);
        let max_cache_bytes = min_limit(options.max_cache_bytes, self.spec.limits.max_cache_bytes);
        let engine = build_engine(self.spec.limits.max_expression_operations);
        let all_bindings = self.enumerate_bindings(&engine, &bindings, max_candidates)?;
        let root = Simulator::new(initial.clone(), map)?;
        let mut trie = PrefixTrie::new(root, max_nodes, max_cache_bytes);
        let mut stats = SearchStats {
            candidate_count: all_bindings.len() as u64,
            ..SearchStats::default()
        };
        let mut successful = Vec::new();
        let mut all_tuples = Vec::with_capacity(all_bindings.len());
        for candidate_bindings in all_bindings {
            let tuple = self.tuple(&candidate_bindings);
            all_tuples.push(tuple.clone());
            let candidate = self.resolve_candidate(&engine, candidate_bindings, tuple)?;
            stats.rhai_evaluations = stats
                .rhai_evaluations
                .saturating_add(candidate.rhai_evaluations);
            stats.naive_frame_count = stats
                .naive_frame_count
                .saturating_add(candidate.observe_until as u64);
            if let Some(result) =
                self.simulate_candidate(&engine, &initial, &candidate, &mut trie, &mut stats)?
            {
                stats.successful_count += 1;
                successful.push(result);
            }
        }
        stats.saved_frames = stats
            .naive_frame_count
            .saturating_sub(stats.unique_simulated_frames);
        stats.trie_nodes = trie.nodes.len() as u64;
        stats.peak_cache_bytes = trie.peak_cache_bytes;
        Ok(self.build_result(successful, all_tuples, stats, output))
    }

    fn build_result(
        &self,
        mut successful: Vec<CandidateResult>,
        all_tuples: Vec<Vec<i64>>,
        stats: SearchStats,
        output: Vec<OutputMode>,
    ) -> FuzzResult {
        successful.sort_by(|left, right| self.compare_candidates(left, right));
        let wants_best = output.iter().any(|mode| matches!(mode, OutputMode::Best));
        let wants_windows = output
            .iter()
            .any(|mode| matches!(mode, OutputMode::Windows));
        let wants_coverage = output
            .iter()
            .any(|mode| matches!(mode, OutputMode::Coverage));
        let wants_candidates = output
            .iter()
            .any(|mode| matches!(mode, OutputMode::Candidates));
        let top_count = output
            .iter()
            .filter_map(|mode| match mode {
                OutputMode::Top(count) => Some(*count),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let exact_windows = if wants_windows {
            self.exact_windows(&successful)
        } else {
            Vec::new()
        };
        let connected_regions = if wants_windows {
            self.connected_regions(&successful)
        } else {
            Vec::new()
        };
        let coverage_report =
            wants_coverage.then(|| self.coverage_report(&all_tuples, &successful));
        let candidates = wants_candidates.then(|| successful.clone()).unwrap_or_default();
        FuzzResult {
            best: wants_best.then(|| successful.first().cloned()).flatten(),
            top: successful.into_iter().take(top_count).collect(),
            candidates,
            exact_windows,
            connected_regions,
            coverage_report,
            stats,
        }
    }

    #[cfg(feature = "parallel")]
    pub fn search_parallel(
        &self,
        initial: PlayerSnapshot,
        map: &Map,
        runtime_bindings: HashMap<String, i64>,
        output: Vec<OutputMode>,
        options: SearchOptions,
    ) -> Result<FuzzResult, FuzzError> {
        use rayon::prelude::*;

        let output = if output.is_empty() {
            self.spec.search.output.clone()
        } else {
            output
        };
        let mut bindings = self.spec.search.bindings.clone();
        bindings.extend(bindings_from_hash_map(runtime_bindings));
        self.validate_runtime_binding_names(&bindings)?;
        let max_candidates = min_limit(options.max_candidates, self.spec.limits.max_candidates);
        let max_nodes = min_limit(options.max_trie_nodes, self.spec.limits.max_trie_nodes);
        let max_cache_bytes = min_limit(options.max_cache_bytes, self.spec.limits.max_cache_bytes);
        let engine = build_engine(self.spec.limits.max_expression_operations);
        let all_bindings = self.enumerate_bindings(&engine, &bindings, max_candidates)?;
        let all_tuples = all_bindings
            .iter()
            .map(|bindings| self.tuple(bindings))
            .collect::<Vec<_>>();
        let shards = self.split_parallel_shards(all_bindings);
        let mut builder = rayon::ThreadPoolBuilder::new();
        if let Some(workers) = options.workers.filter(|workers| *workers > 0) {
            builder = builder.num_threads(workers);
        }
        let pool = builder
            .build()
            .map_err(|error| FuzzError::Spec(format!("cannot create Rayon pool: {error}")))?;
        let results = pool.install(|| {
            shards
                .into_par_iter()
                .map(|shard| self.simulate_shard(&initial, map, shard, max_nodes, max_cache_bytes))
                .collect::<Result<Vec<_>, _>>()
        })?;
        let mut successful = Vec::new();
        let mut stats = SearchStats::default();
        for (shard_successes, shard_stats) in results {
            successful.extend(shard_successes);
            add_stats(&mut stats, shard_stats);
        }
        Ok(self.build_result(successful, all_tuples, stats, output))
    }

    #[cfg(feature = "parallel")]
    fn split_parallel_shards(
        &self,
        bindings: Vec<BTreeMap<String, i64>>,
    ) -> Vec<Vec<BTreeMap<String, i64>>> {
        let Some(variable) = self.variables.iter().find(|variable| {
            bindings
                .iter()
                .map(|bindings| {
                    *bindings
                        .get(&variable.name)
                        .expect("enumerated binding contains variable")
                })
                .collect::<BTreeSet<i64>>()
                .len()
                > 1
        }) else {
            return vec![bindings];
        };
        let mut shards = BTreeMap::<i64, Vec<BTreeMap<String, i64>>>::new();
        for binding in bindings {
            let value = *binding
                .get(&variable.name)
                .expect("enumerated binding contains split variable");
            shards.entry(value).or_default().push(binding);
        }
        shards.into_values().collect()
    }

    #[cfg(feature = "parallel")]
    fn simulate_shard(
        &self,
        initial: &PlayerSnapshot,
        map: &Map,
        bindings: Vec<BTreeMap<String, i64>>,
        max_nodes: u64,
        max_cache_bytes: u64,
    ) -> Result<(Vec<CandidateResult>, SearchStats), FuzzError> {
        let engine = build_engine(self.spec.limits.max_expression_operations);
        let root = Simulator::new(initial.clone(), map)?;
        let mut trie = PrefixTrie::new(root, max_nodes, max_cache_bytes);
        let mut stats = SearchStats {
            candidate_count: bindings.len() as u64,
            ..SearchStats::default()
        };
        let mut successful = Vec::new();
        for candidate_bindings in bindings {
            let tuple = self.tuple(&candidate_bindings);
            let candidate = self.resolve_candidate(&engine, candidate_bindings, tuple)?;
            stats.rhai_evaluations = stats
                .rhai_evaluations
                .saturating_add(candidate.rhai_evaluations);
            stats.naive_frame_count = stats
                .naive_frame_count
                .saturating_add(candidate.observe_until as u64);
            if let Some(result) =
                self.simulate_candidate(&engine, initial, &candidate, &mut trie, &mut stats)?
            {
                stats.successful_count += 1;
                successful.push(result);
            }
        }
        stats.saved_frames = stats
            .naive_frame_count
            .saturating_sub(stats.unique_simulated_frames);
        stats.trie_nodes = trie.nodes.len() as u64;
        stats.peak_cache_bytes = trie.peak_cache_bytes;
        Ok((successful, stats))
    }

    fn validate_configured_binding_names(&self) -> Result<(), FuzzError> {
        self.validate_runtime_binding_names(&self.spec.search.bindings)
    }

    fn validate_runtime_binding_names(
        &self,
        bindings: &BTreeMap<String, i64>,
    ) -> Result<(), FuzzError> {
        for name in bindings.keys() {
            if !self.variables.iter().any(|variable| variable.name == *name) {
                return Err(FuzzError::Spec(format!(
                    "binding references unknown variable `{name}`"
                )));
            }
        }
        Ok(())
    }

    fn count_bindings(
        &self,
        engine: &Engine,
        bindings: &BTreeMap<String, i64>,
        limit: u64,
    ) -> Result<u64, FuzzError> {
        let mut count = 0u64;
        let mut current = BTreeMap::new();
        self.enumerate_recursive(engine, 0, bindings, &mut current, &mut |_, _| {
            count = count.checked_add(1).ok_or(FuzzError::CandidateLimit {
                count: u64::MAX,
                limit,
            })?;
            if count > limit {
                return Err(FuzzError::CandidateLimit { count, limit });
            }
            Ok(())
        })?;
        Ok(count)
    }

    fn enumerate_bindings(
        &self,
        engine: &Engine,
        bindings: &BTreeMap<String, i64>,
        limit: u64,
    ) -> Result<Vec<BTreeMap<String, i64>>, FuzzError> {
        let mut result = Vec::new();
        let mut current = BTreeMap::new();
        self.enumerate_recursive(engine, 0, bindings, &mut current, &mut |current, _| {
            if result.len() as u64 >= limit {
                return Err(FuzzError::CandidateLimit {
                    count: result.len() as u64 + 1,
                    limit,
                });
            }
            result.push(current.clone());
            Ok(())
        })?;
        Ok(result)
    }

    fn enumerate_recursive(
        &self,
        engine: &Engine,
        index: usize,
        requested: &BTreeMap<String, i64>,
        current: &mut BTreeMap<String, i64>,
        visit: &mut impl FnMut(&BTreeMap<String, i64>, &[i64]) -> Result<(), FuzzError>,
    ) -> Result<(), FuzzError> {
        if index == self.variables.len() {
            let tuple = self.tuple(current);
            return visit(current, &tuple);
        }
        let variable = &self.variables[index];
        let from =
            self.eval_integer(engine, &variable.from, current, "range.from", None, &mut 0)?;
        let to = self.eval_integer(engine, &variable.to, current, "range.to", None, &mut 0)?;
        if from > to {
            return Err(FuzzError::Spec(format!(
                "variable `{}` resolves to an empty range [{from}, {to}]",
                variable.name
            )));
        }
        if let Some(value) = requested.get(&variable.name).copied() {
            if value < from || value > to || (value - from) % variable.step != 0 {
                return Err(FuzzError::Spec(format!(
                    "binding {}={value} is outside its resolved range [{from}, {to}] step {}",
                    variable.name, variable.step
                )));
            }
            current.insert(variable.name.clone(), value);
            self.enumerate_recursive(engine, index + 1, requested, current, visit)?;
            current.remove(&variable.name);
            return Ok(());
        }
        let mut value = from;
        loop {
            current.insert(variable.name.clone(), value);
            self.enumerate_recursive(engine, index + 1, requested, current, visit)?;
            current.remove(&variable.name);
            match value.checked_add(variable.step) {
                Some(next) if next <= to => value = next,
                _ => break,
            }
        }
        Ok(())
    }

    fn tuple(&self, bindings: &BTreeMap<String, i64>) -> Vec<i64> {
        self.variables
            .iter()
            .map(|variable| {
                *bindings
                    .get(&variable.name)
                    .expect("all variables are bound")
            })
            .collect()
    }

    fn resolve_candidate(
        &self,
        engine: &Engine,
        variables: BTreeMap<String, i64>,
        tuple: Vec<i64>,
    ) -> Result<ResolvedCandidate, FuzzError> {
        let mut expression_evaluations = 0;
        let observe = self.eval_integer(
            engine,
            &self.observe_until,
            &variables,
            "observe_until",
            None,
            &mut expression_evaluations,
        )?;
        let observe_until: u32 = observe.try_into().map_err(|_| {
            FuzzError::Spec("observe_until must be a non-negative u32 frame".into())
        })?;
        if observe_until > self.spec.limits.max_input_frames {
            return Err(FuzzError::Spec(format!(
                "observe_until {observe_until} exceeds max_input_frames {}",
                self.spec.limits.max_input_frames
            )));
        }
        let mut inputs = vec![InputState::default(); observe_until as usize];
        let mut left = vec![false; observe_until as usize];
        let mut right = vec![false; observe_until as usize];
        let mut up = vec![false; observe_until as usize];
        let mut down = vec![false; observe_until as usize];
        let mut checkpoints = BTreeMap::<u32, Vec<usize>>::new();
        let mut metadata = Vec::with_capacity(self.inputs.len());
        let mut verified_inputs = Vec::new();
        for (index, declaration) in self.inputs.iter().enumerate() {
            let at = self.eval_frame(
                engine,
                &declaration.at,
                &variables,
                "input.at",
                index,
                &mut expression_evaluations,
            )?;
            if at >= observe_until {
                return Err(FuzzError::Spec(format!(
                    "input {index} occurs at frame {at}, outside observe_until {observe_until}"
                )));
            }
            let finite_hold = declaration
                .held_time
                .as_ref()
                .map(|held| {
                    self.eval_integer(
                        engine,
                        held,
                        &variables,
                        "input.held_time",
                        Some(index),
                        &mut expression_evaluations,
                    )
                })
                .transpose()?;
            if matches!(finite_hold, Some(value) if value <= 0) {
                return Err(FuzzError::Spec(format!(
                    "input {index} has non-positive held_time"
                )));
            }
            let generic_hold = if declaration.hold_infinite {
                Some(HoldTime::Infinite)
            } else {
                finite_hold.map(HoldTime::Frames)
            };
            let effective_hold = generic_hold.or_else(|| {
                if declaration.keys.iter().any(|key| matches!(key, Key::Grab)) {
                    Some(HoldTime::Infinite)
                } else if declaration.keys.iter().any(|key| matches!(key, Key::Jump)) {
                    Some(HoldTime::Frames(JumpHoldPolicy::MaxEffect.frames() as i64))
                } else {
                    None
                }
            });
            metadata.push(ResolvedInputMetadata {
                at,
                held_time: effective_hold,
            });
            if declaration.verify {
                verified_inputs.push(VerifiedInput {
                    input_index: index,
                    frame: at,
                    keys: declaration
                        .keys
                        .iter()
                        .map(|key| key.name().to_owned())
                        .collect(),
                });
            }
            if !declaration.before_input.is_empty() || !declaration.after_input.is_empty() {
                checkpoints.entry(at).or_default().push(index);
            }
            for key in &declaration.keys {
                match key {
                    Key::Left => apply_direction(
                        &mut left,
                        at,
                        finite_hold,
                        declaration.hold_infinite,
                        observe_until,
                    )?,
                    Key::Right => apply_direction(
                        &mut right,
                        at,
                        finite_hold,
                        declaration.hold_infinite,
                        observe_until,
                    )?,
                    Key::Up => apply_direction(
                        &mut up,
                        at,
                        finite_hold,
                        declaration.hold_infinite,
                        observe_until,
                    )?,
                    Key::Down => apply_direction(
                        &mut down,
                        at,
                        finite_hold,
                        declaration.hold_infinite,
                        observe_until,
                    )?,
                    Key::Dash => inputs[at as usize].dash_pressed = true,
                    Key::CrouchDash => {
                        inputs[at as usize].dash_pressed = true;
                        inputs[at as usize].crouch_dash_pressed = true;
                    }
                    Key::Jump => {
                        let hold = if declaration.hold_infinite {
                            return Err(FuzzError::Spec(format!(
                                "input {index}: jump cannot use hold::inf; omit held_time for MaxEffect"
                            )));
                        } else {
                            finite_hold.unwrap_or(JumpHoldPolicy::MaxEffect.frames() as i64)
                        };
                        inputs[at as usize].jump_pressed = true;
                        apply_bool_hold(&mut inputs, at, hold, observe_until, |input| {
                            &mut input.jump_held
                        })?;
                    }
                    Key::Grab => {
                        let hold = if declaration.hold_infinite || finite_hold.is_none() {
                            None
                        } else {
                            finite_hold
                        };
                        apply_bool_hold(
                            &mut inputs,
                            at,
                            hold.unwrap_or((observe_until - at) as i64),
                            observe_until,
                            |input| &mut input.grab_held,
                        )?;
                    }
                }
            }
        }
        for frame in 0..observe_until as usize {
            inputs[frame].move_x = match (left[frame], right[frame]) {
                (true, false) => -1,
                (false, true) => 1,
                _ => 0,
            };
            inputs[frame].move_y = match (up[frame], down[frame]) {
                (true, false) => -1,
                (false, true) => 1,
                _ => 0,
            };
        }
        Ok(ResolvedCandidate {
            variables,
            tuple,
            observe_until,
            inputs,
            checkpoints,
            metadata,
            verified_inputs,
            rhai_evaluations: expression_evaluations,
        })
    }

    fn simulate_candidate(
        &self,
        engine: &Engine,
        initial: &PlayerSnapshot,
        candidate: &ResolvedCandidate,
        trie: &mut PrefixTrie,
        stats: &mut SearchStats,
    ) -> Result<Option<CandidateResult>, FuzzError> {
        let mut simulator = trie.root();
        let mut current = 0u32;
        for (&frame, declarations) in &candidate.checkpoints {
            self.advance_to(&mut simulator, trie, candidate, &mut current, frame, stats)?;
            for &input_index in declarations {
                if !self.evaluate_conditions(
                    engine,
                    &self.inputs[input_index].before_input,
                    "before_input",
                    input_index,
                    initial,
                    simulator.snapshot(),
                    None,
                    &candidate.variables,
                    candidate.metadata[input_index],
                    stats,
                )? {
                    stats.pruned_before += 1;
                    return Ok(None);
                }
            }
            let before_snapshot = simulator.snapshot().clone();
            simulator.step(candidate.inputs[frame as usize])?;
            stats.unique_simulated_frames += 1;
            current = frame + 1;
            trie.cache(&candidate.inputs[..current as usize], &simulator);
            for &input_index in declarations {
                if !self.evaluate_conditions(
                    engine,
                    &self.inputs[input_index].after_input,
                    "after_input",
                    input_index,
                    initial,
                    &before_snapshot,
                    Some(simulator.snapshot()),
                    &candidate.variables,
                    candidate.metadata[input_index],
                    stats,
                )? {
                    stats.pruned_after += 1;
                    return Ok(None);
                }
            }
        }
        self.advance_to(
            &mut simulator,
            trie,
            candidate,
            &mut current,
            candidate.observe_until,
            stats,
        )?;
        for expression in &self.success {
            stats.rhai_evaluations += 1;
            if !self.evaluate_bool(
                engine,
                expression,
                "success",
                None,
                &candidate.variables,
                ExpressionContext {
                    variables: &candidate.variables,
                    initial: Some(initial),
                    current: Some(simulator.snapshot()),
                    before: None,
                    after: None,
                    final_state: Some(simulator.snapshot()),
                    at: None,
                    held_time: None,
                    input_index: None,
                    verify: None,
                },
            )? {
                return Ok(None);
            }
        }
        let mut objective_values = Vec::with_capacity(self.objectives.len());
        for objective in &self.objectives {
            stats.rhai_evaluations += 1;
            let dynamic = self.evaluate_dynamic(
                engine,
                &objective.expression,
                "objective",
                None,
                &candidate.variables,
                ExpressionContext {
                    variables: &candidate.variables,
                    initial: Some(initial),
                    current: Some(simulator.snapshot()),
                    before: None,
                    after: None,
                    final_state: Some(simulator.snapshot()),
                    at: None,
                    held_time: None,
                    input_index: None,
                    verify: None,
                },
            )?;
            objective_values.push(dynamic_number(dynamic).ok_or_else(|| {
                self.expression_error(
                    "objective",
                    None,
                    &candidate.variables,
                    "objective result must be a finite number".into(),
                )
            })?);
        }
        Ok(Some(CandidateResult {
            bindings: candidate.variables.clone(),
            final_state: simulator.snapshot().clone(),
            objective_values,
            verified_inputs: candidate.verified_inputs.clone(),
            tuple: candidate.tuple.clone(),
        }))
    }

    fn advance_to(
        &self,
        simulator: &mut Simulator,
        trie: &mut PrefixTrie,
        candidate: &ResolvedCandidate,
        current: &mut u32,
        target: u32,
        stats: &mut SearchStats,
    ) -> Result<(), FuzzError> {
        debug_assert!(*current <= target);
        if *current < target
            && let Some(cached) = trie.cached_at(&candidate.inputs, target)
        {
            *simulator = cached;
            *current = target;
            return Ok(());
        }
        while *current < target {
            simulator.step(candidate.inputs[*current as usize])?;
            *current += 1;
            stats.unique_simulated_frames += 1;
            trie.cache(&candidate.inputs[..*current as usize], simulator);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_conditions(
        &self,
        engine: &Engine,
        expressions: &[CompiledExpression],
        phase: &str,
        input_index: usize,
        initial: &PlayerSnapshot,
        before: &PlayerSnapshot,
        after: Option<&PlayerSnapshot>,
        variables: &BTreeMap<String, i64>,
        metadata: ResolvedInputMetadata,
        stats: &mut SearchStats,
    ) -> Result<bool, FuzzError> {
        for expression in expressions {
            stats.rhai_evaluations += 1;
            if !self.evaluate_bool(
                engine,
                expression,
                phase,
                Some(input_index),
                variables,
                ExpressionContext {
                    variables,
                    initial: Some(initial),
                    current: after.or(Some(before)),
                    before: Some(before),
                    after,
                    final_state: None,
                    at: Some(metadata.at as i64),
                    held_time: metadata.held_time,
                    input_index: Some(input_index),
                    verify: Some(self.inputs[input_index].verify),
                },
            )? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_bool(
        &self,
        engine: &Engine,
        expression: &CompiledExpression,
        phase: &str,
        input_index: Option<usize>,
        variables: &BTreeMap<String, i64>,
        context: ExpressionContext<'_>,
    ) -> Result<bool, FuzzError> {
        let dynamic =
            self.evaluate_dynamic(engine, expression, phase, input_index, variables, context)?;
        dynamic.try_cast::<bool>().ok_or_else(|| {
            self.expression_error(
                phase,
                input_index,
                variables,
                "condition must evaluate to bool".into(),
            )
        })
    }

    fn evaluate_dynamic(
        &self,
        engine: &Engine,
        expression: &CompiledExpression,
        phase: &str,
        input_index: Option<usize>,
        variables: &BTreeMap<String, i64>,
        context: ExpressionContext<'_>,
    ) -> Result<Dynamic, FuzzError> {
        evaluate(engine, expression, context)
            .map_err(|message| self.expression_error(phase, input_index, variables, message))
    }

    fn eval_integer(
        &self,
        engine: &Engine,
        number: &CompiledNumber,
        variables: &BTreeMap<String, i64>,
        phase: &str,
        input_index: Option<usize>,
        evaluations: &mut u64,
    ) -> Result<i64, FuzzError> {
        match number {
            CompiledNumber::Integer(value) => Ok(*value),
            CompiledNumber::Expression(expression) => {
                *evaluations += 1;
                let dynamic = self.evaluate_dynamic(
                    engine,
                    expression,
                    phase,
                    input_index,
                    variables,
                    ExpressionContext {
                        variables,
                        initial: None,
                        current: None,
                        before: None,
                        after: None,
                        final_state: None,
                        at: None,
                        held_time: None,
                        input_index,
                        verify: None,
                    },
                )?;
                dynamic.try_cast::<i64>().ok_or_else(|| {
                    self.expression_error(
                        phase,
                        input_index,
                        variables,
                        "frame expression must evaluate to an i64 integer".into(),
                    )
                })
            }
        }
    }

    fn eval_frame(
        &self,
        engine: &Engine,
        number: &CompiledNumber,
        variables: &BTreeMap<String, i64>,
        phase: &str,
        input_index: usize,
        evaluations: &mut u64,
    ) -> Result<u32, FuzzError> {
        self.eval_integer(
            engine,
            number,
            variables,
            phase,
            Some(input_index),
            evaluations,
        )?
        .try_into()
        .map_err(|_| {
            self.expression_error(
                phase,
                Some(input_index),
                variables,
                "frame expression must evaluate to a non-negative u32".into(),
            )
        })
    }

    fn expression_error(
        &self,
        phase: &str,
        input_index: Option<usize>,
        bindings: &BTreeMap<String, i64>,
        message: String,
    ) -> FuzzError {
        FuzzError::Expression {
            phase: phase.to_owned(),
            input: input_index
                .map(|index| format!(" at input {index}"))
                .unwrap_or_default(),
            bindings: format_bindings(bindings),
            message,
        }
    }

    fn compare_candidates(&self, left: &CandidateResult, right: &CandidateResult) -> Ordering {
        for (index, objective) in self.objectives.iter().enumerate() {
            let left_value = left.objective_values[index];
            let right_value = right.objective_values[index];
            let ordering = match objective.kind {
                ObjectiveKind::Maximize => right_value.total_cmp(&left_value),
                ObjectiveKind::Minimize => left_value.total_cmp(&right_value),
                ObjectiveKind::Approach { target } => (left_value - target)
                    .abs()
                    .total_cmp(&(right_value - target).abs()),
            };
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        left.tuple.cmp(&right.tuple)
    }

    fn exact_windows(&self, successful: &[CandidateResult]) -> Vec<ExactWindow> {
        let Some(last_variable) = self.variables.last() else {
            return Vec::new();
        };
        let prefix_length = self.variables.len() - 1;
        let mut groups = BTreeMap::<Vec<i64>, Vec<i64>>::new();
        for candidate in successful {
            groups
                .entry(candidate.tuple[..prefix_length].to_vec())
                .or_default()
                .push(candidate.tuple[prefix_length]);
        }
        groups
            .into_iter()
            .map(|(prefix_values, mut values)| {
                values.sort_unstable();
                values.dedup();
                ExactWindow {
                    prefix: self.prefix_map(&prefix_values),
                    last_variable: last_variable.name.clone(),
                    intervals: intervals(&values, last_variable.step),
                }
            })
            .collect()
    }

    fn coverage_report(
        &self,
        all_tuples: &[Vec<i64>],
        successful: &[CandidateResult],
    ) -> CoverageReport {
        let Some(last_variable) = self.variables.last() else {
            return CoverageReport {
                covered_variable: String::new(),
                reference: "max_width".into(),
                entries: Vec::new(),
            };
        };
        let prefix_length = self.variables.len() - 1;
        let mut all_prefixes = BTreeSet::new();
        for tuple in all_tuples {
            all_prefixes.insert(tuple[..prefix_length].to_vec());
        }
        let mut results = BTreeMap::<Vec<i64>, Vec<&CandidateResult>>::new();
        for candidate in successful {
            results
                .entry(candidate.tuple[..prefix_length].to_vec())
                .or_default()
                .push(candidate);
        }
        let mut entries = all_prefixes
            .into_iter()
            .map(|prefix_values| {
                let candidates = results.remove(&prefix_values).unwrap_or_default();
                let mut values: Vec<_> = candidates
                    .iter()
                    .map(|candidate| candidate.tuple[prefix_length])
                    .collect();
                values.sort_unstable();
                values.dedup();
                let width = values.len() as u64;
                let best = candidates
                    .into_iter()
                    .min_by(|left, right| self.compare_candidates(left, right))
                    .cloned();
                CoverageEntry {
                    prefix: self.prefix_map(&prefix_values),
                    successful_count: values.len() as u64,
                    successful_width: width,
                    coverage_percent: 0.0,
                    intervals: intervals(&values, last_variable.step),
                    best,
                }
            })
            .collect::<Vec<_>>();
        let max_width = entries
            .iter()
            .map(|entry| entry.successful_width)
            .max()
            .unwrap_or(0);
        if max_width > 0 {
            for entry in &mut entries {
                entry.coverage_percent = entry.successful_width as f64 * 100.0 / max_width as f64;
            }
        }
        CoverageReport {
            covered_variable: last_variable.name.clone(),
            reference: "max_width".into(),
            entries,
        }
    }

    fn connected_regions(&self, successful: &[CandidateResult]) -> Vec<RegionSummary> {
        let mut indices = HashMap::new();
        for (index, candidate) in successful.iter().enumerate() {
            indices.insert(candidate.tuple.clone(), index);
        }
        let mut visited = vec![false; successful.len()];
        let mut regions = Vec::new();
        for start in 0..successful.len() {
            if visited[start] {
                continue;
            }
            let mut queue = VecDeque::from([start]);
            let mut members = Vec::new();
            visited[start] = true;
            while let Some(index) = queue.pop_front() {
                members.push(index);
                let tuple = &successful[index].tuple;
                for (axis, variable) in self.variables.iter().enumerate() {
                    for delta in [-variable.step, variable.step] {
                        let Some(value) = tuple[axis].checked_add(delta) else {
                            continue;
                        };
                        let mut adjacent = tuple.clone();
                        adjacent[axis] = value;
                        if let Some(&neighbor) = indices.get(&adjacent)
                            && !visited[neighbor]
                        {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            let mut bounds = BTreeMap::new();
            let mut capacity = 1u64;
            for (axis, variable) in self.variables.iter().enumerate() {
                let min = members
                    .iter()
                    .map(|&index| successful[index].tuple[axis])
                    .min()
                    .expect("region contains start");
                let max = members
                    .iter()
                    .map(|&index| successful[index].tuple[axis])
                    .max()
                    .expect("region contains start");
                bounds.insert(variable.name.clone(), (min, max));
                let axis_count = ((max - min) / variable.step + 1).max(1) as u64;
                capacity = capacity.saturating_mul(axis_count);
            }
            let best = members
                .iter()
                .map(|&index| &successful[index])
                .min_by(|left, right| self.compare_candidates(left, right))
                .expect("region contains start")
                .clone();
            regions.push(RegionSummary {
                bounds,
                successful_count: members.len() as u64,
                density: members.len() as f64 / capacity.max(1) as f64,
                best,
            });
        }
        regions.sort_by(|left, right| self.compare_candidates(&left.best, &right.best));
        regions
    }

    fn prefix_map(&self, values: &[i64]) -> BTreeMap<String, i64> {
        self.variables
            .iter()
            .zip(values)
            .map(|(variable, value)| (variable.name.clone(), *value))
            .collect()
    }
}

fn compile_variable(
    variable: &Variable,
    engine: &Engine,
    cache: &mut HashMap<String, CompiledExpression>,
) -> Result<CompiledVariable, FuzzError> {
    Ok(CompiledVariable {
        name: variable.name.clone(),
        from: compile_number(&variable.range.from, engine, cache)?,
        to: compile_number(&variable.range.to, engine, cache)?,
        step: variable.range.step,
    })
}

fn compile_input(
    input: &InputDeclaration,
    engine: &Engine,
    cache: &mut HashMap<String, CompiledExpression>,
) -> Result<CompiledInput, FuzzError> {
    Ok(CompiledInput {
        keys: input.keys.clone(),
        at: compile_number(&input.at, engine, cache)?,
        held_time: input
            .held_time
            .as_ref()
            .map(|number| compile_number(number, engine, cache))
            .transpose()?,
        hold_infinite: input.hold_infinite,
        before_input: input
            .before_input
            .iter()
            .map(|source| compile_cached(source, engine, cache))
            .collect::<Result<Vec<_>, _>>()?,
        after_input: input
            .after_input
            .iter()
            .map(|source| compile_cached(source, engine, cache))
            .collect::<Result<Vec<_>, _>>()?,
        verify: input.verify,
    })
}

fn compile_number(
    number: &NumberExpression,
    engine: &Engine,
    cache: &mut HashMap<String, CompiledExpression>,
) -> Result<CompiledNumber, FuzzError> {
    match number {
        NumberExpression::Integer(value) => Ok(CompiledNumber::Integer(*value)),
        NumberExpression::Expression(source) => Ok(CompiledNumber::Expression(compile_cached(
            source, engine, cache,
        )?)),
    }
}

fn compile_cached(
    source: &str,
    engine: &Engine,
    cache: &mut HashMap<String, CompiledExpression>,
) -> Result<CompiledExpression, FuzzError> {
    if let Some(expression) = cache.get(source) {
        return Ok(expression.clone());
    }
    let expression = compile_expression(engine, source)?;
    cache.insert(source.to_owned(), expression.clone());
    Ok(expression)
}

fn apply_direction(
    values: &mut [bool],
    at: u32,
    finite_hold: Option<i64>,
    infinite: bool,
    observe_until: u32,
) -> Result<(), FuzzError> {
    if !infinite && finite_hold.is_none() {
        return Err(FuzzError::Spec("direction input requires held_time".into()));
    }
    let end = if infinite {
        observe_until
    } else {
        at.saturating_add(u32::try_from(finite_hold.expect("checked above")).unwrap_or(u32::MAX))
            .min(observe_until)
    };
    for value in &mut values[at as usize..end as usize] {
        *value = true;
    }
    Ok(())
}

fn apply_bool_hold(
    inputs: &mut [InputState],
    at: u32,
    hold: i64,
    observe_until: u32,
    field: impl Fn(&mut InputState) -> &mut bool,
) -> Result<(), FuzzError> {
    let hold: u32 = hold
        .try_into()
        .map_err(|_| FuzzError::Spec("held_time must fit a u32 frame count".into()))?;
    let end = at.saturating_add(hold).min(observe_until);
    for input in &mut inputs[at as usize..end as usize] {
        *field(input) = true;
    }
    Ok(())
}

fn dynamic_number(dynamic: Dynamic) -> Option<f64> {
    if let Some(value) = dynamic.clone().try_cast::<i64>() {
        return Some(value as f64);
    }
    dynamic.try_cast::<f64>().filter(|value| value.is_finite())
}

fn intervals(values: &[i64], step: i64) -> Vec<FrameInterval> {
    let Some(&first) = values.first() else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut start = first;
    let mut previous = first;
    for &value in &values[1..] {
        if previous.checked_add(step) != Some(value) {
            result.push((start, previous));
            start = value;
        }
        previous = value;
    }
    result.push((start, previous));
    result
}

fn min_limit(requested: Option<u64>, configured: u64) -> u64 {
    requested
        .filter(|value| *value > 0)
        .map_or(configured, |value| value.min(configured))
}

#[cfg(feature = "parallel")]
fn add_stats(total: &mut SearchStats, shard: SearchStats) {
    total.candidate_count = total.candidate_count.saturating_add(shard.candidate_count);
    total.successful_count = total
        .successful_count
        .saturating_add(shard.successful_count);
    total.pruned_before = total.pruned_before.saturating_add(shard.pruned_before);
    total.pruned_after = total.pruned_after.saturating_add(shard.pruned_after);
    total.naive_frame_count = total
        .naive_frame_count
        .saturating_add(shard.naive_frame_count);
    total.unique_simulated_frames = total
        .unique_simulated_frames
        .saturating_add(shard.unique_simulated_frames);
    total.saved_frames = total.saved_frames.saturating_add(shard.saved_frames);
    total.trie_nodes = total.trie_nodes.saturating_add(shard.trie_nodes);
    total.peak_cache_bytes = total
        .peak_cache_bytes
        .saturating_add(shard.peak_cache_bytes);
    total.rhai_evaluations = total
        .rhai_evaluations
        .saturating_add(shard.rhai_evaluations);
}

fn bindings_from_hash_map(bindings: HashMap<String, i64>) -> BTreeMap<String, i64> {
    bindings.into_iter().collect()
}

fn format_bindings(bindings: &BTreeMap<String, i64>) -> String {
    let entries = bindings
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>();
    format!("({})", entries.join(", "))
}

#[allow(dead_code)]
const _: usize = size_of::<InputState>();
