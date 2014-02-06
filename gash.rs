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

use std::{io, run, os};
use std::io::buffered::BufferedReader;
use std::io::signal::{Listener, Interrupt};
use std::run::Process;
use std::io::stdin;
use std::str;
use extra::getopts;

struct Shell {
    cmd_prompt: ~str,
}

impl Shell {

    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
        }
    }
    
    fn run(&mut self) {

        
        spawn(proc() {
            let mut listener = Listener::new();
            listener.register(Interrupt);
            loop {
                match listener.port.recv() {
                    Interrupt => (),
                    _ => (),
                }
            }
        });


        let mut stdin = BufferedReader::new(stdin());

        let mut cmdHist = ~[];
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            cmdHist.push(cmd_line.clone());
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            
            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                "cd"    =>  { self.run_cd(cmd_line); }
                "history" => { self.run_history(cmdHist.clone()); }
                _       =>  { self.run_cmdline(cmd_line); }
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
        if(argv.len() > 0){
            if(argv[argv.len()-1] == ~"&") {
                let tempProgram: ~str = program.clone().to_owned();
                let tempArgv: ~[~str] = argv.clone().to_owned();
                spawn(proc() { 
                    let ret = run::process_output("which", [tempProgram.to_owned()]);
                    let ifExists: bool = ret.expect("exit code error.").status.success();
                    if ifExists {
                        run::process_status(tempProgram, tempArgv);

                    } else {
                        println!("{:s}: command not found", tempProgram);
                    }
                });
            }
        }
        else{
            if self.cmd_exists(program) {
                let output_options = run::process_status(program, argv);

                // loop {
                //     let output_bytes: &~[u8] = &output_options.unwrap().output;
                //     //let output_bytes_new = str::from_utf8(output_bytes.to_owned());
                //     //let s = str::from_utf8(output_bytes.clone().to_owned());
                //     println(output_bytes.to_str());
                // }
                
                //println!("Output from command that was run in the background:\n{:s}",s);
            } else {
                println!("{:s}: command not found", program);
            }
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }

    fn run_cd(&mut self, cmd_line: &str) {
        let arguments: ~[&str] = cmd_line.split(' ').collect(); 
        if(arguments.len() == 2) {
            let dir = arguments[1].clone();
            let path = Path::new(dir.clone());
            if(os::change_dir(&path)){
                let cwd = os::getcwd();
                println!("{}", cwd.display());
            }
            else{
                println("Directory does not exist.");
            }
        }
        else {
            println("Please input only one argument.");
        }
    }

    fn run_history(&mut self, cmd_hist: ~[~str]) {
        for i in range(0, cmd_hist.len()) { 
            println!("{}", cmd_hist[i]);
        }
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
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
