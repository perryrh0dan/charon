use clap::ArgMatches;

pub struct Context {
  pub yes: bool,
  pub no_script: bool,
  pub verbose: bool,

}

impl Context {
  pub fn new(args: &ArgMatches) -> Context {
    let mut ctx = Context {
      yes: false,
      no_script: false,
      verbose: false,
    };

    ctx.set_yes(args.is_present("yes"));
    ctx.set_no_script(args.is_present("no-script"));
    ctx.set_verbose(args.is_present("verbose"));

    return ctx;
  }

  fn set_yes(&mut self, yes: bool) {
    self.yes = yes;
  }

  fn set_no_script(&mut self, ns: bool) {
    self.no_script = ns;
  }

  fn set_verbose(&mut self, verbose: bool) {
    self.verbose = verbose
  }
}
