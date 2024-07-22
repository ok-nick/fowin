// use std::{collections::HashMap, thread, time::Duration};

// use fowin::{Position, Size};
// use rand::Rng;
// use uuid::Uuid;

// use crate::{
//     executor::Executor,
//     timeline::{Action, ExecScope, Step, Timeline},
//     Mutation,
// };

// #[derive(Debug)]
// pub struct Chronology {
//     timeline: Timeline,
//     // For now we store a map from local window id->global window id. This is so that we
//     // can find the window globally via fowin. The reason we don't store the global id
//     // directly in the timeline is because in the future, we should be finding windows
//     // based on their process id + local window id.
//     global_ids: HashMap<u32, String>,
// }

// impl Chronology {
//     pub fn execute<E: Executor>(&mut self, executor: &E) {
//         println!("{:?}", self.timeline.steps);
//         for step in &self.timeline.steps {
//             thread::sleep(Duration::from_secs(1));
//             println!("NEXT");

//             // TODO: verify props for fowin here, before we begin as well

//             match step.details.scope {
//                 // Use fowin locally... kinda contradictory huh.
//                 ExecScope::Local => {
//                     let id = self.global_ids.get(&step.id).unwrap();
//                     let mut found = false;
//                     for window in fowin::iter_windows().flatten() {
//                         let title = window.title().unwrap();
//                         println!("{title}");
//                         if title == *id {
//                             found = true;
//                             match &step.details.action {
//                                 Action::Spawn => {
//                                     todo!()
//                                 }
//                                 Action::Terminate => {
//                                     todo!()
//                                 }
//                                 Action::Mutate(mutation) => match mutation {
//                                     Mutation::Size(size) => window
//                                         .resize(Size {
//                                             width: size.width,
//                                             height: size.height,
//                                         })
//                                         .unwrap(),
//                                     Mutation::Position(position) => window
//                                         .reposition(Position {
//                                             x: position.x,
//                                             y: position.y,
//                                         })
//                                         .unwrap(),
//                                     Mutation::Fullscreen(fullscreen) => match fullscreen {
//                                         true => window.fullscreen().unwrap(),
//                                         false => window.unfullscreen().unwrap(),
//                                     },
//                                     Mutation::Hidden(hidden) => match hidden {
//                                         true => window.hide().unwrap(),
//                                         false => window.show().unwrap(),
//                                     },
//                                     Mutation::AtFront(_) => todo!(),
//                                     Mutation::Focused(_) => todo!(),
//                                     _ => {
//                                         // These properties aren't mutatable by fowin. They should also never be generated
//                                         // in the first place because of timeline generation rules.
//                                         // TODO: add unreachable? also make title ungeneratable locally
//                                     }
//                                 },
//                             }
//                         }
//                     }

//                     if !found {
//                         // TODO: error w/ couldn't find window
//                         panic!("couldn't find window")
//                     }
//                 }
//                 ExecScope::Foreign => {
//                     let id = self
//                         .global_ids
//                         .entry(step.id)
//                         .or_insert_with(|| Uuid::new_v4().to_string());

//                     executor
//                         .execute(Command {
//                             id: id.clone(),
//                             action: step.details.action.clone(),
//                         })
//                         .unwrap();
//                 }
//             }

//             // TODO: verify props using fowin here
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ChronologyBuilder {
//     max_processes: u32,
//     max_windows: u32,
//     max_steps: u32,
// }

// impl ChronologyBuilder {
//     pub fn new() -> ChronologyBuilder {
//         ChronologyBuilder {
//             max_processes: 1,
//             max_windows: 1,
//             max_steps: 1,
//         }
//     }

//     pub fn max_processes(mut self, max: u32) -> Self {
//         self.max_processes = max;
//         self
//     }

//     pub fn max_windows(mut self, max: u32) -> Self {
//         self.max_windows = max;
//         self
//     }

//     pub fn max_steps(mut self, max: u32) -> Self {
//         self.max_steps = max;
//         self
//     }

//     pub fn build<R: Rng>(self, rng: &mut R) -> Chronology {
//         let num_processes = rng.gen_range(1..=self.max_processes);
//         let mut timelines = Vec::with_capacity(num_processes as usize);

//         let mut id = 0;
//         for i in 0..num_processes {
//             let num_windows = rng.gen_range(1..=self.max_windows);
//             let local_timelines = (0..num_windows)
//                 .map(|_| {
//                     let num_steps = rng.gen_range(1..=self.max_steps);
//                     let steps = Timeline::gen_details(num_steps as usize, rng)
//                         .into_iter()
//                         .map(|step| Step { id, details: step })
//                         .collect();

//                     id += 1;

//                     Timeline::new(steps)
//                 })
//                 // TODO: assign windows within each timeline to a process id based on their index in the final vec
//                 .collect::<Vec<Timeline>>();

//             // TODO: tack on a create/destroy to the start/end of each timeline
//             let timeline = Timeline::overlap(&local_timelines, rng);
//             timelines.push(timeline);
//         }

//         Chronology {
//             timeline: Timeline::overlap(&timelines, rng),
//             global_ids: HashMap::new(),
//         }
//     }
// }
