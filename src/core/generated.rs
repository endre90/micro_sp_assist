// the idea here is to try to generate transitions that can correct the model
// by replacing faulty transitions. using a set of provided valid init states
// and goal predicates, and the model which contains errors, we try to solve
// each problem for the init/goal combination. if a plan is found, we save the
// used transitions in it as "valid" in a list. in step 2, we have just returned
// the difference of all transitions and these "valid" transitions so that we
// can indicate to which transitions could be faulty.

// in step 3, the idea is to generate transitions which can be used together with
// the previously annotated "valid" transitions to search for a plan. First of all,
// we should test if the generated transition(s) can help solve the unsuccessfull
// init/goal combination, and later test for all combinations.
// Do it stepwise actually, for each provided init/goal combination generate hints
// and from there select which ones are ok and not ok, do so for each init/goal combination.
// Save the ok ones in the taken transitions list and save the not ok one in the disabled
// transition lists so that they are disable for the future. Do so until the model is comlete?

// in step 4, we solve for all previously valid combinations. if we have managed to get
// the correct transition for the specific init/goal combination, it is time to test it
// together with the other init/goal combinations. we do this iteratively, since it could be
// that we generated a correct transition for the current set of init/goal combinations but
// that that transition is not correct for a future set, so we might have to re move it.
// so, after every iteration, we have to try to solve everything again and make new "valid"
// transitions lists. we do this until we have unsuccessfull init/goal combinations
// we do this iteratively,
// i.e. we reinforce (correct) the model until we can solve all init/goal combinations.
// we have to find the common treansitions which work for every init/goal combination.

// new insights into step 3:
// use step 2 until the set of untriggered transitions stops shrinking
// or until it seems boring to continue shringking?
// then call this step to try to generate the smallest ammount of transitions
// that will satisfy all the valid initial/goal combinations

// first try to generate one transition that satisfies the valid initial/goal combinations
// if it fails to do so for all combinations, discard it and try to generate two different transitions that try to do that
// keep a vector of tried and/or taken and/or failed (discarded) transitions so that different ones are generated next time

// we might have to manually say something like: no, 1 transition is bad, I want 2, now I want 3 and so on...

// keep removing the generated ones in the next iterations

// on one side show these transitions, and on the other side show the names of transitions that were not taken

use rand::seq::SliceRandom;
use std::{
    cmp::max,
    collections::{HashMap},
};

use micro_sp::{
    get_model_vars, simple_transition_planner, Action, PlanningResult,
    Predicate, SPCommon, SPVariable, State, Transition,
};

use crate::core::untrigerred::hint_with_untrigerred_transitions;

#[derive(Debug, PartialEq, Clone)]
pub struct HintFrame {
    pub init: State,
    pub goal: Predicate,
    pub result: PlanningResult,
    pub grade: usize, // probable hint quality, lower is better
}

pub fn generate_random_transition(name: &str, vars: &Vec<SPVariable>) -> Option<Transition> {
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
        Some(Transition::new(name, guard, actions))
    } else {
        None
    }
}

pub fn generate_random_transitions(
    name: &str,
    max_nr: usize,
    vars: &Vec<SPVariable>,
) -> Vec<Transition> {
    let mut nr = 0;
    let mut trans: Vec<Transition> = vec![];
    loop {
        if nr >= max_nr {
            break;
        };
        let maybe = generate_random_transition(name, vars);
        match maybe {
            Some(t) => {
                let comparable_trans = trans
                    .iter()
                    .map(|x| Transition::new("asdf", x.to_owned().guard.clone(), x.actions.clone()))
                    .collect::<Vec<Transition>>();
                let comparable_t =
                    Transition::new("asdf", t.to_owned().guard.clone(), t.actions.clone());
                if !comparable_trans.contains(&comparable_t) {
                    trans.push(t);
                    nr = nr + 1;
                }
            }
            None => (),
        }
    }
    trans
}

