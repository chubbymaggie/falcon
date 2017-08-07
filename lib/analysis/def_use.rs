use analysis::*;
use analysis::analysis_location::AnalysisLocation::*;
use il;
use std::collections::{BTreeMap, BTreeSet};


pub fn def_use(
    reaching_definitions: &BTreeMap<AnalysisLocation, Reaches>,
    control_flow_graph: &il::ControlFlowGraph
) -> Result<BTreeMap<AnalysisLocation, BTreeSet<AnalysisLocation>>> {
    let mut du = BTreeMap::new();

    // initialize the resulting map first
    for rd in reaching_definitions {
        du.insert(rd.0.clone(), BTreeSet::new());
    }

    // for every reaching destination
    for rd in reaching_definitions {
        let this_location = rd.0;
        let def_locations = rd.1;
        // build our haystack of uses to search for defs against
        let haystack: BTreeSet<il::MultiVar> = match *this_location {
            Edge(ref el) => match *el.find(control_flow_graph)
                                     .unwrap()
                                     .condition() {
                Some(ref condition) => condition.collect_scalars()
                                                .iter()
                                                .map(|v| il::MultiVar::Scalar((*v).clone()))
                                                .collect::<BTreeSet<il::MultiVar>>(),
                None => BTreeSet::new()
            },
            Instruction(ref il) => il.find(control_flow_graph)?
                                     .variables_read()
                                     .iter()
                                     .map(|v| v.multi_var_clone())
                                     .collect::<BTreeSet<il::MultiVar>>(),
            EmptyBlock(_) => BTreeSet::new()
        };

        // for each reaching definition that reaches here
        for def_location in def_locations.in_() {
            if let Instruction(ref def_location) = *def_location {
              // if the definition is actually used here
              if let Some(variable_written) = def_location
                                          .find(control_flow_graph)?
                                          .variable_written() {
                  if haystack.contains(&variable_written.multi_var_clone()) {
                      du.get_mut(&def_location.clone().into())
                        .unwrap()
                        .insert(this_location.clone());
                  }
              }
            }
        }
    }

    Ok(du)
}



pub fn use_def(
    reaching_definitions: &BTreeMap<AnalysisLocation, Reaches>,
    control_flow_graph: &il::ControlFlowGraph
) -> Result<BTreeMap<AnalysisLocation, BTreeSet<AnalysisLocation>>> {
    let mut ud = BTreeMap::new();

    // initialize the resulting map first
    for rd in reaching_definitions {
        ud.insert(rd.0.clone(), BTreeSet::new());
    }

    // for every reaching destination
    for rd in reaching_definitions {
        let this_location = rd.0;
        let def_locations = rd.1;
        // build our haystack of uses to search for defs against
        let haystack: BTreeSet<il::MultiVar> = match *this_location {
            Edge(ref el) => match *el.find(control_flow_graph)
                                     .unwrap()
                                     .condition() {
                Some(ref condition) => condition.collect_scalars()
                                            .iter()
                                            .map(|v| il::MultiVar::Scalar((*v).clone()))
                                            .collect::<BTreeSet<il::MultiVar>>(),
                None => BTreeSet::new()
            },
            Instruction(ref il) => il.find(control_flow_graph)?
                                     .variables_read()
                                     .iter()
                                     .map(|v| v.multi_var_clone())
                                     .collect::<BTreeSet<il::MultiVar>>(),
           EmptyBlock(_) => BTreeSet::new()
        };

        // for each reaching definition that reaches here
        for def_location in def_locations.in_() {
            if let Instruction(ref def_location) = *def_location {
              // if the definition is actually used here
              if let Some(variable_written) = def_location
                                          .find(control_flow_graph)?
                                          .variable_written() {
                  if haystack.contains(&variable_written.multi_var_clone()) {
                      ud.get_mut(&this_location.clone())
                        .unwrap()
                        .insert(def_location.clone().into());
                  }
              }
            }
        }
    }

    Ok(ud)
}