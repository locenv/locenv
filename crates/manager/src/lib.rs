use context::Context;

pub fn is_running(ctx: &Context) -> bool {
    // Build path to the file containing port number.
    let mut port = ctx.manager_dir();

    port.push("port");

    port.is_file()
}