// here we will grade the transitions and hints with tier scores
pub fn postprocess(
    valid_combinations: Vec<(State, Predicate)>,
    model: Vec<Transition>,
    max_plan_lenght: usize, // the individual plan length limit
    solutions: Option<(Vec<HintFrame>, Vec<Transition>)>,
) -> Option<(Vec<HintFrame>, Vec<(Transition, usize)>)> {
    let non_taken_transitions = hint_with_untrigerred_transitions(valid_combinations.clone(), model.clone(), max_plan_lenght);
    match solutions {
        Some(s) => {
            let not_scored_hint_frames = s.0.clone();
            let not_scored_transitions = s.1.clone();
            let mut scored_transitions_names: HashMap<String, usize> = HashMap::new();

            for frame in &not_scored_hint_frames {
                let mut frame_grade = 1;
                for nt in &non_taken_transitions {
                    if frame.result.plan.contains(&nt) {
                        frame_grade = frame_grade + 1;
                    }
                }
                for ns in &not_scored_transitions {
                    if frame.result.plan.contains(&ns.name) {
                        if scored_transitions_names.contains_key(&ns.name) {
                            let old_score: usize =
                                scored_transitions_names.get(&ns.name).unwrap().clone();
                            scored_transitions_names
                                .insert(ns.name.clone(), max(old_score.clone(), frame_grade));
                            // scored_transitions.push((ns.clone(), max(old_score.clone(), frame_grade)));
                        } else {
                            scored_transitions_names.insert(ns.name.clone(), frame_grade);
                            // scored_transitions.push((ns.clone(), frame_grade));
                        }
                    }
                }
            }
            let mut scored_transitions = vec![];
            for ns in &not_scored_transitions {
                scored_transitions.push((
                    ns.clone(),
                    scored_transitions_names.get(&ns.name).unwrap().clone(),
                ))
            }

            let mut scored_hint_frames = vec![];
            for ns in &not_scored_hint_frames {
                let mut hint_score = 1;
                for t in &scored_transitions {
                    if ns.result.plan.contains(&t.0.name) {
                        hint_score = max(hint_score, t.1);
                    }
                }
                scored_hint_frames.push(HintFrame {
                    init: ns.init.clone(),
                    goal: ns.goal.clone(),
                    result: ns.result.clone(),
                    grade: hint_score,
                })
            }

            Some((scored_hint_frames, scored_transitions))
        }
        None => None,
    }
}



// pub fn step_3_new(
//     valid_combinations: Vec<(State, Predicate)>,
//     model: Vec<Transition>,
//     max_plan_lenght: usize, // the individual plan length limit
//     max_transitions: usize, // max number of generated transitions in a plan (starts with 1 and goes up to max if .all==True fails)
//     max_solutions: usize, // max number of possible transition solutions per generation length (i.e. max_transition)
//     max_tries: usize,     // we don't want to infinitelly loop
// ) -> Option<(Vec<(State, Predicate, PlanningResult)>, Vec<Transition>)> {
//     let mut initial_results = vec![];
//     let mut valid_generated_transitions = vec![];
//     let mut hint_frames = vec![];
//     valid_combinations.iter().for_each(|(init, goal)| {
//         initial_results.push(simple_transition_planner(
//             init.to_owned(),
//             goal.to_owned(),
//             model.clone(),
//             max_plan_lenght,
//         ))
//     });
//     match initial_results.iter().all(|x| x.found) {
//         true => None, // means that all init/goal combinations are already satisfied with the model
//         false => {
//             let mut nr_transitions = 0;

