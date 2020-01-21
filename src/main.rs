use clap::App;

fn main() {
    let matches = App::new("Hekk")
        .author("adumbidiot")
        .about("A tool to hekk things")
        .subcommand(hekk::commands::adapter_info::cli())
        .get_matches();

    match matches.subcommand() {
        ("adapter-info", Some(matches)) => hekk::commands::adapter_info::exec(&matches),
        (cmd, _) => {
            println!("Unknown command: {:#?}", cmd);
        }
    }
}
