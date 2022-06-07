use context::Context;

pub fn is_running(context: &Context) -> bool {
    context.runtime().manager().port().is_file()
}
