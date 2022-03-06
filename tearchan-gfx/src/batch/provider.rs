use crate::batch::BatchEvent;

pub trait BatchProvider<'a> {
    type Context: 'a;
    fn run(&mut self, context: &mut Self::Context, event: BatchEvent);
}
