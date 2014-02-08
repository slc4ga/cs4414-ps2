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

use std::libc::funcs::posix88;
use std::io::signal::{Listener, Interrupt};
use std::{io, run, os, str, vec, clone};
use std::io::buffered::BufferedReader;
use std::io::mem::BufWriter;
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
            if (cmd_line == ~"") {continue;}
            self.history.push(cmd_line.clone());
            let decomposed = Shell::decompose_Cmdline(cmd_line);
            let mut error : bool = false;
		for j in range(0, decomposed.len()) {
			if(Shell::checkForError(Some(~decomposed.clone()[j]))) {error = true;}
		}
		if error {continue;}
		for j in range(0, decomposed.len()) {
			let check = self.runDecomposed(Some(~decomposed.clone()[j]), ~[]);
			if check==~"exit" { return; }
		}
/*
            let writeRedirect = cmd_line.find_str(" > ");
            let readRedirect = cmd_line.find_str(" < ");
            if(writeRedirect == None && readRedirect == None) {
            	let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            	self.history.push(cmd_line.clone());
            	match cmd_line.slice_from(cmd_line.len() - 2) {
            		" &"    => { self.run_background(program, cmd_line.slice(0, cmd_line.len() - 2));
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
*/
        }
    }

	fn checkForError(cmd : Option<~DecomposedCmd>) -> bool {
		match cmd {
			Some(comm) 	=> {
				if comm.error {return true;}
				else {return Shell::checkForError(comm.pipeToNext);}
			}
			None		=> {
				return false;
			}
		}
	}
	
	fn runDecomposed (&mut self, cmd : Option<~DecomposedCmd>, input : ~[u8]) -> ~str {
		match cmd {
			Some(useCmd) => {
			if useCmd.background {
				let mut shellCopy = ~(Shell::new("gash > "));
				shellCopy.history = self.history.clone();
				let (portSelf, chanSelf): (Port<~Shell>, Chan<~Shell>) = Chan::new();
		
				chanSelf.send(shellCopy);
				
				do spawn { 
					let mut output : ~[u8] = ~[];
					let mut shellCopy : ~Shell = portSelf.recv();
					match useCmd.program {
						~""		=>  { }
						~"exit"		=>  { }
						~"cd"		=>  {
							if(useCmd.args.len() == 1) {
								os::change_dir(&Path::new(useCmd.args[0]));
							}
							else {
								println("Usage: cd <directory>");
							}
						}
						~"history"	=>  {
							match useCmd.outputFile.clone() {
								Some(fileName)	=> {
									let mut newStdOut = File::create(&Path::new(fileName));
									for z in range (0, shellCopy.history.len()) {
										newStdOut.write_str(shellCopy.history[z]);
										newStdOut.write_char('\n');
									}
								}
								_		=> {
									match useCmd.pipeToNext.clone() {
										Some(pipTo)	=> {
											for z in range (0, shellCopy.history.len()) {
												let temp = (shellCopy.history[z] + "\n");
												let tempBytes = temp.as_bytes();
												output = vec::append(output, tempBytes);
											}
										}
										_		=> { shellCopy.run_history(); }
									}
								}
							}
						}
						_		=>  { output = shellCopy.runDecomposedUnit(*useCmd, input); }
						
                			}
				
					shellCopy.runDecomposed(useCmd.pipeToNext.clone(), match useCmd.pipeToNext {Some(x) => {output} _ => {~[]} });
				}
			}
			else {
				let mut output : ~[u8] = ~[];
				match  useCmd.program {
					~""		=>  { }
					~"exit"		=>  { return ~"exit"; }
					~"cd"		=>  {
						if(useCmd.args.len() == 1) {
							os::change_dir(&Path::new(useCmd.args.clone()[0]));
						}
						else {
							println("Usage: cd <directory>");
						}
					}
					~"history"	=>  { 
						match useCmd.outputFile.clone() {
							Some(fileName)	=> {
								let mut newStdOut = File::create(&Path::new(fileName));
								for z in range (0, self.history.len()) {
									newStdOut.write_str(self.history[z]);
									newStdOut.write_char('\n');
								}
							}
							_		=> {
								match useCmd.pipeToNext.clone() {
									Some(pipTo)	=> {
										for z in range (0, self.history.len()) {
											let temp = (self.history[z] + "\n");
											let tempBytes = temp.as_bytes();
											output = vec::append(output, tempBytes);
										}
									}
									_		=> { self.run_history(); }
								}
							}
						}
					}
					_		=>  { output = self.runDecomposedUnit(*useCmd.clone(), input); }
				}
				self.runDecomposed(useCmd.pipeToNext.clone(), match useCmd.pipeToNext {Some(x) => {output} _ => {~[]} });
			}
			}
			_	=> {}
		}
		~""
	}

	fn runDecomposedUnit (&mut self, cmd : DecomposedCmd, input : ~[u8]) -> ~[u8] {
		match (cmd.inputFile.clone(), cmd.outputFile.clone()) {
			(Some(input), Some(output))	=> {
				let path = &Path::new(input.clone());
				let mut newStdOut = File::create(&Path::new(output));
				if (path.exists()) {
					let inputFile = File::open(path);
					match inputFile {
						Some(mut x) => {
							let inputBytes = x.read_to_end();
							let process = run::Process::new(cmd.program, cmd.args, run::ProcessOptions::new());
							match(process) {
								Some(mut toWrite) => {
									toWrite.input().write(inputBytes);
									let output = toWrite.finish_with_output();
									newStdOut.unwrap().write(output.output);
									return output.output;
								}
								None => {}
							}
						}
						None => { println!("gash: {:s}: No such file or directory", input);}
					}
				}
				else {
					println!("gash: {:s}: No such file or directory", input);
				}
			}
			(Some(input), _)		=> {
				let path = &Path::new(input.clone());
				if (path.exists()) {
					let inputFile = File::open(path);
					match inputFile {
						Some(mut x) => {
							let inputBytes = x.read_to_end();
							let process = run::Process::new(cmd.program, cmd.args, run::ProcessOptions::new());
							match(process) {
								Some(mut toWrite) => {
									toWrite.input().write(inputBytes);
									let output = toWrite.finish_with_output();
									match cmd.pipeToNext { None => {print!("{:s}", str::from_utf8(output.output));} _ => {}}
									return output.output;
								}
								None => {}
							}
						}
						None => { println!("gash: {:s}: No such file or directory", input);}
					}
				}
				else {
					println!("gash: {:s}: No such file or directory", input);
				}
			}
			(_, Some(output))		=> {
				let newStdOut = File::create(&Path::new(output));
				match newStdOut {
					Some(mut x) => {
						let process = run::Process::new(cmd.program, cmd.args, run::ProcessOptions::new());
						match process {
							Some(mut pros)	=> {
								if(input != ~[]) { pros.input().write(input); }
								let output = pros.finish_with_output();
								x.write(output.output);
								return output.output;
							}
							_		=> {}
						}
					}
					None => { println("shouldnt be here");}
				}
			}
			(_, _)				=> {
				//let output = self.run_cmdline(cmd.cmd_line);

				//match cmd.pipeToNext { None => {print!("{:s}", str::from_utf8(output));} _ => {}}
				//return output;
				let mut processOptions = run::ProcessOptions::new();
				match cmd.pipeToNext { None => {processOptions.out_fd = Some(1);} _ => {}}
				let process = run::Process::new(cmd.program, cmd.args, processOptions);
				match process {
					Some(mut pros)	=> {
						if(input != ~[]) { pros.input().write(input); }
						let output = pros.finish_with_output();
						//match cmd.pipeToNext { None => {print!("{:s}", str::from_utf8(output.output));} _ => {}}
						return output.output;
					}
					_		=> {}
				}
			}
		}
		~[]
	}

	fn decompose_Cmdline (cmd_line: &str) -> ~[DecomposedCmd]{
		let mut decomposed = ~[];
		let mut decomposedCmd = DecomposedCmd {
			cmd_line: ~"",
			program: ~"",
			args: ~[],
			background: false,
			inputFile: None,
			outputFile: None,
			pipeToNext: None,
			error: false,
		};

		let background = cmd_line.find_str("&");
		match background {
			Some(index)	=> {
				if(index==0) {
					println("Syntax error near '&'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd1 = cmd_line.slice(0, index).trim();
				let cmd2 = if(index < cmd_line.len()) {cmd_line.slice_from(index+1).trim()} else {""};
				decomposed = Shell::decompose_Cmdline(cmd1);
				decomposed[0].background = true;
				if(cmd2!="") { 
					decomposed = vec::append(decomposed, Shell::decompose_Cmdline(cmd2));
				}
				return decomposed;
			}
			_		=> {
			}
		}

		let pipe = cmd_line.find_str("|");
		match pipe {
			Some(index)	=> {
				if(index==0 || index == cmd_line.len()-1) {
					println("Syntax error near '|'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd1 = cmd_line.slice(0, index).trim();
				let cmd2 = cmd_line.slice_from(index+1).trim();
				decomposed = Shell::decompose_Cmdline(cmd1);
				decomposed[decomposed.len() - 1].pipeToNext = Some(~Shell::decompose_Cmdline(cmd2)[0]);
				return decomposed;
			}
			_		=> {
			}
		}

		let writeRedirect = cmd_line.find_str(">");
		let readRedirect = cmd_line.find_str("<");
		match (readRedirect, writeRedirect) {
			(Some(read), Some(write))	=> {
				if(write==0 || write == cmd_line.len()-1) {
					println("Syntax error near '>'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				if(read==0 || read == cmd_line.len()-1) {
					println("Syntax error near '<'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				if(read>write) {
					let cmd = cmd_line.slice(0, write).trim().to_owned();
					let output = cmd_line.slice(write+1, read).trim().to_owned();
					let input = cmd_line.slice_from(read+1).to_owned();
					decomposedCmd.outputFile = Some(output);
					decomposedCmd.inputFile = Some(input);
					decomposedCmd.cmd_line = cmd.clone();
					decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
					let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
					args.remove(0);
				}
				else {
					let cmd = cmd_line.slice(0, read).trim().to_owned();
					let input = cmd_line.slice(read+1, write).trim().to_owned();
					let output = cmd_line.slice_from(write+1).trim().to_owned();
					decomposedCmd.inputFile = Some(input);
					decomposedCmd.outputFile = Some(output);
					decomposedCmd.cmd_line = cmd.clone();
					decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
					let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
					args.remove(0);
					decomposedCmd.args = args;
				}
			}
			(Some(read), _)			=> {
				if(read==0 || read == cmd_line.len()-1) {
					println("Syntax error near '<'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd = cmd_line.slice(0, read).trim().to_owned();
				let input = cmd_line.slice_from(read+1).trim().to_owned();
				decomposedCmd.inputFile = Some(input);
				decomposedCmd.cmd_line = cmd.clone();
				decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
			(_, Some(write))		=> {
				if(write==0 || write == cmd_line.len()-1) {
					println("Syntax error near '>'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd = cmd_line.slice(0, write).trim().to_owned();
				let output = cmd_line.slice_from(write+1).trim().to_owned();
				decomposedCmd.outputFile = Some(output);
				decomposedCmd.cmd_line = cmd.clone();
				decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
			(_, _)				=> {
				decomposedCmd.cmd_line = cmd_line.to_owned();
				decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
		}
		
		decomposed.push(decomposedCmd);
		return decomposed;
	}
    
    fn run_cmdline(&mut self, cmd_line: &str) -> ~[u8]{
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            return self.run_cmd(program, argv);
        }
	return ~[];
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) -> ~[u8] {
        if self.cmd_exists(program) {
		let mut output : ~[u8] = ~[];
		let stdinHandle = io::stdio::stdin();
		//let oldStdout = ~io::stdio::stdout();
		{
			//let newStdout = ~BufWriter::new(output);
			//io::stdio::set_stdout(newStdout);
			let out = run::process_output(program, argv);
			match out {
				Some(procOut)	=> {
					output = procOut.output;
				}
				None		=> {}
			}
		}
		//io::stdio::set_stdout(oldStdout);
		return output;
        } else {
		println!("{:s}: command not found", program);
		return ~[];
        }
    }

    fn run_cd(&mut self, program: &str) {
        let mut argv: ~[~str] =
            program.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
    	let mut programs : ~str = ~"";
            if argv.len() > 1 {
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

    let (portSelf, chanSelf): (Port<Listener>, Chan<Listener>) = Chan::new();    
    chanSelf.send(listener);

    do spawn{
        loop {
            let listener = portSelf.recv();
            match listener.port.recv() {
                Interrupt => { println!("Got Interrupt'ed");
                                unsafe { posix88::signal::kill(std::libc::getpid() , std::libc::SIGINT); }
                            }
                _ => (),
            }
        }
    };
    
    match opt_cmd_line {
        Some(cmd_line) => {Shell::new("").run_cmdline(cmd_line);},
        None           => Shell::new("gash > ").run()
    }
}

struct DecomposedCmd {
	cmd_line: ~str,
	program: ~str,
	args: ~[~str],
	background: bool,
	inputFile: Option<~str>,
	outputFile: Option<~str>,
	pipeToNext: Option<~DecomposedCmd>,
	error: bool,
}

impl clone::Clone for DecomposedCmd {
	fn clone(&self) -> DecomposedCmd {
		DecomposedCmd {
			cmd_line: self.cmd_line.clone(),
			program: self.program.clone(),
			args: self.args.clone(),
			background : self.background,
			inputFile: self.inputFile.clone(),
			outputFile: self.outputFile.clone(),
			pipeToNext: self.pipeToNext.clone(),
			error: self.error,
		}
	}
}

impl DecomposedCmd {
	fn print(&self) {
		print!("cmd_line: {:s}\nprogram: {:s}\nargs:", self.cmd_line, self.program);
		if self.args.len() > 0 {
			for i in range(0, self.args.len()) {
				print!("\n\t{:s}", self.args[i]);
			}
		}
		println!("\nbackground: {:b}\ninputFile: {:s}\noutputFile: {:s}\nerror: {:b}", self.background, 
			match self.clone().inputFile { Some(name) => {name} _ => {~""} }, match self.clone().outputFile { Some(name) => {name} _ => {~""} }, self.error);
		match self.clone().pipeToNext {
			Some(next) => {
				println("Pipe to:");
				next.print();
			}
			_ => {println("Pipe to: None");}
		}
	}
}