//             let mut tried_transitions = model.clone(); // include the modeled and later the transitions that have failed to solve all
//             let vars = get_model_vars(&model); // get all the variables that are part of the model
//             'outer: loop {
//                 if nr_transitions >= max_transitions {
//                     break 'outer Some((hint_frames, valid_generated_transitions));
//                 };
//                 nr_transitions = nr_transitions + 1;
//                 let mut nr_solutions = 1;
//                 let mut nr_tries = 1;
//                 'inner: loop {
//                     if nr_solutions > max_solutions {
//                         break;
//                     };
//                     if nr_tries > max_tries {
//                         break;
//                     };
//                     nr_tries = nr_tries + 1;
//                     let gt = generate_random_transition(
//                         &format!("FIX_{}_{}", nr_transitions, nr_solutions),
//                         &vars,
//                     ); // for now just one, more later
//                     match gt {
//                         None => {} //generation failed, just increment the number of tries
//                         Some(generated_transition) => {
//                             let copmarable_generated_transition = Transition::new(
//                                 "asdf",
//                                 generated_transition.guard.clone(),
//                                 generated_transition.actions.clone(),
//                             );
//                             let comparable_tried_transitions = tried_transitions
//                                 .iter()
//                                 .map(|t| {
//                                     Transition::new("asdf", t.guard.clone(), t.actions.clone())
//                                 })
//                                 .collect::<Vec<Transition>>();
//                             match comparable_tried_transitions
//                                 .contains(&copmarable_generated_transition)
//                             {
//                                 true => {} // we have already tried this one, just increment the number of tries
//                                 false => {
//                                     // ok now we can add the generated transition(s) to the model and try to find a solution
//                                     tried_transitions.push(generated_transition.clone());
//                                     let mut valid_combination_results = vec![];
//                                     let mut modified_model = model.clone();
//                                     modified_model.push(generated_transition.clone());

//                                     for (init, goal) in &valid_combinations {
//                                         valid_combination_results.push((
//                                             init.clone(),
//                                             goal.clone(),
//                                             simple_transition_planner(
//                                                 init.clone(),
//                                                 goal.clone(),
//                                                 modified_model.clone(),
//                                                 max_plan_lenght,
//                                             ),
//                                         ));
//                                     }
//                                     match valid_combination_results.iter().all(|x| x.2.found) {
//                                         false => {}
//                                         true => {
//                                             nr_solutions = nr_solutions + 1;
//                                             let copmarable_valid_generated_transitions =
//                                                 valid_generated_transitions
//                                                     .iter()
//                                                     .map(|t| {
//                                                         Transition::new(
//                                                             "asdf",
//                                                             t.guard.clone(),
//                                                             t.actions.clone(),
//                                                         )
//                                                     })
//                                                     .collect::<Vec<Transition>>();
//                                             if !copmarable_valid_generated_transitions
//                                                 .contains(&copmarable_generated_transition)
//                                             {
//                                                 valid_generated_transitions
//                                                     .push(generated_transition.clone());
//                                             }
//                                             // filter out the combinations which don't use the generated transitions
//                                             let mut filtered = vec![];
//                                             for x in valid_combination_results {
//                                                 if x.2.plan.contains(&generated_transition.name) {
//                                                     filtered.push(x.clone())
//                                                 }
//                                             }
//                                             hint_frames.extend(filtered);
//                                             // hint_frames = valid_combination_results;
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }



// // will try marking the hints here
// // explain this one in the paper and then maybe later expant to the multiple transition generation one
// pub fn step_3_new_new(
//     valid_combinations: Vec<(State, Predicate)>,
//     model: Vec<Transition>,
//     max_plan_lenght: usize, // the individual plan length limit
//     max_transitions: usize, // max number of generated transitions in a plan (starts with 1 and goes up to max if .all==True fails)
//     max_solutions: usize, // max number of possible transition solutions per generation length (i.e. max_transition)
//     max_tries: usize,     // we don't want to infinitelly loop
// ) -> Option<(Vec<HintFrame>, Vec<Transition>)> {
//     let mut initial_results = vec![];
//     let mut valid_generated_transitions = vec![];
//     let mut hint_frames = vec![];
//     valid_combinations.iter().for_each(|(init, goal)| {
//         initial_results.push(simple_transition_planner(
//             init.to_owned(),
//             goal.to_owned(),
//             model.clone(),
//             max_plan_lenght,
//         ))
//     });
//     let non_taken_transitions = step_2(valid_combinations.clone(), model.clone(), max_plan_lenght);
//     match initial_results.iter().all(|x| x.found) {
//         true => None, // means that all init/goal combinations are already satisfied with the model
//         false => {
//             let mut nr_transitions = 0;

