#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, and, eq, simple_transition_planner, t, v, Action, Predicate, SPCommon, SPValue,
    SPValueType, SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, Transition,
};
use std::collections::{HashMap, HashSet};

use crate::core::anomalies::hint_with_anomalies;

#[test]
fn test_hint_with_anomalies_1() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval()),
        (stat.clone(), "off".to_spval()),
    ]));

    let t1 = t!(
        "a_to_b",
        and!(
            eq!(&pos.cr(), "a".cl()),
            eq!(&stat.cr(), "on".cl())
        ),
        vec!(a!(pos.clone(), "b".cl()))
    );
    // introduce fault here
    let t2 = t!(
        "b_to_c",
        and!(
            eq!(&pos.cr(), "b".cl()),
            eq!(&stat.cr(), "on".cl())
        ),
        vec!(a!(pos.clone(), "a".cl()))
    );
    let t3 = t!(
        "c_to_d",
        and!(
            eq!(&pos.cr(), "c".cl()),
            eq!(&stat.cr(), "on".cl())
        ),
        vec!(a!(pos.clone(), "d".cl()))
    );
    let t4 = t!(
        "turn_on",
        eq!(&stat.cr(), "off".cl()),
        vec!(a!(stat.clone(), "on".cl()))
    );
    let t5 = t!(
        "turn_off",
        eq!(&stat.cr(), "on".cl()),
        vec!(a!(stat.clone(), "off".cl()))
    );

    let result = hint_with_anomalies(
        vec![t1.clone(), t2.clone(), t3.clone(), t4.clone(), t5.clone()],
        100, // max_tries
        50,  // max_combinations
        5,   // max solutions
        20,  // max_plan_lenght
    );

    println!("combination coverage: {}%", result.combination_coverage);
    println!("solution coverage: {}%", result.solution_coverage);
    println!("results shown: {}", result.solution.len());
    println!("time to solve: {:?}", result.time);
    println!("-----------------------------");
    for r in result.solution {
        let mut inits =
            r.0.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        inits.sort();
        let mut goals =
            r.1.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        goals.sort();

        println!("init: {:?}", inits);
        println!("goal: {:?}", goals);
        println!("plan: {:?}", r.2.plan);
        println!("-----------------------------");
    }
}

#[test]
fn test_hint_with_anomalies_2() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval()),
        (stat.clone(), "off".to_spval()),
    ]));

    // introduce fault here
    let mut transitions = vec![];
    for pos1 in &pos.domain {
        for pos2 in &pos.domain {
            if pos1 != pos2 {
                transitions.push(t!(
                    &format!("{}_to_{}", &pos1.to_string(), &pos2.to_string()),
                    and!(
                        eq!(&pos.cr(), pos1.to_string().cl()),
                        eq!(&stat.cr(), "on".cl())
                    ),
                    vec!(a!(pos.clone(), pos1.to_string().cl()))
                ))
            }
        }
    }

    transitions.push(t!(
        "turn_on",
        eq!(&stat.cr(), "off".cl()),
        vec!(a!(stat.clone(), "on".cl()))
    ));
    transitions.push(t!(
        "turn_off",
        eq!(&stat.cr(), "on".cl()),
        vec!(a!(stat.clone(), "off".cl()))
    ));

    let result = hint_with_anomalies(
        transitions,
        100, // max_tries
        50,  // max_combinations
        5,   // max solutions
        20,  // max_plan_lenght
    );

    println!("combination coverage: {}%", result.combination_coverage);
    println!("solution coverage: {}%", result.solution_coverage);
    println!("results shown: {}", result.solution.len());
    println!("time to solve: {:?}", result.time);
    println!("-----------------------------");
    for r in result.solution {
        let mut inits =
            r.0.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        inits.sort();
        let mut goals =
            r.1.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        goals.sort();

        println!("init: {:?}", inits);
        println!("goal: {:?}", goals);
        println!("plan: {:?}", r.2.plan);
        println!("-----------------------------");
    }
}
