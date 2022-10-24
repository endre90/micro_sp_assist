// use "untrigerred" until the set of untriggered transitions stops shrinking
// or until it seems boring to continue shringking?
// then call this step to try to generate the smallest ammount of transitions
// that will satisfy all the valid initial/goal combinations

use rand::seq::SliceRandom;

use micro_sp::{
    get_model_vars, simple_transition_planner, Action, PlanningResult,
    Predicate, SPCommon, SPVariable, State, Transition,
};

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct HintFrame {
    pub init: State,
    pub goal: Predicate,
    pub result: PlanningResult,
    pub grade: usize, // probable hint quality, lower is better
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransitionFrame {
    pub trans: Vec<Transition>,
    pub grade: usize, // probable transition quality, lower is better
}

impl fmt::Display for HintFrame {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmtr,
            "grade: {}\ninit: {}\ngoal: {}\nplan: {:?}\n---------------------------\n",
            self.grade,
            self.init
                .state
                .iter()
                .map(|(var, val)| format!("{} = {} ", var.name.to_string(), val.to_string()))
                .collect::<String>(),
            self.goal,
            self.result.plan
        )
    }
}

impl fmt::Display for TransitionFrame {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmtr,
            "grade: {}: trans: {}\n",
            self.grade,
            self.trans
                .iter()
                .map(|t| format!("{} ", t))
                .collect::<String>()
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GeneratedHint {
    pub frames: Vec<HintFrame>,
    pub trans: Vec<TransitionFrame>,
}

impl fmt::Display for GeneratedHint {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut frames_string = "".to_string();
        self.frames.iter().for_each(|f| {
            frames_string.extend(format!("{}", f).chars());
        });
        let mut transition_strings = "".to_string();
        self.trans.iter().for_each(|t| {
            transition_strings.extend(
                format!(
                    "grade {}: [{}]\n",
                    t.grade,
                    t.trans.iter().map(|x| format!("{} ", x)).collect::<String>()
                )
                .chars(),
            );
        });
        write!(fmtr, "---------------------------\n{}{}", frames_string, transition_strings)
    }
}

pub fn generate_random_transitions(
    namespace: &str,
    vars: &Vec<SPVariable>,
    how_many: usize,
    max_tries: usize,
) -> Vec<Transition> {
    let mut generated_transitions: Vec<Transition> = vec![];
    let mut nr_trans = 1;
    let mut nr_tries = 0;
    while nr_trans <= how_many && nr_tries <= max_tries {
        nr_tries = nr_tries + 1;
        let mut guard_vec = vec![];
        let mut action_vec = vec![];
        vars.iter()
            .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
                Some(random_value) => {
                    guard_vec.push((v.clone(), SPCommon::SPValue(random_value.clone())));
                }
                None => panic!("Variable {:?} has no domain?", v.name),
            });
        vars.iter()
            .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
                Some(random_value) => {
                    action_vec.push((v.clone(), SPCommon::SPValue(random_value.clone())));
                }
                None => panic!("Variable {:?} has no domain?", v.name),
            });
        if guard_vec != action_vec {
            let guard = Predicate::AND(
                guard_vec
                    .iter()
                    .map(|(var, val)| {
                        Predicate::EQ(SPCommon::SPVariable(var.to_owned()), val.to_owned())
                    })
                    .collect(),
            );
            let actions = action_vec
                .iter()
                .map(|(var, val)| Action {
                    var: var.to_owned(),
                    common: val.to_owned(),
                })
                .collect();
            let new_trans = Transition::new(&format!("{}_{}", namespace, nr_trans), guard, actions);
            if !generated_transitions.contains(&new_trans) {
                generated_transitions.push(new_trans);
                nr_trans = nr_trans + 1;
            }
        }
    }
    generated_transitions
}

pub fn hint_with_generated_transitions(
    valid_combinations: Vec<(State, Predicate)>, // used defined valid init/goal combinations
    model: Vec<Transition>,                      // the initial model that might contain a fault
    max_plan_lenght: usize,                      // the individual plan length limit
    max_transitions: usize, // max number of generated transitions in a plan (starts with 1 and goes up to max if .all==True fails)
    max_solutions: usize, // max number of possible transition solutions per generation length (i.e. max_transition)
    max_tries: usize,     // we don't want to loop infinitely
    kill_search: bool      // don't increment the number of transitions if solutions were found for the given lenght
) -> GeneratedHint {
    let mut mut_max_transitions = max_transitions.clone();
    let mut nr_trans = 0;
    let mut hint_frames = vec![];
    let mut transition_frames = vec![];
    let mut tried_transition_combinations = vec![];

    // check first if maybe all combinations are already solvable using the current model
    let mut initial_test_results = vec![];
    valid_combinations.iter().for_each(|(init, goal)| {
        initial_test_results.push(simple_transition_planner(
            init.to_owned(),
            goal.to_owned(),
            model.clone(),
            max_plan_lenght,
        ))
    });

    if initial_test_results.iter().all(|x| x.found) {
        mut_max_transitions = 0
    }

    let vars = get_model_vars(&model);
    while nr_trans < mut_max_transitions {
        nr_trans = nr_trans + 1;
        let mut nr_solutions = 1;
        let mut nr_tries = 0;
        while nr_solutions <= max_solutions && nr_tries <= max_tries {
            let gts = generate_random_transitions(
                &format!("{}_{}", nr_trans, nr_solutions),
                &vars,
                nr_trans,
                max_tries,
            );
            nr_tries = nr_tries + 1;
            if !tried_transition_combinations.contains(&gts) {
                tried_transition_combinations.push(gts.clone());
                let mut intermediate_combintaion_results = vec![];
                let mut new_model = model.clone();
                new_model.extend(gts.clone());
                for (source, sink) in &valid_combinations {
                    let result = simple_transition_planner(
                        source.clone(),
                        sink.clone(),
                        new_model.clone(),
                        max_plan_lenght,
                    );
                    intermediate_combintaion_results.push(HintFrame {
                        init: source.clone(),
                        goal: sink.clone(),
                        result: result.clone(),
                        grade: 0,
                    });
                }
                if intermediate_combintaion_results
                    .iter()
                    .all(|res| res.result.found)
                {
                    if kill_search {
                        mut_max_transitions = nr_trans;
                    }
                    nr_solutions = nr_solutions + 1;
                    transition_frames.push(
                        TransitionFrame {
                            trans: gts.clone(),
                            grade: 0
                        }
                    );
                    for res in intermediate_combintaion_results {
                        if gts.iter().all(|gt| res.result.plan.contains(&gt.name)) {
                            hint_frames.push(res)
                        }
                    }
                }
            }
        }
    }
    GeneratedHint {
        frames: hint_frames,
        trans: transition_frames,
    }
}