//             let mut tried_transitions = model.clone(); // include the modeled and later the transitions that have failed to solve all
//             let vars = get_model_vars(&model); // get all the variables that are part of the model
//             'outer: loop {
//                 if nr_transitions >= max_transitions {
//                     break 'outer Some((hint_frames, valid_generated_transitions));
//                 };
//                 nr_transitions = nr_transitions + 1;
//                 let mut nr_solutions = 1;
//                 let mut nr_tries = 1;
//                 'inner: loop {
//                     if nr_solutions > max_solutions {
//                         break;
//                     };
//                     if nr_tries > max_tries {
//                         break;
//                     };
//                     nr_tries = nr_tries + 1;
//                     let gt = generate_random_transition(
//                         &format!("FIX_{}_{}", nr_transitions, nr_solutions),
//                         &vars,
//                     ); // for now just one, more later
//                     match gt {
//                         None => {} //generation failed, just increment the number of tries
//                         Some(generated_transition) => {
//                             let copmarable_generated_transition = Transition::new(
//                                 "asdf",
//                                 generated_transition.guard.clone(),
//                                 generated_transition.actions.clone(),
//                             );
//                             let comparable_tried_transitions = tried_transitions
//                                 .iter()
//                                 .map(|t| {
//                                     Transition::new("asdf", t.guard.clone(), t.actions.clone())
//                                 })
//                                 .collect::<Vec<Transition>>();
//                             match comparable_tried_transitions
//                                 .contains(&copmarable_generated_transition)
//                             {

                                
//                                 true => {} // we have already tried this one, just increment the number of tries
//                                 false => {
//                                     // ok now we can add the generated transition(s) to the model and try to find a solution
//                                     tried_transitions.push(generated_transition.clone());
//                                     let mut valid_combination_results = vec![];
//                                     let mut modified_model = model.clone();
//                                     modified_model.push(generated_transition.clone());

//                                     for (init, goal) in &valid_combinations {
//                                         let result = simple_transition_planner(
//                                             init.clone(),
//                                             goal.clone(),
//                                             modified_model.clone(),
//                                             max_plan_lenght,
//                                         );

//                                         // move this to postprocessing
//                                         let tier = 0;
//                                         // for nt in &non_taken_transitions {
//                                         //     if result.plan.contains(&nt) {
//                                         //         tier = tier + 1;
//                                         //     }
//                                         // }
//                                         // we have to check on the transition level first and mark the hints later
//                                         // i.e. if any plans with a FIX contain any nontaken transition for all plans
//                                         valid_combination_results.push(HintFrame {
//                                             init: init.clone(),
//                                             goal: goal.clone(),
//                                             result: result.clone(),
//                                             tier,
//                                         });
//                                     }
//                                     match valid_combination_results.iter().all(|x| x.result.found) {
//                                         false => {}
//                                         true => {
//                                             nr_solutions = nr_solutions + 1;
//                                             let copmarable_valid_generated_transitions =
//                                                 valid_generated_transitions
//                                                     .iter()
//                                                     .map(|t| {
//                                                         Transition::new(
//                                                             "asdf",
//                                                             t.guard.clone(),
//                                                             t.actions.clone(),
//                                                         )
//                                                     })
//                                                     .collect::<Vec<Transition>>();
//                                             if !copmarable_valid_generated_transitions
//                                                 .contains(&copmarable_generated_transition)
//                                             {
//                                                 valid_generated_transitions
//                                                     .push(generated_transition.clone());
//                                             }
//                                             // filter out the combinations which don't use the generated transitions
//                                             let mut filtered = vec![];
//                                             for x in valid_combination_results {
//                                                 if x.result
//                                                     .plan
//                                                     .contains(&generated_transition.name)
//                                                 {
//                                                     filtered.push(x.clone())
//                                                 }
//                                             }
//                                             hint_frames.extend(filtered);
//                                             // hint_frames = valid_combination_results;
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }



