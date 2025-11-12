use clap::{arg, command, value_parser};

fn main() {
	let matches = command!()
		.arg(
			arg!(<INPUT> "Input file to process")
				.value_parser(value_parser!(String)),
		)
		.arg(
			arg!(-o --output <FORMAT> "Output format")
				.required(false)
				.value_parser(["json", "plain"])
				.default_value("json"),
		)
		.get_matches();

	println!("{:?}", matches.get_one::<String>("INPUT"));
}
