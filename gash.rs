//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::io::signal::{Listener, Interrupt};
use std::{io, run, os, str};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use std::io::fs::File;
use extra::getopts;


struct Shell {
    cmd_prompt: ~str,
    history: ~[~str],
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
            history: ~[],
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            let writeRedirect = cmd_line.find_str(" > ");
            let readRedirect = cmd_line.find_str(" < ");
            let cdFound = cmd_line.find_str("cd ");
            println(cdFound);
            if(writeRedirect == None && readRedirect == None && cdFound != None) {
            	let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            	self.history.push(cmd_line.clone());
            	match cmd_line.slice_from(cmd_line.len() - 2) {
            		" &"    => { self.run_background(program, cmd_line);
            			continue; }
                	_	=> { }
            	}
            	match program {
			""           =>  { continue; }
			"exit"       =>  { return; }
			"cd"	     =>  { self.run_cd(cmd_line); }
			"history"    =>  { self.run_history(); }
			_            =>  { self.run_cmdline(cmd_line); }
            	}
            }
            else if (readRedirect == None) {
            	let cmd1 = cmd_line.slice(0, writeRedirect.unwrap());
            	let cmd2 = cmd_line.slice_from(writeRedirect.unwrap()+3);
            	let program = cmd1.splitn(' ', 1).nth(0).expect("no program");
            	let args : ~[&str] = if (cmd1.splitn(' ', 1).nth(1) != None) {cmd1.splitn(' ', 1).nth(1).unwrap().split(' ').collect()} else {~[]};
            	let mut argsOwned : ~[~str] = ~[];
            	for i in range (0, args.len()) {
            		argsOwned.push(args[i].to_owned());
            	}
		let newStdOut = File::create(&Path::new(cmd2));
            	match newStdOut {
            		Some(mut x) => {
            			let process = run::Process::new(program, argsOwned, run::ProcessOptions::new());
            			let output = process.unwrap().finish_with_output();
            			x.write(output.output);
            		}
            		None => { println("shouldnt be here");}
            	}
            }
            else if (writeRedirect == None) {
            	let cmd1 = cmd_line.slice(0, readRedirect.unwrap());
            	let cmd2 = cmd_line.slice_from(readRedirect.unwrap()+3);
            	let program = cmd1.splitn(' ', 1).nth(0).expect("no program");
            	let args : ~[&str] = if (cmd1.splitn(' ', 1).nth(1) != None) {cmd1.splitn(' ', 1).nth(1).unwrap().split(' ').collect()} else {~[]};
            	let mut argsOwned : ~[~str] = ~[];
            	for i in range (0, args.len()) {
            		argsOwned.push(args[i].to_owned());
            	}
           	let path = &Path::new(cmd2);
            	if (path.exists()) {
		let input = File::open(path);
            		match input {
            			Some(mut x) => {
            				let inputBytes = x.read_to_end();
            				let process = run::Process::new(program, argsOwned, run::ProcessOptions::new());
            				match(process) {
            					Some(mut toWrite) => {
            						toWrite.input().write(inputBytes);
            						let output = toWrite.finish_with_output();
            						print!("{:s}", str::from_utf8(output.output));
            					}
            					None => {}
        				}
            			}
            			None => { println!("gash: {:s}: No such file or directory", cmd2);}
            		}
            	}
                else {
           		println!("gash: {:s}: No such file or directory", cmd2);
           	}
            }
        }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            self.run_cmd(program, argv);
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
            run::process_status(program, argv);
        } else {
            println!("{:s}: command not found", program);
        }
    }

    fn run_cd(&mut self, program: &str) {
        let mut argv: ~[~str] =
            program.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
    	let mut programs : ~str = ~"";
            if argv.len() > 0 {
                programs = argv.remove(1);
            }
    	os::change_dir(&Path::new(programs.clone()));
    }

    fn run_history(&mut self) {
        for c in range(0, self.history.len()) {
            println!("{:s}", self.history[c]);
        }
    }

    fn run_background(&mut self, program: &str, cmd_line: &str) {
		let (portSelf, chanSelf): (Port<~Shell>, Chan<~Shell>) = Chan::new();
		let (portProg, chanProg): (Port<~str>, Chan<~str>) = Chan::new();
		let (portLine, chanLine): (Port<~str>, Chan<~str>) = Chan::new();
		
		let mut sendShell = ~(Shell::new("gash > "));
		sendShell.history = self.history.clone();
		chanSelf.send(sendShell);
		chanProg.send(program.to_owned());
		chanLine.send(cmd_line.to_owned());

		do spawn { 
		let program : ~str = portProg.recv();
		let cmd_line : ~str = portLine.recv();
		let mut selfV : ~Shell = portSelf.recv();
                match program {
             	   ~""           =>  { }
              	  ~"exit"       =>  { }
		  ~"cd"	     =>  { selfV.run_cd(cmd_line); }
                  ~"history"    =>  { selfV.run_history(); }
                  _            =>  { selfV.run_cmdline(cmd_line); }
                }
            };
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    
    let opts = ~[
        getopts::optopt("c")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    let mut listener = Listener::new();
    listener.register(Interrupt);

    do spawn{
        loop {
            match listener.port.recv() {
                Interrupt => println!("Got Interrupt'ed"),
                _ => (),
            }
        }
    };
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
