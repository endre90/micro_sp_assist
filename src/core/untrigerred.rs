use std::collections::HashSet;

use micro_sp::{simple_transition_planner, Predicate, State, Transition};

pub fn step_2(
    valid_combinations: Vec<(State, Predicate)>,
    model: Vec<Transition>,
    max_plan_lenght: usize,
) -> Vec<String> {
    let all_transitions = model
        .iter()
        .map(|t| t.name.clone())
        .collect::<HashSet<String>>();
    let mut taken_transitions = HashSet::new();
    for comb in valid_combinations {
        let result = simple_transition_planner(comb.0, comb.1, model.clone(), max_plan_lenght);
        match &result.found {
            true => result.plan.iter().for_each(|t| {
                taken_transitions.insert(t.clone());
            }),
            false => (),
        }
    }
    let not_taken_transitions = all_transitions
        .difference(&taken_transitions)
        .map(|x| x.to_owned())
        .collect::<HashSet<String>>();
    let mut to_return = not_taken_transitions
        .iter()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    to_return.sort();
    to_return
}
