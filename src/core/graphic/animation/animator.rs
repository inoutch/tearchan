use serde::export::Option::Some;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use serde::export::fmt::Debug;

#[derive(Serialize, Deserialize)]
pub struct AnimationData<T, U>
where
    T: Hash + Eq,
{
    pub groups: HashMap<T, AnimationGroup<U>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationGroup<U> {
    pub frames: Vec<U>,
    pub duration_sec: f32,
}

pub struct Animator<T, U>
where
    T: Hash + Eq,
{
    current_state: T,
    current_time_ms: u64,
    current_animation_time_ms: u64,
    data: AnimationData<T, U>,
}

impl<T, U> Animator<T, U>
where
    T: Hash + Eq + Debug,
    U: Debug,
{
    pub fn new(data: AnimationData<T, U>, start_state: T) -> Self {
        debug_assert!(!data.groups.is_empty());

        let animation_time_ms = calc_animation_time(&data, &start_state);
        Animator {
            data,
            current_state: start_state,
            current_time_ms: 0,
            current_animation_time_ms: animation_time_ms,
        }
    }

    pub fn update(&mut self, delta_sec: f32) {
        self.current_time_ms += (delta_sec * 1000.0f32) as u64;
    }

    pub fn animation(&self) -> (&U, u64) {
        let group = &self.data.groups[&self.current_state];
        let duration = (group.duration_sec * 1000.0f32) as u64;
        let frame_index = (self.current_time_ms % self.current_animation_time_ms) / duration;
        (&group.frames[frame_index as usize], frame_index)
    }

    pub fn set_state(&mut self, state: T) {
        if self.current_state == state {
            return;
        }
        debug_assert!(self.data.groups.contains_key(&state));
        debug_assert!(!self.data.groups[&state].frames.is_empty());
        self.current_animation_time_ms = calc_animation_time(&self.data, &state);
        self.current_state = state;
        self.current_time_ms = 0;
    }
}

fn calc_animation_time<T, U>(data: &AnimationData<T, U>, state: &T) -> u64
where
    T: Hash + Eq,
{
    if let Some(group) = data.groups.get(state) {
        debug_assert!(!group.frames.is_empty());
        return group.frames.len() as u64 * (group.duration_sec * 1000.0f32) as u64;
    }
    std::u64::MAX
}

#[cfg(test)]
mod test {
    use crate::core::graphic::animation::animator::{AnimationData, AnimationGroup, Animator};
    use std::collections::HashMap;

    #[derive(Hash, Eq, PartialEq, Debug)]
    enum ExampleState {
        Standing,
        Walking,
    }

    #[test]
    fn test_standard() {
        let mut groups: HashMap<ExampleState, AnimationGroup<()>> = HashMap::new();
        groups.insert(
            ExampleState::Standing,
            AnimationGroup {
                frames: vec![(), ()],
                duration_sec: 0.3,
            },
        );
        groups.insert(
            ExampleState::Walking,
            AnimationGroup {
                frames: vec![(), (), ()],
                duration_sec: 0.5,
            },
        );

        let data = AnimationData { groups };
        let mut animator = Animator::new(data, ExampleState::Standing);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }

        animator.update(0.29);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }

        animator.update(0.01);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 1);
        }

        animator.update(0.3);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }
    }

    #[test]
    fn test_one_frame() {
        let mut groups: HashMap<ExampleState, AnimationGroup<()>> = HashMap::new();
        groups.insert(
            ExampleState::Standing,
            AnimationGroup {
                frames: vec![()],
                duration_sec: 0.5,
            },
        );

        let data = AnimationData { groups };
        let mut animator = Animator::new(data, ExampleState::Standing);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }

        animator.update(2.29);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }

        animator.update(0.38);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }

        animator.update(2_938.293_7);
        {
            let animation = animator.animation();
            assert_eq!(animation.1, 0);
        }
    }

    #[test]
    fn test_state() {
        let mut groups: HashMap<ExampleState, AnimationGroup<String>> = HashMap::new();
        groups.insert(
            ExampleState::Standing,
            AnimationGroup {
                frames: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                duration_sec: 0.01,
            },
        );
        let data = AnimationData { groups };
        let mut animator = Animator::new(data, ExampleState::Standing);
        {
            let animation = animator.animation();
            assert_eq!(animation.0, "A");
            assert_eq!(animation.1, 0);
        }

        animator.update(0.009);
        {
            let animation = animator.animation();
            assert_eq!(animation.0, "A");
            assert_eq!(animation.1, 0);
        }

        animator.update(0.001);
        {
            let animation = animator.animation();
            assert_eq!(animation.0, "B");
            assert_eq!(animation.1, 1);
        }
    }
}
