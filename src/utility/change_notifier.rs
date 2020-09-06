pub trait ChangeNotifier {
    fn request_change(&mut self);
}

pub trait ChangeNotifierObject<T: ChangeNotifier> {

    fn set_change_notifier(&mut self, notifier: T);

    fn clear_change_notifier(&mut self);
}
