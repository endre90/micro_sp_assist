#![allow(unused_imports)]
#![allow(dead_code)]
use crate::core::{generated::generate_random_transitions, graded::hint_with_graded_transitions};
use micro_sp::{
    a, and, eq, s, simple_transition_planner, t, v, bv, Action, Predicate, SPCommon, SPValue,
    SPValueType, SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, Transition,
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_hint_with_graded_transitions_1() {
    let pos = v!("pos", vec!("a", "b", "c", "d"));
    let mut transitions = vec!();
    transitions.push(
        t!(
            "a_to_b",
            eq!(&pos.cr(), "a".cl()),
            vec!(a!(&pos, "b".cl()))
        )
    );
    transitions.push(
        t!(
            "b_to_c",
            eq!(&pos.cr(), "b".cl()),
            vec!(a!(&pos, "a".cl()))
        )
    );
    transitions.push(
        t!(
            "c_to_d",
            eq!(&pos.cr(), "c".cl()),
            vec!(a!(&pos, "d".cl()))
        )
    );
    let mut valid_combinations = vec!();
    // is should be possible to go from "a" to "c"
    valid_combinations.push((s!([(&pos, "a".to_spval())]), eq!(&pos.cr(), "c".cl())));

    let hint = hint_with_graded_transitions(valid_combinations, transitions, 20, 2, 30, 100, true);
    println!("{}", hint)
}

#[test]
fn test_hint_with_graded_transitions_2() {
    let pos = v!("pos", vec!("a", "b", "c", "d", "e", "f"));
    let mut transitions = vec!();
    transitions.push(
        t!(
            "a_to_b",
            eq!(&pos.cr(), "a".cl()),
            vec!(a!(&pos, "b".cl()))
        )
    );
    transitions.push(
        t!(
            "b_to_c",
            eq!(&pos.cr(), "b".cl()),
            vec!(a!(&pos, "a".cl()))
        )
    );
    transitions.push(
        t!(
            "c_to_d",
            eq!(&pos.cr(), "c".cl()),
            vec!(a!(&pos, "d".cl()))
        )
    );
    transitions.push(
        t!(
            "d_to_e",
            eq!(&pos.cr(), "d".cl()),
            vec!(a!(&pos, "b".cl()))
        )
    );
    transitions.push(
        t!(
            "e_to_f",
            eq!(&pos.cr(), "e".cl()),
            vec!(a!(&pos, "f".cl()))
        )
    );
    let mut valid_combinations = vec!();
    // is should be possible to go from "a" to "f"
    valid_combinations.push((s!([(&pos, "a".to_spval())]), eq!(&pos.cr(), "f".cl())));

    // doesn't give much info since it only generates one transition
    // it should also be possible to go from "a" to "c" and "d" to "f"
    valid_combinations.push((s!([(&pos, "a".to_spval())]), eq!(&pos.cr(), "c".cl())));
    valid_combinations.push((s!([(&pos, "d".to_spval())]), eq!(&pos.cr(), "f".cl())));

    // still doesn't give much info, that is why we need to grade the results

    let hint = hint_with_graded_transitions(valid_combinations, transitions, 20, 5, 30, 100, true);
    println!("{}", hint)
}