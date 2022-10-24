use rand::seq::SliceRandom;
use std::{cmp::max, collections::HashMap};

use micro_sp::{
    get_model_vars, simple_transition_planner, Action, PlanningResult, Predicate, SPCommon,
    SPVariable, State, Transition,
};

use std::fmt;

use crate::{core::{
    generated::{hint_with_generated_transitions, GeneratedHint, HintFrame},
    untrigerred::hint_with_untrigerred_transitions,
}, TransitionFrame};

// here we will grade the transitions and hints with tier scores
pub fn hint_with_graded_transitions(
    valid_combinations: Vec<(State, Predicate)>, // used defined valid init/goal combinations
    model: Vec<Transition>,                      // the initial model that might contain a fault
    max_plan_lenght: usize,                      // the individual plan length limit
    max_transitions: usize, // max number of generated transitions in a plan (starts with 1 and goes up to max if .all==True fails)
    max_solutions: usize, // max number of possible transition solutions per generation length (i.e. max_transition)
    max_tries: usize,     // we don't want to loop infinitely
    kill_search: bool      // don't increment the number of transitions if solutions were found for the given lenght
) -> GeneratedHint {
    let non_taken_transitions = hint_with_untrigerred_transitions(
        valid_combinations.clone(),
        model.clone(),
        max_plan_lenght,
    );
    let generated_hints = hint_with_generated_transitions(
        valid_combinations.clone(),
        model.clone(),
        max_plan_lenght,
        max_transitions,
        max_solutions,
        max_tries,
        kill_search
    );
    let unscored_hint_frames = generated_hints.frames.clone();
    let unscored_transition_frames = generated_hints.trans.clone();
    let mut scored_hint_frames = vec![];
    let mut scored_transition_frames = vec![];
    let mut scored_transitions_names: HashMap<String, usize> = HashMap::new();
    for hint_frame in &unscored_hint_frames {
        let mut frame_grade = 1;
        for nt in &non_taken_transitions {
            if hint_frame.result.plan.contains(&nt) {
                frame_grade = frame_grade + 1;
            }
        }
        for trans_frame in &unscored_transition_frames {
            for trans in &trans_frame.trans {
                if scored_transitions_names.contains_key(&trans.name) {
                    let old_score: usize = scored_transitions_names.get(&trans.name).unwrap().clone();
                    scored_transitions_names.insert(trans.name.clone(), max(old_score.clone(), frame_grade));
                } else {
                    scored_transitions_names.insert(trans.name.clone(), frame_grade);
                }
            }
        }
        
        for utf in &unscored_transition_frames {
            let mut grades = vec!();
            for trans in &utf.trans {
                grades.push(scored_transitions_names.get(&trans.name).unwrap().clone())
            }
            let grade = grades.iter().max().unwrap();
            scored_transition_frames.push(
                TransitionFrame {
                    trans: utf.trans.clone(),
                    grade: *grade
                }
            )
        }

        for uhf in &unscored_hint_frames {
            let mut hint_score = 1;
            for t in scored_transitions_names.keys() {
                if uhf.result.plan.contains(&t) {
                    hint_score = max(hint_score, *scored_transitions_names.get(t).unwrap());
                }
            }
            scored_hint_frames.push(HintFrame {
                init: uhf.init.clone(),
                goal: uhf.goal.clone(),
                result: uhf.result.clone(),
                grade: hint_score,
            })
        }
    }
    GeneratedHint {
        frames: scored_hint_frames,
        trans: scored_transition_frames,
    }
}