// pub fn step_3(
//     valid_combinations: Vec<(State, Predicate)>,
//     model: Vec<Transition>,
//     max_plan_lenght: usize,
//     max_trans: usize,
//     max_tries: usize,
// ) -> Vec<Transition> {
//     // let mut model_transitions = model.clone();
//     let mut failed_transitions = model.clone();
//     let vars = get_model_vars(&model);
//     let mut nr_trans = 0;
//     let mut tries = 0;
//     let mut failed = false;
//     let mut working_trans = vec![];
//     'outer: loop {
//         if tries >= max_tries {
//             break;
//         }
//         let new_trans = generate_random_transition("FIX", &vars);
//         match new_trans {
//             Some(t) => {
//                 if !failed_transitions.contains(&t) {
//                     // FIRST: we have to check if all of them pass without adding a new transition
//                     // generate up to several counterexample transitions for one transition length i.e. FIX_0 (forbid the ones that exist already)
//                     // for more than 1 FIX, also have more sounterexamples

//                     // later, maybe also add forbidden init/goal combinations so that we can narrow the search more?

//                     // need to check for all transitions

//                     // also, have to remove the failed ones from the main transitions list
//                     // and have a failed tries list, so that we don't end up with random going back plans and such...
//                     // println!("ADDED NEW TRANSITION!");
//                     let mut model_transitions = model.clone();
//                     model_transitions.push(t.clone());
//                     'inner: for (init, goal) in &valid_combinations {
//                         let result = simple_transition_planner(
//                             init.clone(),
//                             goal.clone(),
//                             model_transitions.clone(),
//                             max_plan_lenght,
//                         );
//                         if !result.found {
//                             failed_transitions.push(t.clone());
//                             failed = true;
//                             break 'inner;
//                         }
//                     }
//                     if !failed {
//                         working_trans.push(t.clone());
//                         break 'outer;
//                     } else {
//                         failed = false;
//                     }
//                 }
//             }
//             None => (),
//         }
//         tries = tries + 1;
//     }

//     working_trans
// }


// // TODO: Implement EQ and PArtialEQ for transitions to be able to compare them
// // doesn't work yet
// // will try marking the hints here
// pub fn step_3_final(
//     valid_combinations: Vec<(State, Predicate)>,
//     model: Vec<Transition>,
//     max_plan_lenght: usize, // the individual plan length limit
//     max_transitions: usize, // max number of generated transitions in a plan (starts with 1 and goes up to max if .all==True fails)
//     max_solutions: usize, // max number of possible transition solutions per generation length (i.e. max_transition)
//     max_tries: usize,     // we don't want to infinitelly loop
// ) -> Option<(Vec<HintFrame>, Vec<Vec<Transition>>)> {
//     let mut initial_results = vec![];
//     let mut valid_generated_transitions = vec![];
//     let mut hint_frames = vec![];
//     valid_combinations.iter().for_each(|(init, goal)| {
//         initial_results.push(simple_transition_planner(
//             init.to_owned(),
//             goal.to_owned(),
//             model.clone(),
//             max_plan_lenght,
//         ))
//     });
//     let non_taken_transitions = step_2(valid_combinations.clone(), model.clone(), max_plan_lenght);
//     match initial_results.iter().all(|x| x.found) {
//         true => None, // means that all init/goal combinations are already satisfied with the model
//         false => {
//             let mut nr_transitions = 0;

