use crate::batch::v2::BatchEvent;

pub trait BatchProvider<'a> {
    type Context: 'a;
    fn run(&mut self, context: &mut Self::Context, event: BatchEvent);
}