//             // now we want to see if we have already generated the transition vectors (i.e. for sizes 1, 2, 3...)
//             let mut tried_transitions = model
//                 .clone()
//                 .iter()
//                 .map(|t| vec![t.clone()])
//                 .collect::<Vec<Vec<Transition>>>();
//             let vars = get_model_vars(&model); // get all the variables that are part of the model
//             'outer: loop {
//                 if nr_transitions >= max_transitions {
//                     break 'outer Some((hint_frames, valid_generated_transitions));
//                 };
//                 nr_transitions = nr_transitions + 1;
//                 let mut nr_solutions = 1;
//                 let mut nr_tries = 1;
//                 'inner: loop {
//                     if nr_solutions > max_solutions {
//                         break;
//                     };
//                     if nr_tries > max_tries {
//                         break;
//                     };
//                     nr_tries = nr_tries + 1;

//                     // now we want to generate more than one transition
//                     let generated_transitions = generate_random_transitions(
//                         &format!("FIX_{}_{}", nr_transitions, nr_solutions),
//                         nr_transitions,
//                         &vars,
//                     );
//                     let comparable_generated_transitions = generated_transitions
//                         .iter()
//                         .map(|t| Transition::new("asdf", t.guard.clone(), t.actions.clone()))
//                         .collect::<Vec<Transition>>();
//                     let comparable_tried_transitions = tried_transitions
//                         .iter()
//                         .map(|tv| {
//                             tv.iter()
//                                 .map(|t| {
//                                     Transition::new("asdf", t.guard.clone(), t.actions.clone())
//                                 })
//                                 .collect::<Vec<Transition>>()
//                         })
//                         .collect::<Vec<Vec<Transition>>>();

//                     match comparable_tried_transitions.contains(&comparable_generated_transitions) {
//                         true => {} // we have already tried these, just increment the number of tries
//                         false => {
//                             // ok now we can add the generated transition(s) to the model and try to find a solution
//                             tried_transitions.push(generated_transitions.clone());
//                             let mut valid_combination_results = vec![];
//                             let mut modified_model = model.clone();
//                             for t in &generated_transitions {
//                                 modified_model.push(t.clone());
//                             }
//                             for (init, goal) in &valid_combinations {
//                                 let result = simple_transition_planner(
//                                     init.clone(),
//                                     goal.clone(),
//                                     modified_model.clone(),
//                                     max_plan_lenght,
//                                 );

//                                 let tier = 0;
//                                 valid_combination_results.push(HintFrame {
//                                     init: init.clone(),
//                                     goal: goal.clone(),
//                                     result: result.clone(),
//                                     tier,
//                                 });
//                             }

//                             match valid_combination_results.iter().all(|x| x.result.found) {
//                                 false => {}
//                                 true => {
//                                     nr_solutions = nr_solutions + 1;
//                                     let copmarable_valid_generated_transitions =
//                                         valid_generated_transitions
//                                             .iter()
//                                             .map(|tv| {
//                                                 tv.iter()
//                                                     .map(|t| {
//                                                         Transition::new(
//                                                             "asdf",
//                                                             t.guard.clone(),
//                                                             t.actions.clone(),
//                                                         )
//                                                     })
//                                                     .collect::<Vec<Transition>>()
//                                             })
//                                             .collect::<Vec<Vec<Transition>>>();

//                                     if !copmarable_valid_generated_transitions
//                                         .contains(&comparable_generated_transitions)
//                                     {
//                                         valid_generated_transitions
//                                             .push(generated_transitions.clone());
//                                     }
//                                     let mut filtered = vec![];
//                                     for x in &valid_combination_results {
//                                         for t in &generated_transitions {
//                                             if x.result.plan.contains(&t.name) {
//                                                 filtered.push(x.clone())
//                                             }
//                                         }
//                                     }
//                                     hint_frames.extend(filtered);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
